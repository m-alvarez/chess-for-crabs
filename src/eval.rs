use crate::board::Board;
use crate::piece::{Color, Piece};

pub trait Evaluator {
    fn evaluate(&mut self, board: &Board) -> i64;
}

pub struct MaterialCount();
fn piece_value(p: Piece) -> i64 {
    use Piece::*;
    match p {
        Pawn => 100,
        Knight => 320,
        Bishop => 330,
        Rook => 500,
        Queen => 900,
        King => u32::max_value() as i64,
    }
}

const PIECE_VALUES: [i64; 8] = [
    0,                       // Black
    0,                       // White
    100,                     // Pawn
    320,                     // Knight
    330,                     // Bishop
    500,                     // Rook
    900,                     // Queen
    u32::max_value() as i64, // King
];

impl Evaluator for MaterialCount {
    fn evaluate(&mut self, board: &Board) -> i64 {
        let mut count = 0;
        for piece in Piece::list() {
            let v = piece_value(*piece);
            let diff = (board[*piece] & board[Color::White]).popcnt()
                - (board[*piece] & board[Color::Black]).popcnt();
            count += v * diff;
        }
        count
        // Vectorized code is slower by a good amount
        /*
        use std::arch::x86_64::*;
        unsafe {
            let pop_vec = _mm512_load_epi64(board.bitboards.0.as_ptr() as *const i64);
            let w_mask = _mm512_set1_epi64(board[Color::White].0 as i64);
            let b_mask = _mm512_set1_epi64(board[Color::Black].0 as i64);
            let w_popcnt = _mm512_popcnt_epi64(_mm512_and_epi64(pop_vec, w_mask));
            let b_popcnt = _mm512_popcnt_epi64(_mm512_and_epi64(pop_vec, b_mask));
            let diff = _mm512_sub_epi64(w_popcnt, b_popcnt);
            let value_vec = _mm512_loadu_epi64(PIECE_VALUES.as_ptr());
            let weighted = _mm512_mullox_epi64(diff, value_vec);
            _mm512_reduce_add_epi64(weighted)
        }
        */
    }
}
