mod ast;
mod parser;
mod result;
mod types;

use std::{path::Path, path::PathBuf};

use crate::{matrix::transform as tr, Drawer, PPMImg};

use self::{
    ast::Command,
    parser::{parse_file, SymTable},
    result::EngineError,
    result::EngineResult,
    types::Type,
};

pub(crate) fn exec_cmds(commands: Vec<(usize, Command)>) -> EngineResult<()> {
    let mut symbols: SymTable<Type> = SymTable::new();

    let mut drawer = Drawer::new(PPMImg::new(500, 500, 255));

    for (line, cmd) in commands {
        match cmd {
            Command::Push => drawer.push_matrix(),
            Command::Pop => drawer.pop_matrix(),
            Command::TransformCmd(transform) => drawer.transform_by(&match transform {
                ast::Transform::Move { values, knob } => tr::mv(values.0, values.1, values.2),
                ast::Transform::Scale { values, knob } => tr::scale(values.0, values.1, values.2),
                ast::Transform::Rotate {
                    axis,
                    degrees,
                    knob: _,
                } => match axis {
                    'x' => tr::rotatex(degrees),
                    'y' => tr::rotatey(degrees),

                    'z' => tr::rotatez(degrees),
                    _ => unreachable!(),
                },
            }),
            Command::ShapeCmd(shape) => match shape {
                ast::Shape::Sphere {
                    constants,
                    center,
                    r,
                    coord,
                } => drawer.add_sphere(center.into(), r),
                ast::Shape::Torus {
                    constants,
                    center,
                    r0,
                    r1,
                    coord,
                } => drawer.add_torus(center.into(), r0, r1),
                ast::Shape::Box {
                    constants,
                    corner,
                    height,
                    width,
                    depth,
                    coord,
                } => drawer.add_box(corner.into(), width, height, depth),
                ast::Shape::Line {
                    constants,
                    point0,
                    coord0,
                    point1,
                    coord1,
                } => drawer.draw_line(point0.into(), point1.into()),
                ast::Shape::Mesh {
                    constants: _,
                    filename: _,
                    coord: _,
                } => eprintln!("mesh cmd is unsupported"),
            },
            Command::AnimateCmd(a) => eprintln!("unsupported: {:?}", a),
            Command::LightingCmd(c) => match c {
                ast::Lighting::Light {
                    name: _,
                    color: _,
                    location: _,
                } => eprintln!("unimplemented: light"),
                ast::Lighting::Ambient(_) => eprintln!("unimplemented: ambient"),
                ast::Lighting::Constants { name, value } => {
                    symbols.insert(name, Type::ObjConst(value));
                }
                ast::Lighting::Shading(_) => {}
            },
            Command::MiscCmd(cmd) => match cmd {
                ast::Misc::SaveCoord(_) => {}
                ast::Misc::Camera { eye: _, aim: _ } => eprintln!("unsupported: camera"),
                ast::Misc::Save(filepath) => drawer.save(&filepath).or_else(|e| {
                    Err(EngineError::Runtime {
                        line,
                        source: e.into(),
                    })
                })?,
                ast::Misc::GenerateRayfiles => eprintln!("unsupported: generate_rayfiles"),
                ast::Misc::Focal(_) => eprintln!("unsupported: focal"),
                ast::Misc::Display => drawer.display(),
            },
        }
    }

    Ok(())
}

/// MDL Interpreter for a single file
pub struct Interpreter {
    filename: PathBuf,
    /// Whether interpreter should check for possible errors before executing code
    precheck: bool,
}

impl Interpreter {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        Self {
            filename: path.as_ref().to_path_buf(),
            precheck: false,
        }
    }

    pub fn precheck(&mut self) -> &mut Self {
        self.precheck = true;
        self
    }

    pub fn run(&self) -> EngineResult<()> {
        let cmds = parse_file(&self.filename)?;
        if self.precheck {
            check_cmds(&cmds)?;
        }
        exec_cmds(cmds)
    }
}

fn check_cmds(_cmd: &[(usize, Command)]) -> EngineResult<()> {
    todo!("impl check command");

    // let mut symbols: SymTable<Kind> = SymTable::new();
    // let mut frames: Option<u32> = None;
    /*
    // make sure I handle all statements
    let result: EngineResult<()> = match &cmd {
        Command::Push | Command::Pop => Ok(()),
        Command::TransformCmd(ts) => match ts {
            ast::Transform::Move { values, knob }
            | ast::Transform::Scale { values, knob } => symbols.check(knob, Kind::Knob),
            ast::Transform::Rotate {
                axis,
                degrees,
                knob,
            } => self.symbols.check(knob, Kind::Knob),
        },
        Command::ShapeCmd(cmd) => match cmd {
            ast::Shape::Sphere {
                constants,
                center,
                r,
                coord,
            } => self.symbols
                .check(constants, Kind::Const)
                .and_then(|_| self.symbols.check(coord, Kind::Coord)),
            ast::Shape::Torus {
                constants,
                center,
                r0,
                r1,
                coord,
            } => self.symbols
                .check(constants, Kind::Const)
                .and_then(|_| self.symbols.check(coord, Kind::Coord)),
            ast::Shape::Box {
                constants,
                corner,
                height,
                width,
                depth,
                coord,
            } => self.symbols
                .check(constants, Kind::Const)
                .and_then(|_| self.symbols.check(coord, Kind::Coord)),
            ast::Shape::Line {
                constants,
                point0,
                coord0,
                point1,
                coord1,
            } => self.symbols.check(constants, Kind::Const).and_then(|_| {
                self.symbols
                    .check(coord0, Kind::Coord)
                    .and_then(|_| self.symbols.check(coord1, Kind::Coord))
            }),
            ast::Shape::Mesh {
                constants,
                filename,
                coord,
            } => self.symbols
                .check(constants, Kind::Const)
                .and_then(|_| self.symbols.check(coord, Kind::Coord)),
        },
        Command::AnimateCmd(cmd) => match cmd {
            ast::Animate::Basename(_) => Ok(()),
            ast::Animate::SetKnob { name, value } => {
                self.symbols.insert(name.to_owned(), Kind::Knob);
                Ok(())
            }
            ast::Animate::SetAllKnobs(_) => Ok(()),
            ast::Animate::Tween {
                start_frame,
                end_frame,
                knoblist0,
                knoblist1,
            } => {
                self.symbols.check(&Some(knoblist0.to_owned()), Kind::KnobList)
                .and_then(|_| self.symbols.check(&Some(knoblist1.to_owned()), Kind::KnobList))
            }
            ast::Animate::Frames(_) => Ok(()),
            ast::Animate::Vary {
                knob,
                start_frame,
                end_frame,
                start_val,
                end_val,
            } => {}
            ast::Animate::SaveKnobList(_) => {}
        },
        Command::LightingCmd(_) => {}
        Command::MiscCmd(_) => {}
    };

    */
}
