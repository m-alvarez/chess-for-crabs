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
    pub bitboards: [Bitboard; 8],
    pub player: Color,
    pub half_moves: u8,
    pub castling_rights: u8,
    pub en_passant: u8,
}

impl Board {
    pub fn initial() -> Board {
        let mut b = Board {
            bitboards: [Bitboard::empty(); 8],
            player: White,
            half_moves: 0,
            castling_rights: 0b1111,
            en_passant: 0,
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

    pub fn empty() -> Board {
        Board {
            bitboards: [Bitboard::empty(); 8],
            player: White,
            half_moves: 0,
            castling_rights: 0,
            en_passant: 0,
        }
    }

    pub fn apply(&self, mv: &Move) -> Board {
        // This will be SIMD eventually
        let mut new = *self;
        for bb in new.bitboards.iter_mut() {
            *bb &= !mv.delete
        }
        new[mv.piece] |= mv.add;
        new[self.player] |= mv.add;
        new.player = self.player.opponent();
        let new_pawn = new[Pawn] & !self[Pawn];
        new.en_passant = match self.player {
            Black => (((new_pawn.0 >> 16) & mv.delete.0) >> 8) as u8,
            White => (((new_pawn.0 << 16) & mv.delete.0) >> 48) as u8,
        };
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
            Queen => self.linear_attackers(player, &QUEEN_RAYS[target]),
        };
        potential_attackers & self[player] & self[piece]
    }

    pub fn capture_piece_to(&self, piece: Piece, player: Color, target: Bitboard) -> Bitboard {
        let potential_attackers = match piece {
            Pawn => {
                REV_PAWN_ATTACKS[player as usize][target]
                    | REV_PAWN_EP_ATTACKS[player as usize]
                        [(8 - self.en_passant.leading_zeros()) as usize][target]
            }
            King => KING_ATTACKS[target],
            Knight => KNIGHT_ATTACKS[target],
            Bishop => self.linear_attackers(player, &BISHOP_RAYS[target]),
            Rook => self.linear_attackers(player, &ROOK_RAYS[target]),
            Queen => self.linear_attackers(player, &QUEEN_RAYS[target]),
        };
        potential_attackers & self[player] & self[piece]
    }

    pub fn capture_to(&self, player: Color, target: Bitboard) -> Bitboard {
        let mut pattern: Bitboard = Bitboard::empty();
        for piece in Piece::list() {
            let bb = self.capture_piece_to(*piece, player, target);
            pattern |= bb;
        }
        pattern
    }

    pub fn is_pre_legal(&self, mv: &AlgebraicMove) -> Option<Move> {
        // This doesn't have to be fast, since the machine never generates AlgebraicMove
        // objects
        let dst_bb = Bitboard::at(mv.dst_square);
        if (dst_bb & self[self.player]).is_populated() {
            return None;
        }
        let attackers = if mv.captures {
            self.capture_piece_to(mv.piece, self.player, dst_bb)
        } else {
            self.move_piece_to(mv.piece, self.player, dst_bb)
        };
        let attackers = match mv.src_square {
            Some(l) => attackers & Bitboard::line(l),
            None => attackers,
        };
        if attackers.popcnt() != 1 {
            return None;
        }

        let captures = dst_bb & self[self.player.opponent()];
        let captures = if mv.piece == Pawn && captures.is_empty() {
            // En passant capture
            (match self.player {
                White => Bitboard(dst_bb.0 << 8),
                Black => Bitboard(dst_bb.0 >> 8),
            }) & self[self.player.opponent()]
        } else {
            captures
        };

        if captures.is_populated() != mv.captures {
            return None;
        }

        Some(Move {
            piece: mv.piece,
            delete: captures | attackers,
            add: dst_bb,
        })
    }

    pub fn is_legal(&self, mv: &AlgebraicMove) -> Option<Move> {
        let mv = self.is_pre_legal(mv)?;
        if self.apply(&mv).in_check() {
            None
        } else {
            Some(mv)
        }
    }

    pub fn in_check(&self) -> bool {
        let king_bb = self[self.player] & self[King];
        if king_bb.is_empty() {
            return false;
        }
        let attackers = self.capture_to(self.player.opponent(), king_bb);
        attackers.is_populated()
    }

    pub fn kingside_castling_allowed(&self, color: Color) -> bool {
        // Can be done jump-free if need be
        0 != self.castling_rights
            & match color {
                White => 0b1000,
                Black => 0b0010,
            }
    }

    pub fn queenside_castling_allowed(&self, color: Color) -> bool {
        0 != self.castling_rights
            & match color {
                White => 0b0100,
                Black => 0b0001,
            }
    }

    pub fn display(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        if self.in_check() {
            writeln!(w, "Player {} is in check!", self.player)?;
        }
        writeln!(w, "{}", self)
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

const fn empty_chess_board() -> [&'static str; 8 * 8] {
    let mut chars: [&'static str; 8 * 8] = ["_"; 8 * 8];
    const_for!(i in 0 .. 8 * 8 => {
        /*
        chars[i] = if (i % 8 + i / 8) % 2 == 1 {
            "■"
        } else {
            "□"
        }
        */
        chars[i] = "_"
    });
    chars
}

impl Display for Board {
    // This is slow, it doesn't have to be fast.
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        let mut chars: [&'static str; 8 * 8] = empty_chess_board();
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
