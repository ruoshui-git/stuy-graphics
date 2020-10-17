use std::{
    collections::HashMap,
    fs::File,
    io::{prelude::*, BufReader},
    ops::Deref,
    ops::DerefMut,
    path::Path,
};

use super::{
    ast::{self, Command, Symbol},
    result::EngineError,
    result::EngineResult,
    types::Kind,
};

pub(crate) struct SymTable<T>(HashMap<Symbol, T>);

impl<T> SymTable<T> {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }
}

impl SymTable<Kind> {
    /// Error if sym is not in table or if type doesn't match
    fn check(&self, sym: &Option<Symbol>, kind: Kind) -> EngineResult<()> {
        if let Some(ref s) = sym {
            match self.get(s) {
                Some(expected) => {
                    if *expected != kind {
                        return Err(EngineError::SymbolTypeMismatch {
                            name: s.0.to_owned(),
                            expected: *expected,
                            found: kind,
                        });
                    }
                }
                None => {
                    return Err(EngineError::UndefinedSymbol {
                        name: s.0.to_owned(),
                    });
                }
            }
        }
        Ok(())
    }

    // fn find_symbol<'a>(
    // symbols: &'a HashMap<Symbol, Type>,
    // value: &Symbol,
    // ) -> Result<&'a Type, EngineError> {
    // symbols
    //     .get(value)
    //     .ok_or_else(|| EngineError::UndefinedSymbol {
    //         name: value.0.to_owned(),
    //     })
    // }
}

impl<T> Deref for SymTable<T> {
    type Target = HashMap<Symbol, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for SymTable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Parse file into ast and report errors
pub(crate) fn parse_file<T: AsRef<Path>>(path: T) -> Result<Vec<(usize, Command)>, EngineError> {
    let fin = BufReader::new(File::open(path.as_ref())?);

    let mut cmd_list: Vec<(usize, Command)> = vec![];

    for (lnum, line) in fin.lines().enumerate() {
        let lnum = lnum + 1;
        if let (_, Some(cmd)) = ast::parse_line(&line?).map_err(|nom_err| match nom_err {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(e) | nom::Err::Failure(e) => EngineError::Syntax {
                line: lnum,
                input: e.0.to_owned(),
                kind: e.1,
            },
        })? {
            cmd_list.push((lnum, cmd));
        }
    }

    Ok(cmd_list)
}
