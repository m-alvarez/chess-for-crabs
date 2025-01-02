use std::fmt::{Display, Formatter};

use crate::bitboard::Bitboard;
use crate::coords::{Line, Square};
use crate::piece::Piece;
use crate::piece::Piece::*;

// Beware: in a promotion, `piece` is the type of the promoted piece
#[derive(Copy, Clone)]
pub struct Move {
    pub delete: Bitboard,
    pub piece: Piece,
    pub add: Bitboard,
}

#[derive(Copy, Clone)]
pub struct AlgebraicMove {
    pub piece: Piece,
    pub src_square: Option<Line>,
    pub dst_square: Square,
    pub captures: bool,
    pub check: bool,
    pub checkmate: bool,
}

// Everything always comes down to parsing
#[derive(Copy, Clone)]
enum Token {
    Piece(Piece),
    Rank(i32),
    File(i32),
    Captures,
    Check,
    Checkmate,
}

impl Token {
    fn parse(chr: char) -> Option<Token> {
        if let Some(piece) = Piece::from_algebraic(chr) {
            return Some(Token::Piece(piece));
        }
        match chr {
            'x' => Some(Token::Captures),
            '+' => Some(Token::Check),
            '*' => Some(Token::Checkmate),
            'a'..='h' => Some(Token::File(chr as i32 - 'a' as i32)),
            '1'..='8' => Some(Token::Rank(chr as i32 - '1' as i32)),
            _ => None,
        }
    }
}

impl AlgebraicMove {
    pub fn parse(s: &str) -> Option<AlgebraicMove> {
        use Token::*;
        // Fun fact: algebraic notation is easier to parse back-to-front
        let mut mv = AlgebraicMove {
            piece: Pawn,
            src_square: None,
            dst_square: Square { x: 0, y: 0 },
            captures: false,
            check: false,
            checkmate: false,
        };
        let mut chrs = s.chars().rev().map(Token::parse).peekable();
        // Consume check or checkmate mark
        match (*chrs.peek()?)? {
            Check => {
                chrs.next();
                mv.check = true
            }
            Checkmate => {
                chrs.next();
                mv.checkmate = true
            }
            _ => (),
        }
        // Consume dst
        match (chrs.next()??, chrs.next()??) {
            // remember it's backwards
            (Rank(y), File(x)) => mv.dst_square = Square::xy(x, y).unwrap(),
            _ => return None,
        }
        // Consume capture mark
        match chrs.peek() {
            Some(Some(Captures)) => {
                chrs.next();
                mv.captures = true
            }
            Some(None) => return None,
            _ => (),
        }
        // Consume disambiguate
        match chrs.peek() {
            None => (),
            Some(tok) => match (*tok)? {
                Rank(r) => {
                    chrs.next();
                    mv.src_square = Some(Line::AtY(r as u8))
                }
                File(f) => {
                    chrs.next();
                    mv.src_square = Some(Line::AtX(f as u8))
                }
                _ => (),
            },
        }
        // Consume piece, if any
        match chrs.peek() {
            None => mv.piece = Pawn,
            Some(tok) => match (*tok)? {
                Piece(piece) => {
                    chrs.next();
                    mv.piece = piece
                }
                _ => return None,
            },
        }
        match chrs.peek() {
            None => Some(mv),
            Some(_) => None,
        }
    }
}
impl Display for AlgebraicMove {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match self.piece {
            Pawn => (),
            piece => write!(fmt, "{}", piece.algebraic())?,
        };
        match self.src_square {
            None => (),
            Some(l) => write!(fmt, "{}", l)?,
        };
        if self.captures {
            write!(fmt, "x")?
        };
        write!(fmt, "{}", self.dst_square)?;
        if self.checkmate {
            write!(fmt, "*")?
        } else if self.check {
            write!(fmt, "+")?
        }
        Ok(())
    }
}
