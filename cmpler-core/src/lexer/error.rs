use thiserror::Error;
use crate::utils::span::Span;

#[derive(Debug, Error)]
#[error("Lex error at {span:?}: unexpected token '{token}'")]
pub struct LexError {
    pub token: String,
    pub span: Span,
}
