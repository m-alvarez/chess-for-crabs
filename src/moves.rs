use std::fmt::{Display, Formatter};
use std::iter::Peekable;

use crate::bitboard::Bitboard;
use crate::coords::Square;
use crate::piece::Piece;
use crate::piece::Piece::*;

// Beware: in a promotion, `piece` is the type of the promoted piece
#[derive(Copy, Clone, Debug)]
pub struct SimpleMove {
    pub delete: Bitboard,
    pub piece: Piece,
    pub add: Bitboard,
}

#[derive(Copy, Clone, Debug)]
pub enum Move {
    Simple(SimpleMove),
    CastleLong,
    CastleShort,
}

#[derive(Copy, Clone)]
pub struct SimpleAlgebraicMove {
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
    Simple(SimpleAlgebraicMove),
    CastleLong { check: bool, checkmate: bool },
    CastleShort { check: bool, checkmate: bool },
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
use Token::*;

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

fn try_token<It: Iterator<Item = Option<Token>>>(
    tokens: &mut Peekable<It>,
    tok: Token,
) -> Option<Token> {
    let next = (*tokens.peek()?)?;
    if next == tok {
        tokens.next();
        Some(tok)
    } else {
        None
    }
}

fn try_piece<It: Iterator<Item = Option<Token>>>(tokens: &mut Peekable<It>) -> Option<Piece> {
    match (*tokens.peek()?)? {
        Token::Piece(p) => {
            tokens.next();
            Some(p)
        }
        _ => None,
    }
}

fn try_rank<It: Iterator<Item = Option<Token>>>(tokens: &mut Peekable<It>) -> Option<i32> {
    match (*tokens.peek()?)? {
        Token::Rank(r) => {
            tokens.next();
            Some(r)
        }
        _ => None,
    }
}

fn try_file<It: Iterator<Item = Option<Token>>>(tokens: &mut Peekable<It>) -> Option<i32> {
    match (*tokens.peek()?)? {
        Token::File(r) => {
            tokens.next();
            Some(r)
        }
        _ => None,
    }
}

fn try_castling<It: Iterator<Item = Option<Token>>>(
    tokens: &mut Peekable<It>,
) -> Option<AlgebraicMove> {
    let mut check = false;
    let mut checkmate = false;
    if let Some(_) = try_token(tokens, Check) {
        check = true
    } else if let Some(_) = try_token(tokens, Checkmate) {
        checkmate = true
    }
    match (*tokens.peek()?)? {
        Oh => {
            // Must be castling - is it short or long?
            tokens.next();
            if tokens.next()?? != Token::Dash || tokens.next()?? != Token::Oh {
                return None;
            }
            match tokens.next() {
                None => Some(AlgebraicMove::CastleShort { check, checkmate }),
                Some(Some(Token::Dash)) => {
                    if tokens.next()?? == Token::Oh {
                        Some(AlgebraicMove::CastleLong { check, checkmate })
                    } else {
                        None
                    }
                }
                Some(_) => None,
            }
        }
        _ => None,
    }
}

impl AlgebraicMove {
    pub fn parse(s: &str) -> Option<AlgebraicMove> {
        use Token::*;
        let mut tokens = s.chars().rev().map(Token::parse).peekable();

        if let Some(castle) = try_castling(&mut tokens) {
            return Some(castle);
        }

        // Fun fact: algebraic notation is easier to parse back-to-front
        let mut mv = SimpleAlgebraicMove {
            piece: Pawn,
            disambiguate: (None, None),
            dst_square: Square { x: 0, y: 0 },
            captures: false,
            check: false,
            checkmate: false,
            promotion: None,
        };

        // Consume check or checkmate mark
        if let Some(_) = try_token(&mut tokens, Check) {
            mv.check = true
        } else if let Some(_) = try_token(&mut tokens, Checkmate) {
            mv.checkmate = true
        }

        if let Some(promote_to) = try_piece(&mut tokens) {
            // Promotion
            mv.promotion = Some(promote_to);
            let _equals = try_token(&mut tokens, Equals)?;
        }

        // Consume dst
        match (try_rank(&mut tokens), try_file(&mut tokens)) {
            // remember it's backwards
            (Some(y), Some(x)) => mv.dst_square = Square::xy(x, y),
            _ => return None,
        }
        // Consume capture mark
        if let Some(_) = try_token(&mut tokens, Captures) {
            mv.captures = true
        }
        // Consume disambiguate
        if let Some(r) = try_rank(&mut tokens) {
            if let Some(f) = try_file(&mut tokens) {
                mv.disambiguate = (Some(f as u8), Some(r as u8))
            } else {
                mv.disambiguate = (None, Some(r as u8))
            }
        } else if let Some(f) = try_file(&mut tokens) {
            mv.disambiguate = (Some(f as u8), None)
        }
        // Consume piece, if any
        mv.piece = if let Some(piece) = try_piece(&mut tokens) {
            if mv.promotion.is_some() {
                // Only pawns can promote!
                return None;
            }
            piece
        } else {
            Pawn
        };
        // No tokens should remain
        match tokens.peek() {
            None => Some(Self::Simple(mv)),
            Some(_) => None,
        }
    }
}
impl Display for AlgebraicMove {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        let mv: &SimpleAlgebraicMove;
        match self {
            AlgebraicMove::CastleLong { check, checkmate } => {
                return write!(
                    fmt,
                    "O-O-O{}{}",
                    if *check { "+" } else { "" },
                    if *checkmate { "#" } else { "" }
                )
            }
            AlgebraicMove::CastleShort { check, checkmate } => {
                return write!(
                    fmt,
                    "O-O{}{}",
                    if *check { "+" } else { "" },
                    if *checkmate { "#" } else { "" }
                )
            }
            AlgebraicMove::Simple(m) => mv = m,
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
