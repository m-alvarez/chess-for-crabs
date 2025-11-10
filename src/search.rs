use std::cmp::{max, min};

use crate::{board::Board, eval::Evaluator, moves::Move, piece::Color};

const MAX_MOVES: usize = 28 * (1 + 8) // Max possible queen moves
    + 14 * 2 // Max possible rook moves
    + 14 * 2 // Max possible bishop moves
    + 8 * 2 // Max possible knight moves
    + (9 + 2) // Max possible king moves
;

pub struct IDAB<Ev: Evaluator> {
    pub evaluator: Ev,
    pub searched_positions: i64,
    pub move_buffers: Vec<[Move; MAX_MOVES]>,
}

impl<Ev: Evaluator> IDAB<Ev> {
    pub fn new(evaluator: Ev) -> IDAB<Ev> {
        IDAB {
            evaluator,
            searched_positions: 0,
            move_buffers: Vec::new(),
        }
    }

    pub fn evaluate(
        &mut self,
        pos: Board,
        player: Color,
        depth: u64,
        alpha: i64,
        beta: i64,
    ) -> i64 {
        self.searched_positions += 1;
        if depth == 0 {
            self.evaluator.evaluate(&pos)
        } else {
            let mut best = None;
            let mut moves = Vec::with_capacity(32);
            pos.pre_legal_moves(&mut moves);
            for mv in moves {
                let new_pos = pos.apply(&mv);
                let score = self.evaluate(new_pos, player.opponent(), depth - 1, alpha, beta);
                if let Some(best_score) = best {
                    if player == Color::White {
                        best = Some(max(score, best_score))
                    } else {
                        best = Some(min(score, best_score))
                    }
                } else {
                    best = Some(score)
                }
            }
            match best {
                Some(b) => b,
                None => {
                    debug_assert!(!pos.in_check(player.opponent()));
                    if pos.in_check(player) {
                        match player {
                            Color::Black => i64::max_value(),
                            Color::White => i64::min_value(),
                        }
                    } else {
                        0
                    }
                }
            }
        }
    }
}
