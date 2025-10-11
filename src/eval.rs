use crate::board::Board;
use crate::piece::{Color, Piece};

pub trait Evaluator {
    fn evaluate(&self, board: &Board) -> u64;
}

pub struct MaterialCount();
fn piece_value(p: Piece) -> u64 {
    use Piece::*;
    match p {
        Pawn => 100,
        Knight => 320,
        Bishop => 330,
        Rook => 500,
        Queen => 900,
        King => u32::max_value() as u64,
    }
}
impl Evaluator for MaterialCount {
    fn evaluate(&self, board: &Board) -> u64 {
        let mut count = 0;
        for piece in Piece::list() {
            let v = piece_value(*piece);
            count += v * (board[*piece] & board[Color::White]).popcnt() as u64;
            count -= v * (board[*piece] & board[Color::Black]).popcnt() as u64;
        }
        count
    }
}
