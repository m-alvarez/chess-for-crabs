use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, IndexMut};

use crate::bitboard::Bitboard;
use crate::moves::{AlgebraicMove, Move};
use crate::piece::{Color, Piece};
use crate::utils::*;
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

    pub fn apply(&self, player: Color, mv: &Move) -> Board {
        // This will be SIMD eventually
        let mut new = *self;
        for bb in new.bitboards.iter_mut() {
            *bb &= !mv.delete
        }
        new[mv.piece] |= mv.add;
        new[player] |= mv.add;

        new
    }

    pub fn linear_attackers<const N: usize>(&self, player: Color, rays: &[Ray; N]) -> Bitboard {
        let occupancy = (self[player] | self[player.opponent()]).0;
        let mut pattern = 0;
        for i in 0..N {
            pattern |= lsb(rays[i].pos.0 & occupancy);
            pattern |= msb(rays[i].neg.0 & occupancy);
        }
        Bitboard(pattern)
    }

    pub fn move_piece_to(&self, piece: Piece, player: Color, target: Bitboard) -> Bitboard {
        let potential_attackers = match piece {
            Pawn => REV_PAWN_MOVES[player as usize][target],
            King => KING_ATTACKS[target],
            Knight => KNIGHT_ATTACKS[target],
            Bishop => self.linear_attackers(player, &BISHOP_RAYS[target]),
            Rook => self.linear_attackers(player, &ROOK_RAYS[target]),
            Queen => {
                let occupancy = self[player] | self[player.opponent()];
                for i in 0 .. 4 {
                    println!("Ray {i}+\n{:?}", QUEEN_RAYS[target][i].pos & occupancy);
                    println!("Ray {i}-\n{:?}", Bitboard(
                            msb((QUEEN_RAYS[target][i].neg & occupancy).0)
                            ));
                }
                self.linear_attackers(player, &QUEEN_RAYS[target])
            },
        };
        potential_attackers & self[player] & self[piece]
    }

    pub fn attack_piece_to(&self, piece: Piece, player: Color, target: Bitboard) -> Bitboard {
        let potential_attackers = match piece {
            Pawn => REV_PAWN_ATTACKS[player as usize][target],
            King => KING_ATTACKS[target],
            Knight => KNIGHT_ATTACKS[target],
            Bishop => self.linear_attackers(player, &BISHOP_RAYS[target]),
            Rook => self.linear_attackers(player, &ROOK_RAYS[target]),
            Queen => self.linear_attackers(player, &QUEEN_RAYS[target]),
        };
        potential_attackers & self[player] & self[piece]
    }

    pub fn attack_to(&self, player: Color, target: Bitboard) -> Bitboard {
        let mut pattern: Bitboard = Bitboard::empty();
        for piece in Piece::list() {
            let bb = self.attack_piece_to(*piece, player, target);
            pattern |= bb;
        }
        pattern
    }

    pub fn is_pre_legal(&self, player: Color, mv: &AlgebraicMove) -> Option<Move> {
        let dst_bb = Bitboard::at(mv.dst_square);
        if (dst_bb & self[player]).is_populated() {
            println!("Populated");
            return None;
        }
        let attackers = self.move_piece_to(mv.piece, player, dst_bb);
        let attackers = match mv.src_square {
            Some(l) => attackers & Bitboard::line(l),
            None => attackers,
        };
        if attackers.popcnt() != 1 {
            println!("No attackers");
            return None;
        }
        let delete = dst_bb | attackers;

        Some(Move {
            piece: mv.piece,
            delete,
            add: dst_bb,
        })
    }

    pub fn in_check(&self, player: Color) -> bool {
        let king_bb = self[player] & self[King];
        if king_bb.is_empty() {
            return false;
        }
        let attackers = self.attack_to(player.opponent(), king_bb);
        attackers.is_populated()
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
            write!(fmt, "{} ", i + 1)?;
            for j in 0..8 {
                write!(fmt, "{}", chars[i * 8 + j])?;
                write!(fmt, "{}", if j == 7 { "\n" } else { " " })?
            }
        }
        write!(fmt, "  a b c d e f g h")?;
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
