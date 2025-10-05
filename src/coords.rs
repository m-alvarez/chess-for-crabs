use std::fmt::{Debug, Display, Formatter};

#[derive(Copy, Clone)]
pub struct Square {
    pub x: u8,
    pub y: u8,
}
impl Square {
    pub const fn xy_checked(x: i32, y: i32) -> Option<Square> {
        if x < 0 || y < 0 || x >= 8 || y >= 8 {
            None
        } else {
            Some(Square {
                x: x as u8,
                y: y as u8,
            })
        }
    }

    pub const fn xy(x: i32, y: i32) -> Square {
        Square {
            x: x as u8,
            y: y as u8,
        }
    }

    pub const fn algebraic(file: char, rank: u8) -> Option<Square> {
        let file = file as i32 - 'a' as i32;
        Square::xy_checked(file, rank as i32)
    }

    pub const fn index(self) -> u64 {
        (63 - self.x - self.y * 8) as u64
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

pub const E: (i32, i32) = (1, 0);
pub const NE: (i32, i32) = (1, 1);
pub const N: (i32, i32) = (0, 1);
pub const NW: (i32, i32) = (-1, 1);
pub const W: (i32, i32) = (-1, 0);
pub const SW: (i32, i32) = (-1, -1);
pub const S: (i32, i32) = (0, -1);
pub const SE: (i32, i32) = (1, -1);
