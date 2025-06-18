use crate::bitboard::Bitboard;
use crate::coords::{Line, Square, E, N, NE, NW, S, SE, SW, W};
use crate::piece::Color;
use std::ops::{Index, IndexMut};
use Color::*;
use Line::*;

const fn precompute_ranks() -> SquareIndex<Bitboard> {
    let mut ranks = SquareIndex::new();
    const_for!(y in 0 .. 8 => {
        let mask = 0b11111111 << (8 * y);
        const_for!(x in 0 .. 8 => {
            let pt = Bitboard::at(Square::xy(x, y));
            *ranks.get_mut(pt) = Bitboard(mask);
        });
    });
    ranks
}

const fn precompute_files() -> SquareIndex<Bitboard> {
    let mut files = SquareIndex::new();
    const_for!(x in 0 .. 8 => {
        let mask = 0x0101010101010101 << x;
        const_for!(y in 0 .. 8 => {
            let pt = Bitboard::at(Square::xy(x, y));
            *files.get_mut(pt) = Bitboard(mask);
        });
    });
    files
}

const fn precompute_rank_attacks() -> [[Bitboard; 64]; 8] {
    let mut attacks: [[Bitboard; 64]; 8] = [[Bitboard::empty(); 64]; 8];
    const_for!(rook_file in 0 .. 8 => {
        const_for!(file_pattern in 0 .. 64 => {
            let real_pattern = file_pattern << 1;
            let mut attack_bb = Bitboard::empty();
            const_for!(i in rook_file+1 .. 8 => {
                let bit = 1 << (7 - i);
                attack_bb.0 |= bit;
                if real_pattern & bit != 0 {
                    break;
                }
            });
            const_for!(i in rook_file-1 .. 0 => {
                let bit = 1 << (7 - i);
                attack_bb.0 |= bit;
                if real_pattern & bit != 0 {
                    break;
                }
            });
            attacks[rook_file as usize][file_pattern as usize] = attack_bb;
        });
    });
    attacks
}

const fn precompute_diagonals(delta: i32) -> SquareIndex<Bitboard> {
    let mut diagonals = SquareIndex::new();
    const_for!(x0 in 0 .. 8 => {
        const_for!(y0 in 0 .. 8 => {
            let pt = Bitboard::at(Square::xy(x0, y0));
            let mut x: i32 = x0;
            let mut y: i32 = y0;
            const_for!(i in 0 .. 8 => {
                diagonals.get_mut(pt).0 |= Bitboard::at(Square::xy(x, y)).0;
                x = (x + 1) % 8;
                y = (y + delta).rem_euclid(8);
            })
        })
    });
    diagonals
}

pub const RANKS: SquareIndex<Bitboard> = precompute_ranks();
pub const FILES: SquareIndex<Bitboard> = precompute_files();
// BEWARE: this is NOT a SquareIndex, as it's indexed by compressed rank occupancy patterns
pub const RANK_ATTACKS: [[Bitboard; 64]; 8] = precompute_rank_attacks();
pub const NW_DIAGONALS: SquareIndex<Bitboard> = precompute_diagonals(1);
pub const SW_DIAGONALS: SquareIndex<Bitboard> = precompute_diagonals(-1);

#[derive(Clone, Copy)]
pub struct SquareIndex<T>([T; 64]);

impl SquareIndex<Bitboard> {
    const fn new() -> Self {
        SquareIndex([Bitboard(0); 64])
    }
}

impl <T> SquareIndex<T> {
    const fn get(&self, bb: Bitboard) -> &T {
        &self.0[bb.to_index()]
    }
    const fn get_mut(&mut self, bb: Bitboard) -> &mut T {
        &mut self.0[bb.to_index()]
    }
}

impl <T> Index<Bitboard> for SquareIndex<T> {
    type Output = T;
    fn index(&self, bb: Bitboard) -> &Self::Output {
        debug_assert_eq!(bb.0.count_ones(), 1);
        self.get(bb)
    }
}

