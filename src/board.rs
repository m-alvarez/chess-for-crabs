use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, IndexMut};

use crate::bitboard::Bitboard;
use crate::coords::Line;
use crate::moves::{AlgebraicMove, Move, SimpleAlgebraicMove, SimpleMove};
use crate::piece::{Color, Piece};
use Color::*;
use Piece::*;

use crate::patterns::*;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Board {
    // First two entries are color boards, then piece boards
    pub bitboards: [Bitboard; 8],
    pub player: Color,
    pub half_moves: u8,
    pub castling_rights: u8,
    pub en_passant: u8,
}

// Mostly used to debug incorrect "illegal move" messages
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum IllegalMove {
    OccupiedSquare,
    Unreachable,
    CaptureMismatch,
    InCheck,
    CastlingThroughPiece,
    CastlingThroughCheck,
    NoCastlingPermissions,
    Ambiguous,
    IllegalPromotion,
}

use IllegalMove::*;
impl IllegalMove {
    pub fn as_str(self) -> &'static str {
        match self {
            OccupiedSquare => "Destination square is occupied",
            Unreachable => "Piece cannot reach that square",
            CaptureMismatch => "Capture mismatch",
            InCheck => "Move would put you in check",
            CastlingThroughPiece => "Piece in the way of castling",
            CastlingThroughCheck => "Castling through a check",
            NoCastlingPermissions => "Castling without rights",
            Ambiguous => "Ambiguous",
            IllegalPromotion => "Illegal promotion (incorrect piece or rank)",
        }
    }
}

