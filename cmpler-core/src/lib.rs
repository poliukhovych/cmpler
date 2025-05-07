pub mod config;
pub mod driver;
pub mod error;
pub mod lexer;
pub mod logger;
pub mod parser;
pub mod ast;
pub mod semantic;
pub mod ir;
pub mod codegen;
pub mod utils;

pub use lexer::{Token, TokenKind, lex};
pub use error::CompilerError;
pub use config::Config;
pub use driver::compile;
