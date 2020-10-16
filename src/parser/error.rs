use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("syntax error on input {input}: {kind:?}")]
    Syntax {
        input: String,
        kind: nom::error::ErrorKind
    },
    #[error("symbol {name} is undefined")]
    UndefinedSymbol {
        name: String
    },
    #[error("symbol {name} defined as {expected} but here used as {found}")]
    SymbolTypeMismatch {
        name: String,
        expected: String,
        found: String,
    }
    // Syntax(#[from] nom::Err),
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
