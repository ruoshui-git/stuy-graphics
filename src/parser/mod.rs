use std::{
    collections::HashMap,
    fs::File,
    io::{self, prelude::*, BufReader},
    path::Path,
};

use ast::Command;

mod ast;
mod error;
mod types;

use error::EngineError;

use self::{ast::Symbol, types::Type};

/// Parse file into ast and report errors
fn parse_file<T: AsRef<Path>>(path: T) -> Result<Vec<(usize, Command)>, EngineError> {
    let mut fin = BufReader::new(File::open(path.as_ref())?);

    let mut cmd_list: Vec<(usize, Command)> = vec![];
    let mut symbols: HashMap<Symbol, Type> = HashMap::new();

    for (lnum, line) in fin.lines().enumerate() {
        // let (_, opcmd) =
        if let (_, Some(cmd)) = ast::parse_line(&line?).map_err(|nom_err| match nom_err {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(e) | nom::Err::Failure(e) => EngineError::Syntax {
                input: e.0.to_owned(),
                kind: e.1,
            },
        })? {
            match &cmd {
                Command::Push => None,
                Command::Pop => None,
                Command::TransformCmd(ts) => match ts {
                    ast::Transform::Move { values, knob }
                    | ast::Transform::Scale { values, knob } => match knob {
                        Some(ref symbol) => {
                            find_symbol(&symbols, &symbol).err()
                        }
                        None => None,
                    },
                    ast::Transform::Rotate {
                        axis,
                        degrees,
                        knob,
                    } => match knob {
                        Some(ref sym) => {
                            find_symbol(&symbols, &sym).err()
                        }
                        None => None
                    },
                },
                Command::ShapeCmd(cmd) => {
                    match cmd {
                        ast::Shape::Sphere { constants, center, r, coord } => {
                            
                        }
                        ast::Shape::Torus { constants, center, r0, r1, coord } => {}
                        ast::Shape::Box { constants, corner, height, width, depth, coord } => {}
                        ast::Shape::Line { constants, point0, coord0, point1, coord1 } => {}
                        ast::Shape::Mesh { constants, filename, coord } => {}
                    }
                }
                Command::AnimateCmd(cmd) => {}
                Command::LightingCmd(_) => {}
                Command::MiscCmd(_) => {}
            }
            .expect(format!("error: line {}", lnum));

            cmd_list.push((lnum, cmd));
        }
    }

    Ok(cmd_list)
}

fn find_symbol<'a>(
    symbols: &'a HashMap<Symbol, Type>,
    value: &Symbol,
) -> Result<&'a Type, EngineError> {
    symbols
        .get(value)
        .ok_or_else(|| EngineError::UndefinedSymbol {
            name: value.0.to_owned(),
        })
}


