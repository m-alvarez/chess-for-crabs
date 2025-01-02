use std::fmt::{Display, Debug, Formatter};

#[derive(Copy, Clone)]
pub struct Square {
    pub x: u8,
    pub y: u8,
}
impl Square {
    pub const fn xy(x: i32, y: i32) -> Option<Square> {
        if x < 0 || y < 0 || x >= 8 || y >= 8 {
            None
        } else {
            Some(Square {
                x: x as u8,
                y: y as u8,
            })
        }
    }
}
impl Display for Square {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}{}", (self.x + 'a' as u8) as char, self.y + 1)
    }
}
impl Debug for Square {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "({}, {})", self.x, self.y)
    }
}

#[derive(Copy, Clone)]
pub enum Line {
    AtX(u8),
    AtY(u8),
}
impl Display for Line {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match self {
            Line::AtY(r) => write!(fmt, "{}", r + 1),
            Line::AtX(f) => write!(fmt, "{}", (f + ('a' as u8)) as char),
        }
    }
}
impl Debug for Line {
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
