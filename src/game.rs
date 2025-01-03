use crate::board::Board;
use crate::move_log::MoveLog;
use crate::piece::Color;
use crate::moves::{AlgebraicMove, Move};

use Color::*;

pub struct Game {
    // Castling rights go as in FEN: KQkq
    pub castling: u8,
    // This is a bitmap, not a coordinate!
    pub en_passant: u8,
    pub player: Color,
    pub board: Board,
    pub log: MoveLog,
}

impl Game {
    pub fn new() -> Game {
        Game {
            castling: 0b1111,
            en_passant: 0,
            player: White,
            board: Board::initial(),
            log: MoveLog::new(),
        }
    }

    pub fn can_castle_kingside(&self, player: Color) -> bool {
        self.castling | (0b0010 << (player as usize * 2)) != 0
    }

    pub fn can_castle_queenside(&self, player: Color) -> bool {
        self.castling | (0b0001 << (player as usize * 2)) != 0
    }

    pub fn display_board(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        if self.board.in_check(self.player) {
            writeln!(w, "{} IN CHECK!", self.player)?;
        }
        writeln!(w, "{}", self.board)
    }

    pub fn is_pre_legal(&self, mv: &AlgebraicMove) -> Option<Move> {
        self.board.is_pre_legal(self.player, mv)
    }

    pub fn is_legal(&self, mv: &AlgebraicMove) -> Option<Move> {
        let mv = self.is_pre_legal(mv)?;
        if self.board.apply(self.player, &mv).in_check(self.player) {
            None
        } else {
            Some(mv)
        }
    }

    pub fn make_move(&mut self, alg: &AlgebraicMove, mv: &Move) {
        self.log.append(*alg);
        self.board = self.board.apply(self.player, mv);
        self.player = self.player.opponent();
    }
}
