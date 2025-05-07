use thiserror::Error;
use std::io;
use crate::lexer::LexError;
use crate::parser::ParseError;
use crate::semantic::SemanticError;
use crate::config::ConfigError;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Lexical error: {0}")]
    Lex(#[from] LexError),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Semantic error: {0}")]
    Semantic(#[from] SemanticError),

    #[error("Code generation error: {0}")]
    Codegen(String),
}