pub struct CastleInfo {
    delete: Bitboard,
    add_rook: Bitboard,
    add_king: Bitboard,
}
const LONG_CASTLE_INFO: [CastleInfo; 2] = [
    CastleInfo {
        delete: Bitboard::from_bytes([0b10001000, 0, 0, 0, 0, 0, 0, 0]),
        add_rook: Bitboard::from_bytes([0b00010000, 0, 0, 0, 0, 0, 0, 0]),
        add_king: Bitboard::from_bytes([0b00100000, 0, 0, 0, 0, 0, 0, 0]),
    },
    CastleInfo {
        delete: Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b10001000]),
        add_rook: Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b00010000]),
        add_king: Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b00100000]),
    },
];
const SHORT_CASTLE_INFO: [CastleInfo; 2] = [
    CastleInfo {
        delete: Bitboard::from_bytes([0b00001001, 0, 0, 0, 0, 0, 0, 0]),
        add_rook: Bitboard::from_bytes([0b00000100, 0, 0, 0, 0, 0, 0, 0]),
        add_king: Bitboard::from_bytes([0b00000010, 0, 0, 0, 0, 0, 0, 0]),
    },
    CastleInfo {
        delete: Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b00001001]),
        add_rook: Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b00000100]),
        add_king: Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b00000010]),
    },
];

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

    pub fn occupancy(&self) -> Bitboard {
        self[Black] | self[White]
    }

    pub fn apply_simple(&self, mv: &SimpleMove) -> Board {
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

    pub fn apply_castle(&self, info: &CastleInfo) -> Board {
        let mut new = *self;
        new[Rook] = (new[Rook] & !info.delete) | info.add_rook;
        new[King] = (new[King] & !info.delete) | info.add_king;
        new[self.player] = (new[self.player] & !info.delete) | info.add_rook | info.add_king;
        new.player = self.player.opponent();
        new
    }

    pub fn apply(&self, mv: &Move) -> Board {
        match mv {
            Move::Simple(s) => self.apply_simple(s),
            Move::CastleLong => self.apply_castle(&LONG_CASTLE_INFO[self.player as usize]),
            Move::CastleShort => self.apply_castle(&SHORT_CASTLE_INFO[self.player as usize]),
        }
    }

    // TODO: pawn moves with blocking can be done with rays and collision testing
    pub fn pawn_move(&self, player: Color, target: Bitboard) -> Bitboard {
        let potential_attackers = REV_PAWN_MOVES[player as usize][target];
        let occupied = (self[Black] | self[White]).0;
        let potential_attackers = potential_attackers
            | (if player == Black {
                Bitboard(!(occupied >> 8))
            } else {
                Bitboard(!(occupied << 8))
            } & REV_PAWN_DBL_MOVES[player as usize][target]);
        potential_attackers & self[player] & self[Pawn]
    }

    pub fn piece_capture(&self, player: Color, piece: Piece, target: Bitboard) -> Bitboard {
        let potential_attackers = match piece {
            Pawn => {
                REV_PAWN_ATTACKS[player as usize][target]
                    | REV_PAWN_EP_ATTACKS[player as usize]
                        [(8 - self.en_passant.leading_zeros()) as usize][target]
            }
            King => KING_ATTACKS[target],
            Knight => KNIGHT_ATTACKS[target],
            Bishop => self.bishop_reach(target),
            Rook => self.rook_reach(target),
            Queen => self.queen_reach(target),
        };
        potential_attackers & self[player] & self[piece]
    }

    pub fn capture_to(&self, player: Color, target: Bitboard) -> Bitboard {
        let mut pattern: Bitboard = Bitboard::empty();
        for piece in Piece::list() {
            let bb = self.piece_capture(player, *piece, target);
            pattern |= bb;
        }
        pattern
    }

    pub fn is_pre_legal(&self, mv: &SimpleAlgebraicMove) -> Result<Move, IllegalMove> {
        let piece;
        if let Some(promote_to) = mv.promotion {
            if mv.piece != Pawn {
                return Err(IllegalMove::IllegalPromotion);
            }
            if match self.player {
                White => mv.dst_square.y != 7,
                Black => mv.dst_square.y != 0,
            } {
                return Err(IllegalMove::IllegalPromotion);
            }
            piece = promote_to
        } else {
            piece = mv.piece
        }
        // This doesn't have to be fast, since the machine never generates AlgebraicMove
        // objects
        let dst_bb = Bitboard::at(mv.dst_square);
        if (dst_bb & self[self.player]).is_populated() {
            return Err(IllegalMove::OccupiedSquare);
        }
        let mut attackers = if mv.piece != Pawn || mv.captures {
            self.piece_capture(self.player, mv.piece, dst_bb)
        } else {
            self.pawn_move(self.player, dst_bb)
        };
        if let Some(file) = mv.disambiguate.0 {
            attackers &= Bitboard::line(Line::AtX(file))
        }
        if let Some(rank) = mv.disambiguate.1 {
            attackers &= Bitboard::line(Line::AtY(rank))
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
            return Err(IllegalMove::CaptureMismatch);
        }

        if attackers.popcnt() == 0 {
            return Err(IllegalMove::Unreachable);
        }
        if attackers.popcnt() == 1 {
            // Happy case, no need to iterate over every bit. The performance difference
            // is meaningless, though
            let tentative_move = SimpleMove {
                piece,
                delete: captures | attackers,
                add: dst_bb,
            };
            if self.apply_simple(&tentative_move).in_check(self.player) {
                Err(IllegalMove::InCheck)
            } else {
                Ok(Move::Simple(tentative_move))
            }
        } else {
            // I don't like that this is necessary, but the game dumps I get from
            // lichess don't disambiguate pieces when one of them is pinned. Again,
            // this is not performance-critical, so this doesn't matter
            for i in 0..63 {
                let pat = Bitboard(1 << i) & attackers;
                if pat.is_populated() {
                    let tentative_move = SimpleMove {
                        piece,
                        delete: captures | pat,
                        add: dst_bb,
                    };
                    if self.apply_simple(&tentative_move).in_check(self.player) {
                        attackers &= !pat
                    }
                }
            }
            if attackers.popcnt() == 0 {
                Err(IllegalMove::InCheck)
            } else if attackers.popcnt() > 1 {
                Err(IllegalMove::Ambiguous)
            } else {
                Ok(Move::Simple(SimpleMove {
                    piece,
                    delete: captures | attackers,
                    add: dst_bb,
                }))
            }
        }
    }

    pub fn is_legal(&self, mv: &AlgebraicMove) -> Result<Move, IllegalMove> {
        let mv = match mv {
            AlgebraicMove::Simple(mv) => self.is_pre_legal(mv),
            AlgebraicMove::CastleLong { .. } => self.castle_long(),
            AlgebraicMove::CastleShort { .. } => self.castle_short(),
        }?;
        if self.apply(&mv).in_check(self.player) {
            Err(IllegalMove::InCheck)
        } else {
            Ok(mv)
        }
    }

    fn check_castle_move(&self, path: Bitboard, mid_square: Bitboard) -> Result<(), IllegalMove> {
        if (path & self.occupancy()).is_populated() {
            return Err(IllegalMove::CastlingThroughPiece);
        }
        if self
            .capture_to(self.player.opponent(), mid_square)
            .is_empty()
        {
            Ok(())
        } else {
            Err(IllegalMove::CastlingThroughCheck)
        }
    }

    fn castle_long(&self) -> Result<Move, IllegalMove> {
        if !self.long_castling_allowed(self.player) {
            return Err(IllegalMove::NoCastlingPermissions);
        }

        let () = self.check_castle_move(
            LONG_CASTLE_PATH[self.player as usize],
            LONG_CASTLE_MID_SQUARE[self.player as usize],
        )?;
        Ok(Move::CastleLong)
    }

    fn castle_short(&self) -> Result<Move, IllegalMove> {
        if !self.short_castling_allowed(self.player) {
            return Err(IllegalMove::NoCastlingPermissions);
        }

        let () = self.check_castle_move(
            SHORT_CASTLE_PATH[self.player as usize],
            SHORT_CASTLE_MID_SQUARE[self.player as usize],
        )?;
        Ok(Move::CastleShort)
    }

    pub fn in_check(&self, color: Color) -> bool {
        let king_bb = self[color] & self[King];
        if king_bb.is_empty() {
            return false;
        }
        let attackers = self.capture_to(color.opponent(), king_bb);
        attackers.is_populated()
    }

    pub fn short_castling_allowed(&self, color: Color) -> bool {
        // Can be done jump-free if need be
        0 != self.castling_rights
            & match color {
                White => 0b1000,
                Black => 0b0010,
            }
    }

    pub fn long_castling_allowed(&self, color: Color) -> bool {
        0 != self.castling_rights
            & match color {
                White => 0b0100,
                Black => 0b0001,
            }
    }

    pub fn display(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        if self.in_check(self.player) {
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
