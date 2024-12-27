use std::fmt::{Display, Formatter};

use crate::piece::Color;
use Color::*;

#[derive(Copy, Clone)]
pub struct Abs;
#[derive(Copy, Clone)]
pub struct Rel;

pub trait CoordKind {
    type Type;
    type Opposite: CoordKind<Type = u8>;
}
impl CoordKind for Rel {
    type Type = u8;
    type Opposite = Abs;
}
impl CoordKind for Abs {
    type Type = u8;
    type Opposite = Rel;
}

#[derive(Copy, Clone)]
pub struct Square<Kind: CoordKind> {
    pub x: Kind::Type,
    pub y: Kind::Type,
}
impl<Kind: CoordKind<Type = u8>> Square<Kind> {
    pub const fn xy(x: i32, y: i32) -> Option<Square<Kind>> {
        if x < 0 || y < 0 || x >= 8 || y >= 8 {
            None
        } else {
            Some(Square::<Kind> {
                x: x as u8,
                y: y as u8,
            })
        }
    }

    const fn to_opposite(self, player: Color) -> Square<Kind::Opposite> {
        Square::<Kind::Opposite> {
            x: self.x,
            y: match player {
                White => self.y,
                Black => 7 - self.y,
            },
        }
    }
}
impl Square<Rel> {
    pub const fn to_abs(self, player: Color) -> Square<Abs> {
        self.to_opposite(player)
    }
}
impl Square<Abs> {
    pub const fn to_rel(self, player: Color) -> Square<Rel> {
        self.to_opposite(player)
    }
}
impl Display for Square<Abs> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}{}", (self.x + 'a' as u8) as char, self.y + 1)
    }
}
impl Display for Square<Rel> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "({}, {})", self.x, self.y)
    }
}

#[derive(Copy, Clone)]
pub enum Line<Kind: CoordKind> {
    AtX(Kind::Type),
    AtY(Kind::Type),
}
impl<Kind: CoordKind<Type = u8>> Line<Kind> {
    const fn to_opposite(self, player: Color) -> Line<Kind::Opposite> {
        match self {
            Line::AtX(x) => Line::AtX(x),
            Line::AtY(y) => Line::AtY(match player {
                White => y,
                Black => 7 - y,
            }),
        }
    }
}
impl Line<Abs> {
    pub const fn to_rel(self, player: Color) -> Line<Rel> {
        self.to_opposite(player)
    }
}
impl Line<Rel> {
    pub const fn to_abs(self, player: Color) -> Line<Abs> {
        self.to_opposite(player)
    }
}
impl Display for Line<Abs> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match self {
            Line::AtY(r) => write!(fmt, "{}", r + 1),
            Line::AtX(f) => write!(fmt, "{}", (f + ('a' as u8)) as char),
        }
    }
}
impl Display for Line<Rel> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match self {
            Line::AtY(r) => write!(fmt, "(*, {})", r),
            Line::AtX(f) => write!(fmt, "({}, *)", f),
        }
    }
}

pub const E: (i32, i32) = (1, 0);
pub const NE: (i32, i32) = (1, 1);
pub const N: (i32, i32) = (0, 1);
pub const NW: (i32, i32) = (-1, 1);
pub const W: (i32, i32) = (-1, 0);
pub const SW: (i32, i32) = (-1, -1);
pub const S: (i32, i32) = (0, -1);
pub const SE: (i32, i32) = (1, -1);
