use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, IndexMut};

use crate::utils::*;
use crate::bitboard::Bitboard;
use crate::coords::{Rel, Square};
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

    pub fn linear_attackers<const N: usize>(&self, rays: &[Ray; N]) -> Bitboard {
        let mut pattern = 0;
        const_for!(i in 0 .. N => {
            pattern |= lsb(rays[i].pos.0 & self[White].0);
            pattern |= msb(rays[i].neg.0 & self[White].0)
        });
        println!("ATTACKERS:\n{:?}", Bitboard(pattern));
        Bitboard(pattern)
    }

    pub fn attackers(&self, piece: Piece, target: Square<Rel>) -> Bitboard {
        let potential_attackers = match piece {
            Pawn => rev_pawn_moves(target) & self[White],
            King => king_moves(target) & self[White],
            Knight => knight_moves(target) & self[White],
            Bishop => self.linear_attackers(&BISHOP_RAYS[target]),
            Rook => self.linear_attackers(&ROOK_RAYS[target]),
            Queen => self.linear_attackers(&QUEEN_RAYS[target]),
        };
        potential_attackers & self[piece]
    }

    // Beware: `White` is always the active player. The `player` variable only
    // indicates how to convert between absolute and relative coordinates
    pub fn validate_algebraic(&self, player: Color, mv: &AlgebraicMove) -> Option<Move> {
        let dst_rel = mv.dst_square.to_rel(player);
        let dst_bb = Bitboard::at(dst_rel);
        if (dst_bb & self[White]).is_populated() {
            return None
        }
        let attackers = self.attackers(mv.piece, dst_rel);
        let attackers = match mv.src_square {
            Some(l) => attackers & Bitboard::line(l.to_rel(player)),
            None => attackers,
        };
        if attackers.popcnt() != 1 {
            return None;
        }
        let delete = dst_bb | attackers;

        Some(Move {
            piece: mv.piece,
            delete,
            add: dst_bb,
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
