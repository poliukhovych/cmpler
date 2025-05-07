pub mod error;
pub mod symbol_table;
pub mod analyzer;

pub use analyzer::SemanticAnalyzer;
pub use error::SemanticError;
