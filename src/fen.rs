use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::coords::Square;
use crate::game::Game;
use crate::move_log::MoveLog;
use crate::piece::Color::*;
use crate::piece::Piece::*;
use crate::piece::{Color, Piece};
use std::fmt::{Display, Formatter, Result};

fn char_piece(c: char) -> Option<Piece> {
    Some(match c.to_ascii_lowercase() {
        'p' => Pawn,
        'n' => Knight,
        'b' => Bishop,
        'r' => Rook,
        'q' => Queen,
        'k' => King,
        _ => return None,
    })
}

fn char_color(c: char) -> Option<Color> {
    if c.is_ascii_lowercase() {
        Some(Black)
    } else if c.is_ascii_uppercase() {
        Some(White)
    } else {
        None
    }
}

fn read_fen_board(s: &str) -> Option<Board> {
    let mut board = Board::empty();

    let mut i = 0;
    for chr in s.chars() {
        if i % 9 == 8 {
            if chr != '/' {
                return None;
            }
            i += 1
        } else if let Some(skip) = chr.to_digit(10) {
            if skip > 0 && skip <= 8 {
                i += skip;
            } else {
                return None;
            }
        } else {
            let x = i % 9;
            let y = 7 - i / 9;
            let piece = char_piece(chr)?;
            let color = char_color(chr)?;
            let bb = Bitboard::at(Square::xy_checked(x as i32, y as i32)?);
            board[piece] |= bb;
            board[color] |= bb;
            i += 1
        }
    }

    Some(board)
}

fn read_castling_rights(s: &str) -> Option<u8> {
    let mut castling_rights = 0b0000;
    for chr in s.chars() {
        let mask = match chr {
            'K' => 0b1000,
            'Q' => 0b0100,
            'k' => 0b0010,
            'q' => 0b0001,
            '-' => return Some(0),
            _ => return None,
        };
        if mask & castling_rights != 0 {
            return None;
        }
        castling_rights |= mask
    }
    Some(castling_rights)
}

fn read_en_passant(s: &str) -> Option<u8> {
    if s == "-" {
        return Some(0);
    }
    let mut chars = s.chars();
    let file = chars.next()?;
    let rank = chars.next()?;
    let sq = Square::algebraic(file, rank.to_digit(10)? as u8)?;
    Some(1 << sq.x)
}

pub fn parse(s: &str) -> Option<Game> {
    let mut segments = s.split_whitespace();
    let mut board = read_fen_board(segments.next()?)?;
    board.player = match segments.next()? {
        "w" => White,
        "b" => Black,
        _ => return None,
    };
    board.castling_rights = read_castling_rights(segments.next()?)?;
    board.en_passant = read_en_passant(segments.next()?)?;
    board.half_moves = str::parse::<u8>(segments.next()?).ok()?;
    let fm = str::parse::<i64>(segments.next()?).ok()?;

    let log = MoveLog {
        ply: fm * 2 + if board.player == Black { 1 } else { 0 },
        moves: Vec::new(),
    };
    Some(Game { board, log })
}

fn write_piece(out: &mut Formatter, color: Color, piece: Piece) -> Result {
    let piece = match piece {
        Pawn => 'P',
        Knight => 'N',
        Bishop => 'B',
        Rook => 'R',
        Queen => 'Q',
        King => 'K',
    };
    let piece = if color == White {
        piece
    } else {
        piece.to_ascii_lowercase()
    };
    write!(out, "{}", piece)
}

fn serialize_board(out: &mut Formatter, b: &Board) -> Result {
    for y in (0..8).rev() {
        let mut empty_spaces = 0;
        for x in 0..8 {
            let square = Bitboard::at(Square::xy(x, y));
            let color = if (square & b[White]).is_populated() {
                White
            } else if (square & b[Black]).is_populated() {
                Black
            } else {
                empty_spaces += 1;
                continue;
            };
            let piece = *Piece::list()
                .iter()
                .find(|pc| (square & b[**pc]).is_populated())
                .unwrap();
            if empty_spaces > 0 {
                write!(out, "{}", empty_spaces)?;
                empty_spaces = 0
            }
            write_piece(out, color, piece)?
        }
        if empty_spaces > 0 {
            write!(out, "{}", empty_spaces)?;
        }
        if y > 0 {
            write!(out, "/")?
        }
    }
    Ok(())
}

fn serialize_player(out: &mut Formatter, c: Color) -> Result {
    write!(
        out,
        "{}",
        match c {
            White => 'w',
            Black => 'b',
        }
    )
}

fn serialize_castling_rights(out: &mut Formatter, board: &Board) -> Result {
    if board.kingside_castling_allowed(White) {
        write_piece(out, White, King)?;
    }
    if board.queenside_castling_allowed(White) {
        write_piece(out, White, Queen)?;
    }
    if board.kingside_castling_allowed(Black) {
        write_piece(out, Black, King)?;
    }
    if board.queenside_castling_allowed(Black) {
        write_piece(out, Black, Queen)?;
    }
    Ok(())
}

fn serialize_en_passant(out: &mut Formatter, board: &Board) -> Result {
    if board.en_passant == 0 {
        write!(out, "-")
    } else {
        let file = (board.en_passant.leading_zeros() + 'a' as u32) as u8 as char;
        let rank = match board.player {
            White => 6,
            Black => 3,
        };
        write!(out, "{}{}", file, rank)
    }
}

// This is mostly just an adapter to dump the FEN of a board
pub struct FEN<'a>(&'a Board);

impl<'a> Display for FEN<'a> {
    fn fmt(&self, out: &mut Formatter) -> Result {
        serialize_board(out, self.0)?;
        write!(out, " ")?;
        serialize_player(out, self.0.player)?;
        write!(out, " ")?;
        serialize_castling_rights(out, self.0)?;
        write!(out, " ")?;
        serialize_en_passant(out, self.0)?;
        write!(out, " {} 0", self.0.half_moves) // TODO: serialize half-moves
    }
}

impl Board {
    pub fn fen<'a>(&'a self) -> FEN<'a> {
        FEN(self)
    }
}
