use crate::bitboard::Bitboard;
use crate::coords::{Line, Square, E, N, NE, NW, S, SE, SW, W};
use crate::piece::Color;
use std::ops::Index;
use Color::*;
use Line::*;

pub const fn ray(square: Square, d: (i32, i32)) -> Bitboard {
    let (dx, dy) = d;
    let mut pattern: u64 = 0;
    let mut x = square.x as i32 + dx;
    let mut y = square.y as i32 + dy;
    while let Some(mv) = Square::xy_checked(x, y) {
        pattern |= Bitboard::at(mv).0;
        x += dx;
        y += dy;
    }
    Bitboard(pattern)
}

#[derive(Copy, Clone)]
pub struct Ray {
    pub pos: Bitboard,
    pub neg: Bitboard,
}

pub struct Raytable<const N: usize>([[Ray; N]; 64]);
impl<const N: usize> Raytable<N> {
    pub const fn rays(&self, pt: Bitboard) -> &[Ray; N] {
        &self.0[pt.to_index()]
    }
}
impl<const N: usize> Index<Bitboard> for Raytable<N> {
    type Output = [Ray; N];
    fn index(&self, idx: Bitboard) -> &Self::Output {
        self.rays(idx)
    }
}

const fn precompute_rays<const N: usize>(ds: [(i32, i32); N]) -> Raytable<N> {
    let empty_ray = Ray {
        pos: Bitboard::empty(),
        neg: Bitboard::empty(),
    };
    let mut rays = [[empty_ray; N]; 64];
    const_for!(x in 0 .. 8 => {
        const_for!(y in 0 .. 8 => {
            const_for!(i in 0 .. N => {
                let idx = (63 - x - y * 8) as usize;
                let sq = Square { x: x as u8, y: y as u8 };
                rays[idx][i].pos = ray(sq, ds[i]);
                rays[idx][i].neg = ray(sq, (-ds[i].0, -ds[i].1));
            });
        });
    });
    Raytable::<N>(rays)
}
pub const BISHOP_RAYS: Raytable<2> = precompute_rays([SE, SW]);
pub const ROOK_RAYS: Raytable<2> = precompute_rays([S, E]);
pub const QUEEN_RAYS: Raytable<4> = precompute_rays([SW, S, SE, E]);

#[derive(Clone, Copy)]
pub struct Attacktable([Bitboard; 64]);
impl Attacktable {
    pub const fn attack(&self, pt: Bitboard) -> &Bitboard {
        &self.0[pt.to_index()]
    }
}
impl Index<Bitboard> for Attacktable {
    type Output = Bitboard;
    fn index(&self, idx: Bitboard) -> &Self::Output {
        &self.attack(idx)
    }
}

const KNIGHT_JUMPS: [(i32, i32); 8] = [
    (-1, -2),
    (-1, 2),
    (1, -2),
    (1, 2),
    (-2, -1),
    (-2, 1),
    (2, -1),
    (2, 1),
];
const KING_JUMPS: [(i32, i32); 8] = [N, S, E, W, NE, SE, NW, SW];
pub const KNIGHT_ATTACKS: Attacktable = precompute_jump_attacks(&KNIGHT_JUMPS);
pub const KING_ATTACKS: Attacktable = precompute_jump_attacks(&KING_JUMPS);

const fn precompute_jump_attacks<const N: usize>(d: &[(i32, i32); N]) -> Attacktable {
    let mut attacks = [Bitboard::empty(); 64];
    const_for!(x in 0 .. 8 => {
        const_for!(y in 0 .. 8 => {
            let idx = (63 - x - y * 8) as usize;
            let mut attack_pat: u64 = 0;
            const_for!(i in 0 .. N => {
                let (dx, dy) = d[i];
                if let Some(sq) = Square::xy_checked(x+dx, y+dy) {
                    attack_pat |= Bitboard::at(sq).0
                }
            });
            // TODO: use a reasonable convention here
            attacks[idx] = Bitboard(attack_pat)
        });
    });
    Attacktable(attacks)
}

