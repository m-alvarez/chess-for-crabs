use crate::bitboard::Bitboard;
use crate::coords::Square;
use crate::board::Board;
use crate::move_log::MoveLog;
use crate::piece::Color::*;
use crate::piece::Piece::*;
use crate::piece::{Color, Piece};
use crate::Game;

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

    println!("The board is [{s}]");

    let mut i = 0;
    for chr in s.chars() {
        println!("{i}: {chr}");
        if i % 9 == 8 {
            if chr != '/' {
                println!("Expected slash, got {chr}");
                return None;
            }
            i += 1
        } else if let Some(skip) = chr.to_digit(10) {
            if skip > 0 && skip <= 8 {
                i += skip;
            } else {
                println!("Bad skip");
                return None
            }
        } else {
            let x = i % 9;
            let y = 7 - i / 9;
            let piece = char_piece(chr)?;
            let color = char_color(chr)?;
            let bb = Bitboard::at(Square::xy(x as i32, y as i32)?);
            board[piece] |= bb;
            board[color] |= bb;
            i += 1
        }
    }

    Some(board)
}

pub fn parse(s: &str) -> Option<Game> {
    let mut segments = s.split_whitespace();
    let board_s = segments.next()?;
    let turn_s = segments.next()?;
    let _castling_s = segments.next()?;
    let _ep_s = segments.next()?;
    let hm_s = segments.next()?;
    let fm_s = segments.next()?;

    let board = read_fen_board(board_s)?;
    let player = match turn_s {
        "w" => White,
        "b" => Black,
        _ => return None,
    };
    let log = MoveLog {
        ply: str::parse::<i64>(hm_s).ok()?,
        moves: Vec::new(),
    };
    Some(Game { board, player, log })
}
