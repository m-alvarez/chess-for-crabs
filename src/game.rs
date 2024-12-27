use crate::board::Board;
use crate::move_log::MoveLog;
use crate::piece::Color;
use crate::moves::{AlgebraicMove, Move};

use Color::*;

pub struct Game {
    pub player: Color,
    pub board: Board,
    pub log: MoveLog,
}

impl Game {
    pub fn new() -> Game {
        Game {
            player: White,
            board: Board::initial(),
            log: MoveLog::new(),
        }
    }

    pub fn display_board(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        let board = if self.player == White {
            self.board
        } else {
            self.board.flip()
        };
        write!(w, "{}", board)
    }

    pub fn validate_algebraic(&self, mv: &AlgebraicMove) -> Option<Move> {
        self.board.validate_algebraic(self.player, mv)
    }

    pub fn make_move(&mut self, alg: &AlgebraicMove, mv: &Move) {
        self.log.append(*alg);
        self.board = self.board.apply(mv).flip();
        self.player = self.player.opponent();
    }
}
