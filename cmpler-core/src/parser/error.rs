use thiserror::Error;
use crate::utils::span::Span;
use crate::lexer::TokenKind;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token `{found:?}` at {span:?}, expected {expected}")]
    Expected {
        expected: String,
        found: TokenKind,
        span: Span,
    },
    #[error("Unexpected token `{0:?}` at {1:?}")]
    Unexpected(TokenKind, Span),
    #[error("Unexpected end of input")]
    Eof,
}
