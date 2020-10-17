use std::io;

use thiserror::Error;

use super::types::Kind;

/// Result to wrap EngineError
pub type EngineResult<T> = Result<T, EngineError>;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("syntax error on line {line}: input: {input}, type: {kind:?}")]
    Syntax {
        line: usize,
        input: String,
        kind: nom::error::ErrorKind,
    },
    #[error("symbol {name} is undefined")]
    UndefinedSymbol { name: String },
    #[error("symbol {name} defined as {expected} but here used as {found}")]
    SymbolTypeMismatch {
        name: String,
        expected: Kind,
        found: Kind,
    },
    #[error("runtime error on line {line}: {source}")]
    Runtime { line: usize, source: RuntimeError }, // Syntax(#[from] nom::Err),
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
}

// #[derive(Error, Debug)]
// pub enum SyntaxError {
//     #[error("bad symbol name")]
//     BadSymbol,

//     #[error("unknown parser error")]
//     Unparseable,
// }

// impl<I> ParseError<I> for SyntaxError {
//     fn from_error_kind(input: I, kind: ErrorKind) -> Self {
//         SyntaxError::Unparseable
//     }

//     fn append(input: I, kind: ErrorKind, other: Self) -> Self {
//         other
//     }
// }
