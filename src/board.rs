use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, IndexMut};

use crate::bitboard::Bitboard;
use crate::moves::{AlgebraicMove, Move};
use crate::piece::{Color, Piece};
use Color::*;
use Piece::*;

use crate::patterns::*;

#[derive(Copy, Clone)]
pub struct Board {
    // First two entries are color boards, then piece boards
    bitboards: [Bitboard; 8],
}

impl Board {
    pub fn initial() -> Board {
        let mut b = Board {
            bitboards: [Bitboard::empty(); 8],
        };
        for color in Color::list() {
            for piece in Piece::list() {
                let piece_bitboard = Bitboard::initial(*color, *piece);
                b[*color] = b[*color] | piece_bitboard;
                b[*piece] = b[*piece] | piece_bitboard;
            }
        }
        b
    }

    pub fn apply(&self, mv: &Move) -> Board {
        // This will be SIMD eventually
        let mut new = *self;
        for bb in new.bitboards.iter_mut() {
            *bb &= !mv.delete
        }
        new[mv.piece] |= mv.add;
        new[White] |= mv.add;

        new
    }

    pub fn flip(&self) -> Board {
        let mut new = *self;
        new[White] = self[Black];
        new[Black] = self[White];
        for bb in new.bitboards.iter_mut() {
            *bb = bb.flip()
        }
        new
    }

    pub fn parse_algebraic(&self, player: Color, mv: &AlgebraicMove) -> Option<Move> {
        let dst_rel = mv.dst_square.to_rel(player);
        let dst_bb = Bitboard::at(dst_rel);
        let sources = match mv.piece {
            Pawn => rev_pawn_moves(dst_rel),
            Rook => rook_moves(dst_rel),
            Bishop => bishop_moves(dst_rel),
            Knight => knight_moves(dst_rel),
            Queen => queen_moves(dst_rel),
            King => king_moves(dst_rel),
        };
        let sources = sources & !dst_bb;
        let sources = sources & self[White];
        let sources = sources & self[mv.piece];
        let sources = match mv.src_square {
            Some(l) => sources & Bitboard::line(l.to_rel(player)),
            None => sources
        };
        if sources.popcnt() != 1 {
            return None;
        }
        let delete = dst_bb | sources;

        Some(Move {
            piece: mv.piece,
            delete,
            add: dst_bb
        })
    }
}

impl Debug for Board {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        writeln!(fmt, "Black bitboard")?;
        writeln!(fmt, "{:?}", self[White])?;
        writeln!(fmt, "White bitboard")?;
        writeln!(fmt, "{:?}", self[Black])?;
        Ok(())
    }
}
impl Display for Board {
    // This is slow, it doesn't have to be fast.
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        let mut chars: [&'static str; 8 * 8] = ["_"; 8 * 8];
        for bit in 0..64 {
            let num: u64 = 1 << 63 - bit;
            let mask = Bitboard(num);
            'find_piece: for color in Color::list() {
                if (self[*color] & mask).is_populated() {
                    for piece in Piece::list() {
                        if (self[*piece] & mask).is_populated() {
                            chars[bit] = piece.to_unicode(*color);
                            break 'find_piece;
                        }
                    }
                }
            }
        }
        for i in (0..8).rev() {
            for j in 0..8 {
                write!(fmt, "{}", chars[i * 8 + j])?;
                write!(fmt, "{}", if j == 7 { "\n" } else { " " })?
            }
        }
        Ok(())
    }
}

impl Index<Piece> for Board {
    type Output = Bitboard;
    fn index(&self, index: Piece) -> &Bitboard {
        &self.bitboards[2 + index as usize]
    }
}
impl IndexMut<Piece> for Board {
    fn index_mut(&mut self, index: Piece) -> &mut Bitboard {
        &mut self.bitboards[2 + index as usize]
    }
}
impl Index<Color> for Board {
    type Output = Bitboard;
    fn index(&self, index: Color) -> &Bitboard {
        &self.bitboards[index as usize]
    }
}
impl IndexMut<Color> for Board {
    fn index_mut(&mut self, index: Color) -> &mut Bitboard {
        &mut self.bitboards[index as usize]
    }
}
