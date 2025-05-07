use crate::ast::nodes::{Program, Decl, Stmt, Expr};
use crate::lexer::{Token, TokenKind};
use crate::utils::span::Span;
use crate::parser::error::ParseError;
use tracing::instrument;

const OPS: &[(TokenKind, u8, bool)] = &[
    (TokenKind::Assign,       0, true),
    (TokenKind::Equal,        1, false),
    (TokenKind::NotEqual,     1, false),
    (TokenKind::Less,         2, false),
    (TokenKind::LessEqual,    2, false),
    (TokenKind::Greater,      2, false),
    (TokenKind::GreaterEqual, 2, false),
    (TokenKind::Plus,         3, false),
    (TokenKind::Minus,        3, false),
    (TokenKind::Star,         4, false),
    (TokenKind::Slash,        4, false),
];

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    #[instrument(level = "info", skip(self))]
    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut decls = Vec::new();
        while !self.is_eof() {
            decls.push(self.parse_decl()?);
        }
        Ok(Program { decls })
    }

    #[instrument(level = "debug", skip(self))]
    fn parse_decl(&mut self) -> Result<Decl, ParseError> {
        let int_tok = self.expect(TokenKind::Int)?;
        let name_tok = self.expect_identifier("declaration name")?;
        let span_start = int_tok.span.start;
        let span = Span { start: span_start, end: name_tok.span.end };

        if self.consume(TokenKind::LParen) {
            self.expect(TokenKind::RParen)?;
            let body = self.parse_block()?;
            Ok(Decl::Function { name: name_tok.text.clone(), params: Vec::new(), body, span })
        } else {
            self.expect(TokenKind::Assign)?;
            let init = self.parse_expr()?;
            self.expect(TokenKind::Semicolon)?;
            Ok(Decl::Var { name: name_tok.text.clone(), init: Box::new(init), span })
        }
    }

    #[instrument(level = "debug", skip(self))]
    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while !self.consume(TokenKind::RBrace) {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    #[instrument(level = "debug", skip(self))]
    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        if self.consume(TokenKind::LBrace) {
            let mut stmts = Vec::new();
            while !self.consume(TokenKind::RBrace) {
                stmts.push(self.parse_stmt()?);
            }
            return Ok(Stmt::Block(stmts));
        }
        if self.consume(TokenKind::Semicolon) {
            return Ok(Stmt::Empty);
        }
        if self.peek_kind() == Some(TokenKind::Int) {
            let int_tok = self.expect(TokenKind::Int)?;
            let name_tok = self.expect_identifier("local variable name")?;
            self.expect(TokenKind::Assign)?;
            let init = self.parse_expr()?;
            self.expect(TokenKind::Semicolon)?;
            let span = Span { start: int_tok.span.start, end: init.span().end };
            return Ok(Stmt::LocalVar { name: name_tok.text.clone(), init, span });
        }
        if self.consume(TokenKind::Return) {
            let expr = self.parse_expr()?;
            self.expect(TokenKind::Semicolon)?;
            return Ok(Stmt::Return(Box::new(expr)));
        }
        if self.consume(TokenKind::If) {
            self.expect(TokenKind::LParen)?;
            let cond = self.parse_expr()?;
            self.expect(TokenKind::RParen)?;
            let then_blk = self.parse_block()?;
            let else_blk = if self.consume(TokenKind::Else) { Some(self.parse_block()?) } else { None };
            return Ok(Stmt::If { cond: Box::new(cond), then_block: then_blk, else_block: else_blk });
        }
        if self.consume(TokenKind::While) {
            self.expect(TokenKind::LParen)?;
            let cond = self.parse_expr()?;
            self.expect(TokenKind::RParen)?;
            let body = self.parse_block()?;
            return Ok(Stmt::While(Box::new(cond), body));
        }
        if self.consume(TokenKind::For) {
            self.expect(TokenKind::LParen)?;
            let init = if self.peek_kind() != Some(TokenKind::Semicolon) { Some(Box::new(self.parse_expr()?)) } else { None };
            self.expect(TokenKind::Semicolon)?;
            let cond = if self.peek_kind() != Some(TokenKind::Semicolon) { Some(Box::new(self.parse_expr()?)) } else { None };
            self.expect(TokenKind::Semicolon)?;
            let inc = if self.peek_kind() != Some(TokenKind::RParen) { Some(Box::new(self.parse_expr()?)) } else { None };
            self.expect(TokenKind::RParen)?;
            let body = self.parse_block()?;
            return Ok(Stmt::For { init, cond, inc, body });
        }
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Expr(Box::new(expr)))
    }

    #[instrument(level = "debug", skip(self))]
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_precedence(0)
    }

    fn parse_precedence(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_primary()?;
        loop {
            let mut found = None;
            if let Some(tok) = self.peek() {
                for (op, prec, right) in OPS {
                    if *prec < min_prec { continue; }
                    if tok.kind == *op {
                        found = Some((op.clone(), *prec, *right));
                        break;
                    }
                }
            }
            if let Some((op, prec, right_assoc)) = found {
                self.bump();
                let next_min = if right_assoc { prec } else { prec + 1 };
                let rhs = self.parse_precedence(next_min)?;
                let new_span = Span { start: lhs.span().start, end: rhs.span().end };
                lhs = Expr::Binary { op, left: Box::new(lhs), right: Box::new(rhs), span: new_span };
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let tok = self.bump().ok_or(ParseError::Eof)?;
        match tok.kind {
            TokenKind::IntegerLiteral => Ok(Expr::IntLiteral { value: tok.text.parse().unwrap(), span: tok.span }),
            TokenKind::Identifier      => Ok(Expr::Var { name: tok.text.clone(), span: tok.span }),
            TokenKind::LParen          => {
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            other => Err(ParseError::Unexpected(other, tok.span)),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|t| t.kind.clone())
    }

    fn bump(&mut self) -> Option<Token> {
        let t = self.peek().cloned();
        if t.is_some() { self.pos += 1; }
        t
    }

    fn consume(&mut self, kind: TokenKind) -> bool {
        if self.peek_kind() == Some(kind.clone()) { self.bump(); true } else { false }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        self.peek()
            .filter(|t| t.kind == kind)
            .cloned()
            .map(|t| { self.pos += 1; Ok(t) })
            .unwrap_or_else(|| {
                let found = self.peek().map(|t| t.kind.clone()).unwrap_or(TokenKind::Error);
                let span = self.peek().map(|t| t.span).unwrap_or(Span { start: 0, end: 0 });
                Err(ParseError::Expected { expected: format!("{:?}", kind), found, span })
            })
    }

    fn expect_identifier(&mut self, ctx: &str) -> Result<Token, ParseError> {
        let tok = self.peek().cloned().ok_or(ParseError::Eof)?;
        if tok.kind == TokenKind::Identifier { self.pos += 1; Ok(tok) }
        else { Err(ParseError::Expected { expected: format!("identifier ({})", ctx), found: tok.kind, span: tok.span }) }
    }

    fn is_eof(&self) -> bool {
        self.peek_kind().is_none()
    }
}
