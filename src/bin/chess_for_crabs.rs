#![feature(portable_simd)]
use std::io::{BufRead, Write};

use chess_for_crabs::*;
use fen;
use args::{Args, print_usage};
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

fn play_from(mut game: Game) {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut buffer = String::new();

    loop {
        println!("{}", game.board.fen());
        writeln!(&mut out, "").unwrap();
        game.board.display(&mut out).unwrap();
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
            let mv = match game.board.is_legal(&alg) {
                Ok(mv) => mv,
                Err(err) => {
                    return Err(err.as_str())
                }
            };
            Ok((alg, mv))
        })
        .unwrap();
        game.make_move(&alg, &mv);
        println!("{}", game.log);
    }
}

fn play() {
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

    let game = match option {
        1 => Game::new(),
        2 => {
            println!("Input FEN");
            try_read(&mut buffer, |s| fen::parse(s).ok_or("Invalid FEN")).unwrap()
        }
        _ => unreachable!(),
    };
    play_from(game)
}

fn debug_to(target: Game) {
    let mut buffer = String::new();
    let mut game = Game::new();
    loop {
        if game.log.ply == target.log.ply {
            assert_eq!(game.board, target.board);
            return;
        }
        let (alg, mv) = try_read(&mut buffer, |s| {
            let alg = AlgebraicMove::parse(s).unwrap();
            let mv = game.board.is_legal(&alg).unwrap();
            Ok((alg, mv))
        })
        .unwrap();
        game.make_move(&alg, &mv);
    }
}

fn main() {
    match Args::parse() {
        Some(Args::Interactive) => play(),
        Some(Args::FEN(fen)) => match fen::parse(&fen) {
            Some(game) => play_from(game),
            None => println!("Invalid FEN"),
        },
        None => print_usage()
    }
}
