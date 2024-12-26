use crate::bitboard::Bitboard;
use crate::coords::{Line, Rel, Square};
use Line::*;

pub const fn d_moves<const N: usize>(square: Square<Rel>, d: [(i32, i32); 8]) -> Bitboard {
    let mut pattern = 0;
    let mut i = 0;
    while i < d.len() {
        let (dx, dy) = d[i];
        if let Some(jump) = Square::<Rel>::xy(square.x as i32 + dx, square.y as i32 + dy) {
            pattern |= Bitboard::at(jump).0;
        }
        i += 1
    }
    Bitboard(pattern)
}

pub const fn knight_moves(square: Square<Rel>) -> Bitboard {
    d_moves::<8>(
        square,
        [
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (-2, -1),
            (-2, 1),
            (2, -1),
            (2, 1),
        ],
    )
}

pub const fn pawn_moves(square: Square<Rel>) -> Bitboard {
    // Remember, we're always white
    let origin = Bitboard::at(square);
    // Shift up by one (or two)
    Bitboard((origin.0 >> 8) | (origin.0 >> 16 & Bitboard::line(AtY(3)).0))
}

pub const fn rev_pawn_moves(square: Square<Rel>) -> Bitboard {
    let destination = Bitboard::at(square);
    Bitboard((destination.0 << 8) | (destination.0 << 16 & Bitboard::line(AtY(1)).0))
}

pub const fn rook_moves(square: Square<Rel>) -> Bitboard {
    let mut pattern = 0;
    let mut i = 0;
    while i < 8 {
        pattern |= Bitboard::at(Square::<Rel>::xy(square.x as i32, i).unwrap()).0;
        i += 1
    }
    i = 0;
    while i < 8 {
        pattern |= Bitboard::at(Square::<Rel>::xy(i, square.y as i32).unwrap()).0;
        i += 1;
    }
    Bitboard(pattern)
}

pub const fn bishop_moves(square: Square<Rel>) -> Bitboard {
    let mut pattern = 0;
    let mut x: i32 = square.x as i32;
    let mut y: i32 = square.y as i32;
    while let Some(sq) = Square::<Rel>::xy(x, y) {
        pattern |= Bitboard::at(sq).0;
        x += 1;
        y += 1;
    }
    x = square.x as i32;
    y = square.y as i32;
    while let Some(sq) = Square::<Rel>::xy(x, y) {
        pattern |= Bitboard::at(sq).0;
        x -= 1;
        y -= 1;
    }
    x = square.x as i32;
    y = square.y as i32;
    while let Some(sq) = Square::<Rel>::xy(x, y) {
        pattern |= Bitboard::at(sq).0;
        x -= 1;
        y += 1;
    }
    x = square.x as i32;
    y = square.y as i32;
    while let Some(sq) = Square::<Rel>::xy(x, y) {
        pattern |= Bitboard::at(sq).0;
        x += 1;
        y -= 1;
    }
    Bitboard(pattern)
}

pub const fn queen_moves(square: Square<Rel>) -> Bitboard {
    Bitboard(bishop_moves(square).0 | rook_moves(square).0)
}

pub const fn king_moves(square: Square<Rel>) -> Bitboard {
    d_moves::<8>(
        square,
        [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ],
    )
}
