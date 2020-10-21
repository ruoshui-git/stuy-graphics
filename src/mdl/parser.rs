use std::{
    collections::HashMap,
    fs::File,
    io::{self, prelude::*, BufReader},
    ops::{Deref, DerefMut},
    path::Path,
};

use crate::light::LightProps;

use super::{
    ast::{self, Command, Symbol, VaryInfo},
    result::{EngineError, EngineResult, RuntimeError},
    types::Kind,
    utils::warn_unimpl,
    ExecContext,
};

pub struct SymTable<T>(HashMap<Symbol, T>);

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
                Some(got) => {
                    if *got != kind {
                        return Err(EngineError::SymbolTypeMismatch {
                            name: s.0.to_owned(),
                            expected: *got,
                            found: kind,
                        });
                    }
                }
                None => {
                    return Err(EngineError::SymbolNotFound {
                        name: s.0.to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

impl<T> SymTable<T> {
    pub fn find(&self, op_symbol: &Option<Symbol>) -> EngineResult<Option<&T>> {
        match op_symbol {
            Some(symbol) => match self.get(symbol) {
                Some(t) => Ok(Some(t)),
                None => Err(EngineError::SymbolNotFound {
                    name: symbol.0.to_owned(),
                }),
            },
            None => Ok(None),
        }
    }
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
pub(crate) fn parse_file<T: AsRef<Path>>(path: T) -> EngineResult<ExecContext> {
    let fin = BufReader::new(File::open(path.as_ref())?);

    let mut cmd_list: Vec<(usize, Command)> = vec![];

    let mut frames: Option<u32> = None;
    let mut basename: Option<String> = None;
    let mut vary_list: Vec<(usize, VaryInfo)> = vec![];

    let mut constants_table: SymTable<LightProps> = SymTable::new();

    let script = fin.lines().collect::<io::Result<Vec<String>>>()?;

    // This is the first pass
    // Deals with `frames`, `basename`, `vary` and `constants` commands
    for (lnum, line) in script.iter().enumerate() {
        let lnum = lnum + 1;
        if let (_, Some(cmd)) = ast::parse_line(line).map_err(|nom_err| match nom_err {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(e) | nom::Err::Failure(e) => EngineError::Syntax {
                line: lnum,
                input: e.0.to_owned(),
                kind: e.1,
            },
        })? {
            if let Command::AnimateCmd(animate_cmd) = cmd {
                match animate_cmd {
                    ast::Animate::Basename(name) => basename = Some(name),
                    // ast::Animate::SetKnob { name, value } => {}
                    // ast::Animate::SetAllKnobs(_) => {}
                    // ast::Animate::Tween {
                    //     start_frame,
                    //     end_frame,
                    //     knoblist0,
                    //     knoblist1,
                    // } => {}
                    ast::Animate::Frames(f) => {
                        if frames.is_some() {
                            return Err(EngineError::Runtime {
                                line: lnum,
                                source: RuntimeError::MultipleFrameNumber,
                            });
                        } else {
                            frames = Some(f)
                        }
                    }
                    ast::Animate::Vary(vary_info) => {
                        if vary_info.start_frame >= vary_info.end_frame {
                            return Err(EngineError::Runtime {
                                line: lnum,
                                source: RuntimeError::Semantics(
                                    "start_frame of vary must be < end_frame",
                                ),
                            });
                        }
                        vary_list.push((lnum, vary_info));
                    }
                    // ast::Animate::SaveKnobList(_) => {}
                    _ => cmd_list.push((lnum, Command::AnimateCmd(animate_cmd))),
                }
            } else if let Command::LightingCmd(lighting_cmd) = cmd {
                match lighting_cmd {
                    ast::Lighting::Light {
                        name: _,
                        color: _,
                        location: _,
                    } => warn_unimpl("light", lnum),
                    ast::Lighting::Ambient(_) => warn_unimpl("ambient", lnum),
                    ast::Lighting::Constants { name, value } => {
                        constants_table.insert(name, value.into());
                    }
                    ast::Lighting::Shading(_) => {}
                }
            } else {
                cmd_list.push((lnum, cmd));
            }
        }
    }
    if !vary_list.is_empty() {
        // animation mode enabled
        let frames = match frames {
            Some(f) => {
                if f <= 1 {
                    return Err(EngineError::Runtime {
                        source: RuntimeError::Other(
                            "Animation can't be enalbed (cannot use vary) when total number of frames <= 1",
                        ),
                        line: 0,
                    });
                } else {
                    f
                }
            }
            None => {
                return Err(EngineError::Runtime {
                    line: 0, // I don't want to change this field, so set to a convenient value
                    source: RuntimeError::FramesUndefined,
                });
            }
        };

        for (line, v) in vary_list.iter() {
            if v.end_frame > frames {
                return Err(EngineError::Runtime {
                    line: *line,
                    source: RuntimeError::Semantics(
                        "end_frame of vary must be <= total number of frames",
                    ),
                });
            }
        }

        let basename = basename.unwrap_or_else(|| {
            eprintln!(
                "Animation enabled by frames > 1, but basename not given. Set to `output.gif`"
            );
            String::from("output.gif")
        });
        Ok(ExecContext::Animation {
            script,
            cmd_list,
            basename,
            frames,
            vary_list,
            light_props: constants_table,
        })
    } else {
        // no animation
        Ok(ExecContext::NoAnimation {
            script,
            cmd_list,
            basename: basename.unwrap_or_else(|| String::from("output.png")),
            light_props: constants_table,
        })
    }
}
