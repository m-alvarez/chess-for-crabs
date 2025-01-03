use std::fmt::{Display, Result, Formatter};
use crate::moves::AlgebraicMove;

pub struct MoveLog {
    pub ply: i64,
    pub moves: Vec<AlgebraicMove>,
}

impl MoveLog {
    pub fn new() -> Self {
        MoveLog { ply: 0, moves: Vec::new() }
    }

    pub fn append(&mut self, mv: AlgebraicMove) {
        self.ply += 1; 
        self.moves.push(mv)
    }
}

impl Display for MoveLog {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        let mut ply = self.ply;
        for mv in self.moves.iter() {
            if ply % 2 == 0 {
                let move_n = 1 + ply / 2;
                write!(fmt, "{}. {}", move_n, mv)?;
            } else {
                write!(fmt, " {}", mv)?;
                ply += 1;
                write!(fmt, "\n")?;
            }
        }
        Ok(())
    }
}
