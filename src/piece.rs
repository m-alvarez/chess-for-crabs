use std::fmt::{Display, Formatter, Result};

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}
use Piece::*;

impl Piece {
    pub fn algebraic(self) -> char {
        match self {
            Pawn => 'P',
            Knight => 'N',
            Bishop => 'B',
            Rook => 'R',
            Queen => 'Q',
            King => 'K',
        }
    }
    pub fn from_algebraic(alg: char) -> Option<Self> {
        match alg {
            'P' => Some(Pawn),
            'N' => Some(Knight),
            'B' => Some(Bishop),
            'R' => Some(Rook),
            'Q' => Some(Queen),
            'K' => Some(King),
            _ => None,
        }
    }
    pub const fn list() -> &'static [Piece] {
        &[Pawn, Knight, Bishop, Rook, Queen, King]
    }
    pub fn to_unicode(self, color: Color) -> &'static str {
        let icons = [
            ["♙", "♟"], ["♘", "♞"], ["♗", "♝"], ["♖", "♜"], ["♕", "♛"], ["♔", "♚"]
        ];
        icons[self as usize][color as usize]
    }
}

impl Display for Piece {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        match self {
            Pawn => write!(fmt, "pawn"),
            Knight => write!(fmt, "knight"),
            Bishop => write!(fmt, "bishop"),
            Rook => write!(fmt, "rook"),
            Queen => write!(fmt, "queen"),
            King => write!(fmt, "king"),
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Color {
    Black = 0,
    White = 1,
}
use Color::*;

impl Color {
    pub const fn list() -> &'static [Color] {
        &[Black, White]
    }
    pub const fn opponent(self) -> Color {
        match self {
            White => Black,
            Black => White,
        }
    }
    pub const fn advance_direction(self) -> i32 {
        match self {
            White => 1,
            Black => -1,
        }
    }
}

impl Display for Color {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        match self {
            White => write!(fmt, "white"),
            Black => write!(fmt, "black"),
        }
    }
}
