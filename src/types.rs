use std::fmt::*;
use std::ops::*;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    White = 1,
}

impl Display for Color {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        match self {
            Color::Black => write!(fmt, "Black"),
            Color::White => write!(fmt, "White")
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord)]
#[repr(u8)]
pub enum File {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

impl Display for File {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        write!(fmt, "{}", ('A' as u8 + *self as u8) as char)
    }
}

impl std::iter::Step for File {
    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let start_i = start as usize;
        if count <= start_i {
            Some(unsafe { std::mem::transmute( (start_i - count) as u8 ) } )
        } else {
            None
        }
    }
    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let start_i = start as usize;
        if count < 8 && start_i + count  < 8 {
            Some(unsafe { std::mem::transmute( (start_i + count) as u8 ) } )
        } else {
            None
        }
    }
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        let s_i = *start as u8;
        let e_i = *end as u8;
        if s_i > e_i {
            (0, None)
        } else {
            let d = (e_i - s_i) as usize;
            (d, Some(d))
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord)]
#[repr(u8)]
pub enum Rank {
    R1 = 0,
    R2 = 1,
    R3 = 2,
    R4 = 3,
    R5 = 4,
    R6 = 5,
    R7 = 6,
    R8 = 7,
}

impl Display for Rank {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        write!(fmt, "{}", 1 + *self as u8)
    }
}

impl std::iter::Step for Rank {
    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let start_i = start as usize;
        if count <= start_i {
            Some(unsafe { std::mem::transmute( (start_i - count) as u8 ) } )
        } else {
            None
        }
    }
    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let start_i = start as usize;
        if count < 8 && start_i + count  < 8 {
            Some(unsafe { std::mem::transmute( (start_i + count) as u8 ) } )
        } else {
            None
        }
    }
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        let s_i = *start as u8;
        let e_i = *end as u8;
        if s_i > e_i {
            (0, None)
        } else {
            let d = (e_i - s_i) as usize;
            (d, Some(d))
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}
#[allow(dead_code)]
const PIECE_OK: () = assert!(std::mem::size_of::<Piece>() == 1);

/*
 * A1 is 0
 * H8 is 63
 */
pub struct Square(u8);
impl Square {
    #[inline(always)]
    pub const fn of_index(idx: u8) -> Self {
        debug_assert!(idx < 64);
        Square(idx & 0b111111)
    }
    #[inline(always)]
    pub const fn of_rf(rank: Rank, file: File) -> Self {
        Square((rank as u8 * 8) + file as u8)
    }

    #[inline(always)]
    pub const fn rank(self) -> Rank {
        debug_assert!(self.0 < 64);
        unsafe { std::mem::transmute::<u8, Rank>(self.0 / 8) }
    }

    #[inline(always)]
    pub const fn file(self) -> File {
        debug_assert!(self.0 < 64);
        unsafe { std::mem::transmute::<u8, File>(self.0 % 8) }
    }
}

/*
 * BITBOARD REPRESENTATION
 *
 * 8 56 57 58 59 60 61 62 63
 * 7 ...
 * 6 ...
 * 5 ...
 * 4 ...
 * 3 ...
 * 2 ...
 * 1  0  1  2  3  4  5  6  7
 *    A  B  C  D  E  F  G  H
 */
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    #[inline(always)]
    pub const fn square(pt: Square) -> Self {
        Bitboard(1 << pt.0)
    }
    #[inline(always)]
    pub const fn at(rank: Rank, file: File) -> Self {
        Self::square(Square::of_rf(rank, file))
    }
    #[inline(always)]
    pub const fn rank(r: Rank) -> Self {
        Bitboard(0b11111111 << (r as u8 * 8))
    }
    #[inline(always)]
    pub const fn file(f: File) -> Self {
        Bitboard(0x0101010101010101 << f as u8)
    }

    #[inline(always)]
    pub const fn shift_up(self, i: u8) -> Self {
        debug_assert!(i < 8);
        Bitboard(self.0 << 8 * i)
    }
    #[inline(always)]
    pub const fn shift_down(self, i: u8) -> Self {
        debug_assert!(i < 8);
        Bitboard(self.0 >> 8 * i)
    }
    /* Beware the reversing of the direction */
    #[inline(always)]
    pub const fn shift_left(self, i : u8) -> Self {
        debug_assert!(i < 8);
        Bitboard(self.0 >> i)
    }
    #[inline(always)]
    pub const fn shift_right(self, i : u8) -> Self {
        debug_assert!(i < 8);
        Bitboard(self.0 << i)
    }

    #[inline(always)]
    pub const fn is_populated(self) -> bool {
        self.0.count_ones() != 0
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

impl Debug for Bitboard {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        for rank in Rank::R1 ..= Rank::R8 {
            write!(fmt, "{} ", rank)?;
            for file in File::A ..= File::H {
                if (*self & Bitboard::at(rank, file)).is_populated() {
                    write!(fmt, "X")?
                } else {
                    write!(fmt, "_")?
                }
                write!(fmt, "{}", if file == File::H { "\n" } else { " " })?
            }
        }
        write!(fmt, "  A B C D E F G H")
    }
}

/*
 * Similar layout to Stockfish Move
 * Bits 0-5: destination
 * Bits 6-11: origin
 * Bits 12-14: created (promoted or moved) piece
 * Bit 15: free for now
 */
pub struct Move(u16);

impl Move {
    #[inline(always)]
    pub const fn make(piece: Piece, source: Square, destination: Square) -> Move {
        Move(destination.0 as u16 | (source.0 << 6) as u16 | ((piece as u16) << 12))
    }

    #[inline(always)]
    pub const fn destination(self) -> Square {
        Square((self.0 & 0b111111) as u8)
    }
    #[inline(always)]
    pub const fn source(self) -> Square {
        Square(((self.0 >> 6) & 0b111111) as u8)
    }
    #[inline(always)]
    pub const fn piece(self) -> Piece {
        let piece_field = ((self.0 >> 12) & 0b111) as u8;
        unsafe { std::mem::transmute(piece_field) }
    }
}
