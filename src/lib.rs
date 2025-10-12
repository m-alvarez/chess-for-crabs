#![feature(stdarch_x86_avx512)]
#![feature(pointer_is_aligned_to)]
#[macro_use]
pub mod utils;
pub mod args;
pub mod bitboard;
pub mod board;
pub mod eval;
pub mod fen;
pub mod game;
pub mod gen;
pub mod move_log;
pub mod moves;
pub mod patterns;
pub mod piece;
pub mod search;