pub const REV_PAWN_MOVES: [Attacktable; 2] = precompute_rev_pawn_moves();
pub const REV_PAWN_ATTACKS: [Attacktable; 2] = precompute_rev_pawn_attacks();
pub const REV_PAWN_EP_ATTACKS: [[Attacktable; 9]; 2] = precompute_rev_pawn_ep_attacks();

const fn precompute_rev_pawn_ep_attacks() -> [[Attacktable; 9]; 2] {
    let mut rev_attacks = [[Attacktable([Bitboard::empty(); 64]); 9]; 2];
    const_for!(x_target in 0 .. 8 => {
        const_foreach!((color, y_target, dy) in [(White, 5, -1), (Black, 2, 1)] => {
            let mut pat = 0;
            if let Some(square) = Square::xy_checked(x_target-1, y_target + dy) {
                pat |= Bitboard::at(square).0
            }
            if let Some(square) = Square::xy_checked(x_target+1, y_target + dy) {
                pat |= Bitboard::at(square).0
            }
            let idx = Square::xy(x_target, y_target).index() as usize;
            rev_attacks[color as usize][(x_target + 1) as usize].0[idx] = Bitboard(pat)
        })
    });
    rev_attacks
}

const fn precompute_rev_pawn_attacks() -> [Attacktable; 2] {
    let mut rev_attacks = [Attacktable([Bitboard::empty(); 64]); 2];
    const_for!(x in 0 .. 8 => {
        const_for!(y in 0 .. 8 => {
            let idx = Square::xy(x, y).index();
            let mut white_pat = 0;
            if let Some(square) = Square::xy_checked(x-1, y-1) {
                white_pat |= Bitboard::at(square).0;
            }
            if let Some(square) = Square::xy_checked(x+1, y-1) {
                white_pat |= Bitboard::at(square).0;
            }
            rev_attacks[White as usize].0[idx as usize] = Bitboard(white_pat);
            let mut black_pat = 0;
            if let Some(square) = Square::xy_checked(x-1, y+1) {
                black_pat |= Bitboard::at(square).0
            }
            if let Some(square) = Square::xy_checked(x+1, y+1) {
                black_pat |= Bitboard::at(square).0;
            }
            rev_attacks[Black as usize].0[idx as usize] = Bitboard(black_pat);
        });
    });
    rev_attacks
}

const fn precompute_rev_pawn_moves() -> [Attacktable; 2] {
    let mut rev_moves = [Attacktable([Bitboard::empty(); 64]); 2];
    const_for!(x in 0 .. 8 => {
        const_for!(y in 0 .. 8 => {
            let idx = (63 - x - y * 8) as u64;
            let bb = 1 << idx;
            let white_pat = bb << 8 | (bb << 16 & Bitboard::line(AtY(1)).0);
            rev_moves[White as usize].0[idx as usize] = Bitboard(white_pat);
            let black_pat = bb >> 8 | (bb >> 16 & Bitboard::line(AtY(6)).0);
            rev_moves[Black as usize].0[idx as usize] = Bitboard(black_pat);
        });
    });
    rev_moves
}

// Not quite attacktables, but we store the occlusion maps for castling in here
pub const QUEENSIDE_CASTLE_PATH: [Bitboard; 2] = [
    Bitboard::from_bytes([0b01110000, 0, 0, 0, 0, 0, 0, 0]),
    Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b01110000]),
];
pub const KINGSIDE_CASTLE_PATH: [Bitboard; 2] = [
    Bitboard::from_bytes([0b00000110, 0, 0, 0, 0, 0, 0, 0]),
    Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b00000110]),
];
pub const QUEENSIDE_CASTLE_MID_SQUARE: [Bitboard; 2] = [
    Bitboard::at(Square::xy(3, 7)),
    Bitboard::at(Square::xy(3, 0)),
];
pub const KINGSIDE_CASTLE_MID_SQUARE: [Bitboard; 2] = [
    Bitboard::at(Square::xy(5, 7)),
    Bitboard::at(Square::xy(5, 0)),
];
