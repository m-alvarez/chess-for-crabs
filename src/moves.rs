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
    // I hate castling
    // In the future, maybe it's worth to just have 6 bitboards and do SIMD
    pub king_add: Bitboard,
}

pub const QUEENSIDE_CASTLE_MOVE: [Move; 2] = [
    Move {
        delete: Bitboard::from_bytes([0b10001000, 0, 0, 0, 0, 0, 0, 0]),
        piece: Rook,
        add: Bitboard::from_bytes([0b00010000, 0, 0, 0, 0, 0, 0, 0]),
        king_add: Bitboard::from_bytes([0b00100000, 0, 0, 0, 0, 0, 0, 0]),
    },
    Move {
        delete: Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b10001000]),
        piece: Rook,
        add: Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b00010000]),
        king_add: Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b00100000]),
    },
];
pub const KINGSIDE_CASTLE_MOVE: [Move; 2] = [
    Move {
        delete: Bitboard::from_bytes([0b00001001, 0, 0, 0, 0, 0, 0, 0]),
        piece: Rook,
        add: Bitboard::from_bytes([0b00000100, 0, 0, 0, 0, 0, 0, 0]),
        king_add: Bitboard::from_bytes([0b00000010, 0, 0, 0, 0, 0, 0, 0]),
    },
    Move {
        delete: Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b00001001]),
        piece: Rook,
        add: Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b00000100]),
        king_add: Bitboard::from_bytes([0, 0, 0, 0, 0, 0, 0, 0b00000010]),
    },
];


#[derive(Copy, Clone)]
pub struct SimpleMove {
    pub piece: Piece,
    pub src_square: Option<Line>,
    pub dst_square: Square,
    pub captures: bool,
    pub check: bool,
    pub checkmate: bool,
    pub promotion: Option<Piece>,
}
#[derive(Copy, Clone)]
pub enum AlgebraicMove {
    Simple(SimpleMove),
    CastleLong,
    CastleShort,
}

// Everything always comes down to parsing
#[derive(Copy, Clone, PartialEq, Eq)]
enum Token {
    Piece(Piece),
    Rank(i32),
    File(i32),
    Captures,
    Check,
    Checkmate,
    Dash,
    Oh,
    Equals,
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
            '=' => Some(Token::Equals),
            '-' => Some(Token::Dash),
            'O' | '0' => Some(Token::Oh),
            _ => None,
        }
    }
}

impl AlgebraicMove {
    pub fn parse(s: &str) -> Option<AlgebraicMove> {
        use Token::*;
        let mut chrs = s.chars().rev().map(Token::parse).peekable();
        match (*chrs.peek()?)? {
            Oh => {
                // Must be castling - is it short or long?
                chrs.next();
                if chrs.next()?? != Token::Dash || chrs.next()?? != Token::Oh {
                    return None;
                }
                match chrs.next() {
                    None => return Some(AlgebraicMove::CastleShort),
                    Some(Some(Token::Dash)) => {
                        return if chrs.next()?? == Token::Oh {
                            Some(AlgebraicMove::CastleLong)
                        } else {
                            None
                        }
                    }
                    Some(_) => return None,
                }
            }
            _ => (),
        }

        // Fun fact: algebraic notation is easier to parse back-to-front
        let mut mv = SimpleMove {
            piece: Pawn,
            src_square: None,
            dst_square: Square { x: 0, y: 0 },
            captures: false,
            check: false,
            checkmate: false,
            promotion: None,
        };

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
            (Rank(y), File(x)) => mv.dst_square = Square::xy(x, y),
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
            None => Some(Self::Simple(mv)),
            Some(_) => None,
        }
    }
}
impl Display for AlgebraicMove {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        let mv: &SimpleMove;
        match self {
            AlgebraicMove::CastleLong => return write!(fmt, "O-O-O"),
            AlgebraicMove::CastleShort => return write!(fmt, "O-O"),
            AlgebraicMove::Simple(m) => mv = m
        };
        
        match mv.piece {
            Pawn => (),
            piece => write!(fmt, "{}", piece.algebraic())?,
        };
        match mv.src_square {
            None => (),
            Some(l) => write!(fmt, "{}", l)?,
        };
        if mv.captures {
            write!(fmt, "x")?
        };
        write!(fmt, "{}", mv.dst_square)?;
        if let Some(promote_to) = mv.promotion {
            write!(fmt, "={}", promote_to.algebraic())?
        }
        if mv.checkmate {
            write!(fmt, "*")?
        } else if mv.check {
            write!(fmt, "+")?
        }
        Ok(())
    }
}
