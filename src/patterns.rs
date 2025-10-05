use crate::bitboard::{Bitboard, LINE_AT_Y};
use crate::coords::Square;
use crate::piece::Color;
use std::ops::{Index, IndexMut};
use Color::*;

const fn precompute_ranks() -> SquareIndex<Bitboard> {
    let mut ranks = SquareIndex::new();
    const_for!(y in 0 .. 8 => {
        const_for!(x in 0 .. 8 => {
            let pt = Bitboard::at(Square::xy(x, y));
            const_for!(x1 in 0 .. 8 => {
                ranks.get_mut(pt).0 |= Bitboard::at(Square::xy(x1, y)).0;
            })
        });
    });
    ranks
}

const fn precompute_files() -> SquareIndex<Bitboard> {
    let mut files = SquareIndex::new();
    const_for!(x in 0 .. 8 => {
        const_for!(y in 0 .. 8 => {
            let pt = Bitboard::at(Square::xy(x, y));
            const_for!(y1 in 0 .. 8 => {
                files.get_mut(pt).0 |= Bitboard::at(Square::xy(x, y1)).0;
            })
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
            const_for!(ri in 0 .. rook_file => {
                // Beware the reverse iteration
                let i = rook_file - ri - 1;
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
            const_for!(x1 in 0 .. 8 => {
                let y1 = y0 + (x1 - x0) * delta;
                if y1 >= 0 && y1 < 8 {
                    diagonals.get_mut(pt).0 |= Bitboard::at(Square::xy(x1, y1)).0;
                }
            })
        })
    });
    diagonals
}

pub const RANKS: SquareIndex<Bitboard> = precompute_ranks();
pub const FILES: SquareIndex<Bitboard> = precompute_files();
// BEWARE: this is NOT a SquareIndex, as it's indexed by reduced rank occupancy patterns
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

impl<T> SquareIndex<T> {
    const fn get(&self, bb: Bitboard) -> &T {
        &self.0[bb.to_index()]
    }
    const fn get_mut(&mut self, bb: Bitboard) -> &mut T {
        &mut self.0[bb.to_index()]
    }
}

impl<T> Index<Bitboard> for SquareIndex<T> {
    type Output = T;
    fn index(&self, bb: Bitboard) -> &Self::Output {
        debug_assert_eq!(bb.0.count_ones(), 1);
        self.get(bb)
    }
}

impl<T> IndexMut<Bitboard> for SquareIndex<T> {
    fn index_mut(&mut self, bb: Bitboard) -> &mut T {
        debug_assert_eq!(bb.0.count_ones(), 1);
        self.get_mut(bb)
    }
}

#[rustfmt::skip]
const KNIGHT_JUMPS: [(i32, i32); 8] = [
             (-1,  2), (1,  2),
    (-2,  1),                  (2,  1),

    (-2, -1),                  (2, -1),
             (-1, -2), (1, -2),
];
#[rustfmt::skip]
const KING_JUMPS: [(i32, i32); 8] = [
    (-1,  1), (0,  1), (1,  1),
    (-1,  0),          (1,  0),
    (-1, -1), (0, -1), (1, -1),
];
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

pub const PAWN_SINGLE_MOVES: [SquareIndex<Bitboard>; 2] = precompute_pawn_single_moves();
pub const PAWN_DOUBLE_MOVES: [SquareIndex<Bitboard>; 2] = precompute_pawn_double_moves();
pub const PAWN_ATTACKS: [SquareIndex<Bitboard>; 2] = precompute_pawn_attacks();

#[derive(Copy, Clone)]
pub struct EPInfo {
    pub source_squares: Bitboard,
    pub target_square: Bitboard,
    pub kill_square: Bitboard,
}
pub const PAWN_EP_INFO: [[EPInfo; 9]; 2] = precompute_pawn_ep_info();

const fn precompute_pawn_ep_info() -> [[EPInfo; 9]; 2] {
    let mut infos = [[EPInfo {
        source_squares: Bitboard::empty(),
        target_square: Bitboard::empty(),
        kill_square: Bitboard::empty(),
    }; 9]; 2];

    const_for!(x in 0 .. 8 => {
        const_foreach!((color, home_rank, dy) in [(White, 1, 1), (Black, 6, 1)] => {
            let mut source_squares = Bitboard::empty().0;
            const_foreach!(dx in [-1, 1] => {
                if let Some(source) = Square::xy_checked(x + dx, home_rank) {
                    source_squares |= Bitboard::at(source).0;
                }
            });
            let target_square = Bitboard::at(Square::xy(x, home_rank + dy)).0;
            let kill_square = Bitboard::at(Square::xy(x, home_rank)).0;
            infos[color as usize][x as usize + 1] = EPInfo {
                source_squares: Bitboard(source_squares),
                target_square: Bitboard(target_square),
                kill_square: Bitboard(kill_square)
            };
        });
    });
    infos
}

const fn precompute_pawn_single_moves() -> [SquareIndex<Bitboard>; 2] {
    let mut moves = [SquareIndex::new(); 2];
    const_for!(x in 0 .. 8 => {
        const_for!(y in 0 .. 8 => {
            const_foreach!((color, dy) in [(White, 1), (Black, -1)] => {
                let idx = Square::xy(x, y).index();
                let pat = if let Some(sq) = Square::xy_checked(x, y + dy) {
                    Bitboard::at(sq).0
                } else { 0 };
                moves[color as usize].0[idx as usize] = Bitboard(pat);
            });
        });
    });
    moves
}

const fn precompute_pawn_double_moves() -> [SquareIndex<Bitboard>; 2] {
    let mut moves = [SquareIndex::new(); 2];
    const_for!(x in 0 .. 8 => {
        const_foreach!((color, home_rank, dy) in [(White, 1, 1), (Black, 6, -1)] => {
            let idx = Square::xy(x, home_rank).index();
            let pat = Bitboard::at(Square::xy(x, home_rank + dy + dy)).0;
            moves[color as usize].0[idx as usize] = Bitboard(pat)
        });
    });
    moves
}

const fn precompute_pawn_attacks() -> [SquareIndex<Bitboard>; 2] {
    let mut attacks = [SquareIndex::new(); 2];
    const_for!(x in 0 .. 8 => {
        const_for!(y in 0 .. 8 => {
            let idx = Square::xy(x, y).index();
            const_foreach!((color, dy) in [(White, 1), (Black, -1)] => {
                let mut pat = 0;
                if let Some(square) = Square::xy_checked(x + 1, y + dy) {
                    pat |= Bitboard::at(square).0;
                }
                if let Some(square) = Square::xy_checked(x - 1, y + dy) {
                    pat |= Bitboard::at(square).0;
                }
                attacks[color as usize].0[idx as usize] = Bitboard(pat);
            });
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
            let white_pat = bb << 16 & LINE_AT_Y[1].0;
            rev_moves[White as usize].0[idx as usize] = Bitboard(white_pat);
            let black_pat = bb >> 16 & LINE_AT_Y[6].0;
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
