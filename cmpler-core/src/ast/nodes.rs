use crate::utils::span::Span;
use crate::lexer::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub decls: Vec<Decl>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        span: Span,
    },
    Var {
        name: String,
        init: Box<Expr>,
        span: Span,
    },
}

impl Decl {
    pub fn span(&self) -> Span {
        match self {
            Decl::Function { span, .. } | Decl::Var { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Return(Box<Expr>),
    If {
        cond: Box<Expr>,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    While(Box<Expr>, Vec<Stmt>),
    For {
        init: Option<Box<Expr>>,
        cond: Option<Box<Expr>>,
        inc: Option<Box<Expr>>,
        body: Vec<Stmt>,
    },
    Block(Vec<Stmt>),
    Expr(Box<Expr>),

    LocalVar {
        name: String,
        init: Expr,
        span: Span,
    },

    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLiteral {
        value: i64,
        span: Span,
    },
    Var {
        name: String,
        span: Span,
    },
    Binary {
        op: TokenKind,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::IntLiteral { span, .. }
            | Expr::Var { span, .. }
            | Expr::Binary { span, .. } => *span,
        }
    }
}
