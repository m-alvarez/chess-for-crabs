use std::fmt::{Display, Result, Formatter};
use crate::moves::AlgebraicMove;

pub struct MoveLog(pub Vec<AlgebraicMove>);

impl MoveLog {
    pub fn new() -> Self {
        MoveLog(Vec::new())
    }

    pub fn append(&mut self, mv: AlgebraicMove) {
        self.0.push(mv)
    }
}

impl Display for MoveLog {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        let mut move_n = 1;
        for i in 0 .. self.0.len() {
            if i % 2 == 0 {
                write!(fmt, "{}. {}", move_n, self.0[i])?;
            } else {
                write!(fmt, " {}", self.0[i])?;
                move_n += 1;
                write!(fmt, "\n")?;
            }
        }
        Ok(())
    }
}
