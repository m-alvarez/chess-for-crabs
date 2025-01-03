#![feature(portable_simd)]
use std::io::{BufRead, Write};

#[macro_use]
mod utils;
mod bitboard;
mod board;
mod coords;
mod fen;
mod game;
mod move_log;
mod moves;
mod patterns;
mod piece;

use game::Game;
use moves::AlgebraicMove;

use piece::Piece::*;

fn try_read<T, F: Fn(&str) -> Result<T, &str>>(
    buffer: &mut String,
    parse: F,
) -> std::io::Result<T> {
    let stdin = std::io::stdin();
    let mut in_handle = stdin.lock();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    loop {
        buffer.clear();
        print!("> ");
        out.flush().unwrap();
        in_handle.read_line(buffer)?;
        match parse(buffer.trim()) {
            Ok(result) => return Ok(result),
            Err(msg) => println!("{}", msg),
        }
    }
}

fn main() {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut buffer = String::new();

    println!("(1) New game");
    println!("(2) Import FEN");
    let option = try_read(&mut buffer, |s| {
        if s == "1" {
            Ok(1)
        } else if s == "2" {
            Ok(2)
        } else {
            Err("Invalid option")
        }
    })
    .unwrap();

    let mut game = match option {
        1 => Game::new(),
        2 => {
            println!("Input FEN");
            try_read(&mut buffer, |s| fen::parse(s).ok_or("Invalid FEN")).unwrap()
        },
        _ => unreachable!(),
    };
    loop {
        fen::serialize(&mut out, &game).unwrap();
        writeln!(&mut out, "").unwrap();
        game.display_board(&mut out).unwrap();
        for color in piece::Color::list() {
            if (game.board[King] & game.board[color.opponent()]).is_empty() {
                println!("{color} wins!");
                return;
            }
        }
        let (alg, mv) = try_read(&mut buffer, |s| {
            let alg = if let Some(alg) = AlgebraicMove::parse(s) {
                alg
            } else {
                return Err("I cannot parse that");
            };
            let mv = if let Some(mv) = game.is_pre_legal(&alg) {
                mv
            } else {
                return Err("Illegal move");
            };
            Ok((alg, mv))
        })
        .unwrap();
        game.make_move(&alg, &mv);
        println!("{}", game.log);
    }
}
