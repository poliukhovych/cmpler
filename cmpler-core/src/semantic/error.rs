use thiserror::Error;
use crate::utils::span::Span;

#[derive(Debug, Error)]
pub enum SemanticError {
    #[error("Duplicate symbol '{0}' at {1:?}")]
    DuplicateSymbol(String, Span),

    #[error("Undefined variable '{0}' at {1:?}")]
    UndefinedVariable(String, Span),

    #[error("Type mismatch: expected {expected:?}, found {found:?} at {span:?}")]
    TypeMismatch { expected: String, found: String, span: Span },
}
