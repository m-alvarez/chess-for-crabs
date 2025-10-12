#![feature(portable_simd)]
use std::time::SystemTime;

use chess_for_crabs::*;
use eval::MaterialCount;
use game::Game;
use search::IDAB;

fn main() {
    let game = Game::new();
    let mut search = IDAB::new(MaterialCount());
    let start = SystemTime::now();
    let eval = search.evaluate(game.board, game.board.player, 6, 0, 0);
    std::hint::black_box(eval);
    let end = SystemTime::now();
    let delta = end.duration_since(start).unwrap();
    println!(
        "Evaluated {} positions in {} milliseconds",
        search.searched_positions,
        delta.as_millis()
    );
}
