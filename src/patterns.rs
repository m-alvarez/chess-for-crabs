use crate::bitboard::Bitboard;
use crate::coords::{Line, Rel, Square, E, N, NE, NW, S, SE, SW, W};
use std::ops::Index;
use Line::*;

pub const fn ray(square: Square<Rel>, d: (i32, i32)) -> Bitboard {
    let (dx, dy) = d;
    let mut pattern: u64 = 0;
    let mut x = square.x as i32 + dx;
    let mut y = square.y as i32 + dy;
    while let Some(mv) = Square::<Rel>::xy(x, y) {
        pattern |= Bitboard::at(mv).0;
        x += dx;
        y += dy;
    }
    Bitboard(pattern)
}

pub const fn rays<const N: usize>(square: Square<Rel>, ds: [(i32, i32); N]) -> Bitboard {
    let mut pattern = 0;
    const_for!(i in 0 .. N => {
        pattern |= ray(square, ds[i]).0
    });
    Bitboard(pattern)
}

#[derive(Copy, Clone)]
pub struct Ray {
    pub pos: Bitboard,
    pub neg: Bitboard,
}

pub struct Raytable<const N: usize>([[[Ray; N]; 8]; 8]);
impl<const N: usize> Raytable<N> {
    pub const fn rays(&self, sq: Square<Rel>) -> &[Ray; N] {
        &self.0[sq.x as usize][sq.y as usize]
    }
}
impl<const N: usize> Index<Square<Rel>> for Raytable<N> {
    type Output = [Ray; N];
    fn index(&self, idx: Square<Rel>) -> &Self::Output {
        self.rays(idx)
    }
}

const fn precompute_rays<const N: usize>(ds: [(i32, i32); N]) -> Raytable<N> {
    let empty_ray = Ray { pos: Bitboard::empty(), neg: Bitboard::empty() };
    let mut rays = [[[empty_ray; N]; 8]; 8];
    const_for!(x in 0 .. 8 => {
        const_for!(y in 0 .. 8 => {
            const_for!(i in 0 .. N => {
                let sq = Square { x: x as u8, y: y as u8 };
                rays[x][y][i].pos = ray(sq, ds[i]);
                rays[x][y][i].neg = ray(sq, (-ds[i].0, -ds[i].1));
            });
        });
    });
    Raytable::<N>(rays)
}
pub const BISHOP_RAYS: Raytable<2> = precompute_rays([NW, NE]);
pub const ROOK_RAYS: Raytable<2> = precompute_rays([N, E]);
pub const QUEEN_RAYS: Raytable<4> = precompute_rays([NW, N, NE, E]);

pub struct Attacktable([[Bitboard; 8]; 8]);
impl Attacktable {
    pub const fn attack(&self, sq: Square<Rel>) -> Bitboard {
        self.0[sq.x as usize][sq.y as usize]
    }
}
impl Index<Square<Rel>> for Attacktable {
    type Output = Bitboard;
    fn index(&self, idx: Square<Rel>) -> &Self::Output {
        &self.0[idx.x as usize][idx.y as usize]
    }
}

const fn precompute_attacks<const N: usize>(attack_rays: &Raytable<N>) -> Attacktable {
    let mut attacks = [[Bitboard::empty(); 8]; 8];
    const_for!(x in 0 .. 8 => {
        const_for!(y in 0 .. 8 => {
            let mut pattern: u64 = 0;
            const_for!(i in 0 .. N => {
                pattern |= attack_rays.0[x][y][i].pos.0;
                pattern |= attack_rays.0[x][y][i].neg.0;
            });
            attacks[x][y] = Bitboard(pattern);
        })
    });
    Attacktable(attacks)
}
pub const BISHOP_ATTACKS: Attacktable = precompute_attacks(&BISHOP_RAYS);
pub const ROOK_ATTACKS: Attacktable = precompute_attacks(&ROOK_RAYS);
pub const QUEEN_ATTACKS: Attacktable = precompute_attacks(&QUEEN_RAYS);

pub const fn jumps<const N: usize>(square: Square<Rel>, d: [(i32, i32); 8]) -> Bitboard {
    let mut pattern = 0;
    const_for!(i in 0 .. d.len() => {
        let (dx, dy) = d[i];
        if let Some(jump) = Square::<Rel>::xy(square.x as i32 + dx, square.y as i32 + dy) {
            pattern |= Bitboard::at(jump).0;
        }
    });
    Bitboard(pattern)
}

pub const fn knight_moves(square: Square<Rel>) -> Bitboard {
    jumps::<8>(
        square,
        [
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (-2, -1),
            (-2, 1),
            (2, -1),
            (2, 1),
        ],
    )
}

pub const fn pawn_moves(square: Square<Rel>) -> Bitboard {
    // Remember, we're always white
    let origin = Bitboard::at(square);
    // Shift up by one (or two)
    Bitboard((origin.0 >> 8) | (origin.0 >> 16 & Bitboard::line(AtY(3)).0))
}

pub const fn rev_pawn_moves(square: Square<Rel>) -> Bitboard {
    let destination = Bitboard::at(square);
    Bitboard((destination.0 << 8) | (destination.0 << 16 & Bitboard::line(AtY(1)).0))
}

pub const fn rook_moves(square: Square<Rel>) -> Bitboard {
    rays(square, [N, S, E, W])
}

pub const fn bishop_moves(square: Square<Rel>) -> Bitboard {
    rays(square, [NW, SW, NE, SE])
}

pub const fn queen_moves(square: Square<Rel>) -> Bitboard {
    Bitboard(bishop_moves(square).0 | rook_moves(square).0)
}

pub const fn king_moves(square: Square<Rel>) -> Bitboard {
    jumps::<8>(square, [N, S, E, W, NE, SE, NW, SW])
}
