use std::fmt::{Display, Formatter};
use std::iter::Peekable;

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
    pub disambiguate: (Option<u8>, Option<u8>),
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
            '#' => Some(Token::Checkmate),
            'a'..='h' => Some(Token::File(chr as i32 - 'a' as i32)),
            '1'..='8' => Some(Token::Rank(chr as i32 - '1' as i32)),
            '=' => Some(Token::Equals),
            '-' => Some(Token::Dash),
            'O' | '0' => Some(Token::Oh),
            _ => None,
        }
    }
}

fn try_token<It: Iterator<Item=Option<Token>>>(tokens: &mut Peekable<It>, tok: Token) -> Option<Token> {
    let next = (*tokens.peek()?)?;
    if next == tok {
        tokens.next();
        Some(tok)
    } else {
        None
    }
}

fn try_piece<It: Iterator<Item=Option<Token>>>(tokens: &mut Peekable<It>) -> Option<Piece> {
    match (*tokens.peek()?)? {
        Token::Piece(p) => {
            tokens.next();
            Some(p)
        },
        _ => None
    }
}

fn try_rank<It: Iterator<Item=Option<Token>>>(tokens: &mut Peekable<It>) -> Option<i32> {
    match (*tokens.peek()?)? {
        Token::Rank(r) => {
            tokens.next();
            Some(r)
        },
        _ => None
    }
}

fn try_file<It: Iterator<Item=Option<Token>>>(tokens: &mut Peekable<It>) -> Option<i32> {
    match (*tokens.peek()?)? {
        Token::File(r) => {
            tokens.next();
            Some(r)
        },
        _ => None
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
            disambiguate: (None, None),
            dst_square: Square { x: 0, y: 0 },
            captures: false,
            check: false,
            checkmate: false,
            promotion: None,
        };

        // Consume check or checkmate mark
        if let Some(_) = try_token(&mut chrs, Check) {
            mv.check = true
        } else if let Some(_) = try_token(&mut chrs, Checkmate) {
            mv.checkmate = true
        }
        // Consume dst
        match (try_rank(&mut chrs), try_file(&mut chrs)) {
            // remember it's backwards
            (Some(y), Some(x)) => mv.dst_square = Square::xy(x, y),
            _ => return None
        }
        // Consume capture mark
        if let Some(_) = try_token(&mut chrs, Captures) {
            mv.captures = true
        }
        // Consume disambiguate
        if let Some(r) = try_rank(&mut chrs) {
            if let Some(f) = try_file(&mut chrs) {
                mv.disambiguate = (Some(f as u8), Some(r as u8))
            } else {
                mv.disambiguate = (None, Some(r as u8))
            }
        } else if let Some(f) = try_file(&mut chrs) {
            mv.disambiguate = (Some(f as u8), None)
        }
        // Consume piece, if any
        mv.piece = if let Some(piece) = try_piece(&mut chrs) {
            piece
        } else {
            Pawn
        };
        // No tokens should remain
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
        match mv.disambiguate {
            (None, None) => (),
            (Some(f), Some(r)) => write!(fmt, "{}{}", f, r)?,
            (Some(f), None) => write!(fmt, "{}", f)?,
            (None, Some(r)) => write!(fmt, "{}", r)?,
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
