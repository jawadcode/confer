use std::{
    fmt::{self, Display},
    ops::{Index, Range},
};

use logos::{Logos, SpannedIter};

#[derive(Clone, Copy, Debug, PartialEq, Logos)]
pub enum TK {
    #[regex(r"([A-Za-z]|_)([A-Za-z]|_|\d)*")]
    Ident,
    #[token("fun")]
    Fun,
    #[token("=>")]
    FatArrow,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,

    #[error]
    #[regex(r"[ \t\r\n\f]+", logos::skip)]
    Error,
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Holds the kind of token for parsing, and the span to extract it's text from the source code
pub struct Token {
    /// The type of token
    pub kind: TK,
    /// The position of the `Token` in the source code
    pub span: Span,
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl Token {
    #[inline]
    pub fn text<'input>(&self, input: &'input str) -> &'input str {
        &input[self.span]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Custom span for storing the position of a token or AST node in the source string
pub struct Span {
    /// The start of the span (inclusive)
    pub start: usize,
    /// The end of the span (exclusive)
    pub end: usize,
}

impl Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[Range::<usize>::from(index)]
    }
}

/// A wrapper around `logos::SpannedIter` to map to our custom `Token` type and also to map `None`
/// to `TK::Eof` to allow for easier EOF handling while parsing
pub struct Lexer<'input> {
    /// The length of the input string so the EOF `Token` can have a correct span
    length: usize,
    logos: SpannedIter<'input, TK>,
    eof: bool,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            length: input.len(),
            logos: TK::lexer(input).spanned(),
            eof: false,
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.logos.next() {
            Some((kind, span)) => Some(Token {
                kind,
                span: span.into(),
            }),
            None if self.eof => None,
            None => {
                self.eof = true;
                Some(Token {
                    kind: TK::Eof,
                    span: (self.length..self.length).into(),
                })
            }
        }
    }
}

impl Display for TK {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TK::Ident => f.write_str("identifier"),
            TK::Fun => f.write_str("'fun'"),
            TK::FatArrow => f.write_str("'=>'"),
            TK::LParen => f.write_str("'('"),
            TK::RParen => f.write_str("')'"),
            TK::Error => f.write_str("invalid token"),
            TK::Eof => f.write_str("EOF"),
        }
    }
}
