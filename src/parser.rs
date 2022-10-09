use std::{
    fmt::{self, Display},
    iter::Peekable,
};

use crate::{
    lexer::{Lexer, Token, TK},
    Expr,
};

pub struct Parser<'source> {
    source: &'source str,
    lexer: Peekable<Lexer<'source>>,
}

pub enum SyntaxError {
    UnexpectedEof,
    ExpectedGot(String, TK),
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxError::UnexpectedEof => f.write_str("SyntaxError: Unexpected EOF"),
            SyntaxError::ExpectedGot(expected, got) => {
                write!(f, "SyntaxError: Expected {expected}, got {got}")
            }
        }
    }
}

pub type ParserResult<T> = Result<T, SyntaxError>;

impl<'source> Parser<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            source,
            lexer: Lexer::new(source).peekable(),
        }
    }

    pub fn parse(&mut self) -> ParserResult<Expr> {
        let mut lhs = self.parse()?;
    }

    pub fn parse_basic(&mut self) -> ParserResult<Expr> {
        let mut lhs = match self.peek() {
            TK::Ident => self.parse_ident(),
            TK::Fun => self.parse_abs(),
            TK::LParen => self.parse_grouping(),
            tok => return Err(SyntaxError::ExpectedGot("expression".to_string(), tok)),
        }?;

        if [TK::Ident, TK::Fun, TK::LParen].contains(&self.peek()) {
            lhs = Expr::App(Box::new(lhs), self.parse().map(Box::new)?);
        }

        Ok(lhs)
    }

    fn parse_ident(&mut self) -> ParserResult<Expr> {
        let token = self.expect(TK::Ident)?;
        let text = token.text(self.source);
        Ok(Expr::Var(text.to_string()))
    }

    fn parse_abs(&mut self) -> ParserResult<Expr> {
        self.expect(TK::Fun)?;
        let mut args = Vec::new();
        while self.peek() == TK::Ident {
            let token = self.lexer.next().unwrap();
            let arg = token.text(self.source).to_string();
            args.push(arg);
        }
        self.expect(TK::FatArrow)?;
        let body = self.parse()?;
        let mut abs = Expr::Abs(args.pop().unwrap(), Box::new(body));
        for arg in args.into_iter().rev() {
            abs = Expr::Abs(arg, Box::new(abs));
        }
        Ok(abs)
    }

    fn parse_grouping(&mut self) -> ParserResult<Expr> {
        self.expect(TK::LParen)?;
        let body = self.parse()?;
        self.expect(TK::RParen)?;
        Ok(body)
    }

    fn peek(&mut self) -> TK {
        self.lexer.peek().map(|tok| tok.kind).unwrap_or(TK::Eof)
    }

    fn expect(&mut self, expected: TK) -> ParserResult<Token> {
        let token = self
            .lexer
            .next()
            .ok_or_else(|| SyntaxError::UnexpectedEof)?;
        if token.kind == expected {
            Ok(token)
        } else {
            Err(SyntaxError::ExpectedGot(expected.to_string(), token.kind))
        }
    }
}
