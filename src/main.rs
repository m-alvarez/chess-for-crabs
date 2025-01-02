#![feature(portable_simd)]
use std::io::{BufRead, Write};

#[macro_use]
mod utils;
mod board;
mod moves;
mod bitboard;
mod piece;
mod patterns;
mod move_log;
mod game;
mod coords;

use moves::AlgebraicMove;
use game::Game;

use piece::Piece::*;

fn main() {
    let stdin = std::io::stdin();
    let mut in_handle = stdin.lock();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let mut buffer = String::new();
    let mut game = Game::new();
    loop {
        game.display_board(&mut out).unwrap();
        for color in piece::Color::list() {
            if (game.board[King] & game.board[color.opponent()]).is_empty() {
                println!("{color} wins!");
                return
            }
        }
        buffer.clear();
        print!("> ");
        out.flush().unwrap();
        in_handle.read_line(&mut buffer).unwrap();
        let mv = if let Some(mv) = AlgebraicMove::parse(buffer.trim()) {
            mv
        } else {
            println!("Not a valid move: {}", buffer.trim());
            continue;
        };
        let real_mv = if let Some(mv) = game.is_pre_legal(&mv) {
            mv
        } else {
            println!("Move {} cannot be played", mv);
            continue;
        };
        game.make_move(&mv, &real_mv);
        println!("{}", game.log);
    }
}