impl <T> IndexMut<Bitboard> for SquareIndex<T> {
    fn index_mut(&mut self, bb: Bitboard) -> &mut T {
        debug_assert_eq!(bb.0.count_ones(), 1);
        self.get_mut(bb)
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
pub const KNIGHT_ATTACKS: SquareIndex<Bitboard> = precompute_jump_attacks(&KNIGHT_JUMPS);
pub const KING_ATTACKS: SquareIndex<Bitboard> = precompute_jump_attacks(&KING_JUMPS);

const fn precompute_jump_attacks<const N: usize>(d: &[(i32, i32); N]) -> SquareIndex<Bitboard> {
    let mut attacks = SquareIndex::new();
    const_for!(x in 0 .. 8 => {
        const_for!(y in 0 .. 8 => {
            let mut attack_pat: u64 = 0;
            const_for!(i in 0 .. N => {
                let (dx, dy) = d[i];
                if let Some(sq) = Square::xy_checked(x+dx, y+dy) {
                    attack_pat |= Bitboard::at(sq).0
                }
            });
            // TODO: use a reasonable convention here
            *attacks.get_mut(Bitboard::at(Square::xy(x, y))) = Bitboard(attack_pat)
        });
    });
    attacks
}

pub const REV_PAWN_MOVES: [SquareIndex<Bitboard>; 2] = precompute_rev_pawn_moves();
pub const REV_PAWN_DBL_MOVES: [SquareIndex<Bitboard>; 2] = precompute_rev_pawn_dbl_moves();
pub const REV_PAWN_ATTACKS: [SquareIndex<Bitboard>; 2] = precompute_rev_pawn_attacks();
pub const REV_PAWN_EP_ATTACKS: [[SquareIndex<Bitboard>; 9]; 2] = precompute_rev_pawn_ep_attacks();

const fn precompute_rev_pawn_ep_attacks() -> [[SquareIndex<Bitboard>; 9]; 2] {
    let mut rev_attacks = [[SquareIndex::new(); 9]; 2];
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

const fn precompute_rev_pawn_attacks() -> [SquareIndex<Bitboard>; 2] {
    let mut rev_attacks = [SquareIndex::new(); 2];
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

const fn precompute_rev_pawn_moves() -> [SquareIndex<Bitboard>; 2] {
    let mut rev_moves = [SquareIndex::new(); 2];
    const_for!(x in 0 .. 8 => {
        const_for!(y in 0 .. 8 => {
            let idx = (63 - x - y * 8) as u64;
            let bb = 1 << idx;
            rev_moves[White as usize].0[idx as usize] = Bitboard(bb << 8);
            rev_moves[Black as usize].0[idx as usize] = Bitboard(bb >> 8);
        });
    });
    rev_moves
}

const fn precompute_rev_pawn_dbl_moves() -> [SquareIndex<Bitboard>; 2] {
    let mut rev_moves = [SquareIndex::new(); 2];
    // Redundant computation, but it's all done in compile time
    const_for!(x in 0 .. 8 => {
        const_for!(y in 0 .. 8 => {
            let idx = (63 - x - y * 8) as u64;
            let bb = 1 << idx;
            let white_pat = bb << 16 & Bitboard::line(AtY(1)).0;
            rev_moves[White as usize].0[idx as usize] = Bitboard(white_pat);
            let black_pat = bb >> 16 & Bitboard::line(AtY(6)).0;
            rev_moves[Black as usize].0[idx as usize] = Bitboard(black_pat);
        });
    });
    rev_moves
}

// Not quite attacktables, but we store the occlusion maps for castling in here
pub const LONG_CASTLE_PATH: [Bitboard; 2] = [
    Bitboard::from_bytes([0b01110000, 0, 0, 0, 0, 0, 0, 0]),
    Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b01110000]),
];
pub const SHORT_CASTLE_PATH: [Bitboard; 2] = [
    Bitboard::from_bytes([0b00000110, 0, 0, 0, 0, 0, 0, 0]),
    Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b00000110]),
];
pub const LONG_CASTLE_MID_SQUARE: [Bitboard; 2] = [
    Bitboard::at(Square::xy(3, 7)),
    Bitboard::at(Square::xy(3, 0)),
];
pub const SHORT_CASTLE_MID_SQUARE: [Bitboard; 2] = [
    Bitboard::at(Square::xy(5, 7)),
    Bitboard::at(Square::xy(5, 0)),
];
