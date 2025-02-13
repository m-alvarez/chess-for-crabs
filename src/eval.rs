use crate::board::Board;
use crate::piece::{Color, Piece};

pub trait Evaluator {
    fn evaluate(&self, board: &Board) -> f64;
}

pub struct MaterialCount();
fn piece_value(p: Piece) -> f64 {
    use Piece::*;
    match p {
        Pawn => 1.,
        Knight => 3.,
        Bishop => 3.,
        Rook => 5.,
        Queen => 9.,
        King => f64::INFINITY,
    }
}
impl Evaluator for MaterialCount {
    fn evaluate(&self, board: &Board) -> f64 {
        let mut count = 0.0;
        for piece in Piece::list() {
            let v = piece_value(*piece);
            count += v * (board[*piece] & board[Color::White]).popcnt() as f64;
            count -= v * (board[*piece] & board[Color::Black]).popcnt() as f64;
        }
        count
    }
}
