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

    pub fn undo_last_move(&mut self) {
        assert!(self.log.ply > 0);
        self.log.ply -= 1;
        self.log.moves.pop();
        let mut new_board = Board::initial();
        for alg in &self.log.moves {
            let mv = new_board.is_legal(alg).unwrap();
            new_board = new_board.apply(&mv)
        }
        self.board = new_board
    }
}
