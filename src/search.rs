use std::cmp::{max, min};

use crate::{board::Board, eval::Evaluator, piece::Color};

pub struct IDAB<Ev: Evaluator> {
    pub evaluator: Ev,
    pub searched_positions: i64,
}

impl<Ev: Evaluator> IDAB<Ev> {
    pub fn new(evaluator: Ev) -> IDAB<Ev> {
        IDAB {
            evaluator,
            searched_positions: 0,
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
            pos.for_each_pre_legal_simple_move(&mut |mv| {
                let new_pos = pos.apply_simple(&mv);
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
            });
            pos.for_each_pre_legal_castling_move(&mut |mv| {
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
            });
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
