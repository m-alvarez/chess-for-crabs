use crate::board::Board;
use crate::move_log::MoveLog;
use crate::moves::{AlgebraicMove, Move};

pub struct Game {
    pub board: Board,
    pub log: MoveLog,
}

impl Game {
    pub fn new() -> Game {
        Game {
            board: Board::initial(),
            log: MoveLog::new(),
        }
    }

    pub fn make_move(&mut self, alg: &AlgebraicMove, mv: &Move) {
        self.log.append(*alg);
        self.board = self.board.apply(mv)
    }
}
