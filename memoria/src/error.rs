use thiserror::Error;
use crate::ExecutionError;
use super::parser::ParseError;

#[derive(Error, Debug, PartialEq)]
pub enum DbError {
    #[error("execution error")]
    ExecutionError(#[from] ExecutionError),

    #[error("parsing error: {0}")]
    ParseError(#[from] ParseError),
}

