use std::fmt::{Debug, Formatter};
use std::ops::{BitAnd, BitOr, BitXor, Not, Sub};
use std::ops::{BitAndAssign, BitOrAssign, BitXorAssign};

use std::arch::x86_64::_popcnt64;

use crate::coords::{Line, Square};
use crate::piece::{Color, Piece};
use Color::*;
use Piece::*;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Bitboard(pub u64);

impl Debug for Bitboard {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        for y in (0..8).rev() {
            write!(fmt, "{} ", y + 1)?;
            for x in 0..8 {
                let square = Square::xy(x, y);
                let intersection = *self & Bitboard::at(square);
                write!(fmt, "{}", intersection.popcnt())?;
                write!(fmt, "{}", if x == 7 { "\n" } else { " " })?;
            }
        }
        write!(fmt, "  a b c d e f g h")?;
        Ok(())
    }
}

impl Default for Bitboard {
    fn default() -> Self {
        Bitboard(0)
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;
    fn bitand(self, other: Bitboard) -> Self::Output {
        Bitboard(self.0 & other.0)
    }
}
impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, other: Bitboard) {
        self.0 = self.0 & other.0
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;
    fn bitor(self, other: Bitboard) -> Self::Output {
        Bitboard(self.0 | other.0)
    }
}
impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, other: Bitboard) {
        self.0 = self.0 | other.0
    }
}

impl BitXor for Bitboard {
    type Output = Bitboard;
    fn bitxor(self, other: Bitboard) -> Self::Output {
        Bitboard(self.0 ^ other.0)
    }
}
impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, other: Bitboard) {
        self.0 = self.0 ^ other.0
    }
}

impl Not for Bitboard {
    type Output = Bitboard;
    fn not(self) -> Bitboard {
        Bitboard(!self.0)
    }
}

impl Sub for Bitboard {
    type Output = Bitboard;
    fn sub(self, other: Bitboard) -> Self::Output {
        Bitboard(self.0.wrapping_sub(other.0))
    }
}

static LINE_AT_Y: [Bitboard; 8] = [
    Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b11111111]),
    Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0b11111111, 0]),
    Bitboard::from_bytes([0, 0, 0, 0, 0, 0b11111111, 0, 0]),
    Bitboard::from_bytes([0, 0, 0, 0, 0b11111111, 0, 0, 0]),
    Bitboard::from_bytes([0, 0, 0, 0b11111111, 0, 0, 0, 0]),
    Bitboard::from_bytes([0, 0, 0b11111111, 0, 0, 0, 0, 0]),
    Bitboard::from_bytes([0, 0b11111111, 0, 0, 0, 0, 0, 0]),
    Bitboard::from_bytes([0b11111111, 0, 0, 0, 0, 0, 0, 0]),
];

static LINE_AT_X: [Bitboard; 8] = [
    Bitboard::from_bytes([0b10000000; 8]),
    Bitboard::from_bytes([0b01000000; 8]),
    Bitboard::from_bytes([0b00100000; 8]),
    Bitboard::from_bytes([0b00010000; 8]),
    Bitboard::from_bytes([0b00001000; 8]),
    Bitboard::from_bytes([0b00000100; 8]),
    Bitboard::from_bytes([0b00000010; 8]),
    Bitboard::from_bytes([0b00000001; 8]),
];

impl Bitboard {
    pub const fn from_bytes(bytes: [u8; 8]) -> Bitboard {
        Bitboard(u64::from_le_bytes(bytes))
    }

    pub const fn to_index(self) -> usize {
        self.0.ilog2() as usize
    }

    pub const fn rank(self) -> usize {
        7 - (self.to_index() >> 3)
    }

    pub const fn file(self) -> usize {
        7 - (self.to_index() & 7)
    }

    pub const fn coords(self) -> (usize, usize) {
        (self.file(), self.rank())
    }

    pub const fn initial_white(kind: Piece) -> Bitboard {
        Bitboard::from_bytes(match kind {
            Pawn => [0, 0, 0, 0, 0, 0, 0b11111111, 0],
            Knight => [0, 0, 0, 0, 0, 0, 0, 0b01000010],
            Bishop => [0, 0, 0, 0, 0, 0, 0, 0b00100100],
            Rook => [0, 0, 0, 0, 0, 0, 0, 0b10000001],
            Queen => [0, 0, 0, 0, 0, 0, 0, 0b00010000],
            King => [0, 0, 0, 0, 0, 0, 0, 0b00001000],
        })
    }

    pub const fn initial(color: Color, kind: Piece) -> Bitboard {
        match color {
            White => Bitboard::initial_white(kind),
            Black => Bitboard::initial_white(kind).flip(),
        }
    }

    pub const fn flip(self) -> Bitboard {
        // Hehe
        Bitboard(self.0.to_be().to_le())
    }

    pub const fn empty() -> Self {
        Bitboard(0)
    }

    pub const fn is_populated(self) -> bool {
        self.0 != 0
    }
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn popcnt(self) -> i32 {
        unsafe { _popcnt64(self.0 as i64) }
    }

    pub const fn at(point: Square) -> Bitboard {
        Bitboard(1 << (63 - point.x - point.y * 8))
    }

    pub const fn line(l: Line) -> Bitboard {
        match l {
            Line::AtX(x) => LINE_AT_X[x as usize],
            Line::AtY(y) => LINE_AT_Y[y as usize],
        }
    }

    pub const fn union<const N: usize>(boards: [Bitboard; N]) -> Bitboard {
        let mut pattern: u64 = 0;
        const_for!(i in 0 .. N => {
            pattern = pattern | boards[i].0;
        });
        Bitboard(pattern)
    }
    pub const fn intersection<const N: usize>(boards: [Bitboard; N]) -> Bitboard {
        let mut pattern: u64 = !0;
        const_for!(i in 0 .. N => {
            pattern = pattern & boards[i].0;
        });
        Bitboard(pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bb_rank_file() {
        for x in 0 .. 8 {
            for y in 0 .. 8 {
                let bb = Bitboard::at(Square::xy(x, y));
                assert!(bb.file() == x as usize);
                assert!(bb.rank() == y as usize);
            }
        }
    }
}
