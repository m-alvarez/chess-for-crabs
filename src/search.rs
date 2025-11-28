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
        mut alpha: i64,
        mut beta: i64,
    ) -> i64 {
        self.searched_positions += 1;
        if depth == 0 {
            self.evaluator.evaluate(&pos)
        } else {
            let mut best = match player {
                Color::Black => i64::max_value(),
                Color::White => i64::min_value(),
            };
            let mut moves = Vec::with_capacity(32);
            pos.pre_legal_moves(&mut moves);
            for mv in moves.iter() {
                let new_pos = pos.apply(&mv);
                let score = self.evaluate(new_pos, player.opponent(), depth - 1, alpha, beta);

                if player == Color::White {
                    best = max(score, best);
                    if best >= beta {
                        break;
                    };
                    alpha = max(alpha, score)
                } else {
                    best = min(score, best);
                    if best <= alpha {
                        break;
                    };
                    beta = min(beta, score)
                }
            }
            best
        }
    }

    /* Just for debugging purposes */
    pub fn evaluate_naive(&mut self, pos: Board, player: Color, depth: u64) -> i64 {
        self.searched_positions += 1;
        if depth == 0 {
            self.evaluator.evaluate(&pos)
        } else {
            let mut best = match player {
                Color::Black => i64::max_value(),
                Color::White => i64::min_value(),
            };
            let mut moves = Vec::with_capacity(32);
            pos.pre_legal_moves(&mut moves);
            for mv in moves.iter() {
                let new_pos = pos.apply(&mv);
                let score = self.evaluate_naive(new_pos, player.opponent(), depth - 1);

                if player == Color::White {
                    best = max(score, best);
                } else {
                    best = min(score, best);
                }
            }
            let eval_ab = self.evaluate(pos, player, depth, i64::min_value(), i64::max_value());
            if eval_ab != best {
                std::process::exit(-1);
            }
            best
        }
    }
}
