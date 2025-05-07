use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("Failed to initialize LLVM: {0}")]
    LlvmInitError(String),
    #[error("Unsupported operation: {0}")]
    UnsupportedOp(String),
}
