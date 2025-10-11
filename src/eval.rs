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
impl Evaluator for MaterialCount {
    fn evaluate(&mut self, board: &Board) -> i64 {
        let mut count = 0;
        for piece in Piece::list() {
            let v = piece_value(*piece);
            count += v * (board[*piece] & board[Color::White]).popcnt() as i64;
            count -= v * (board[*piece] & board[Color::Black]).popcnt() as i64;
        }
        count
    }
}
