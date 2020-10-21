#![allow(dead_code, unused_variables)]

pub mod ast;
pub mod parser;
pub mod result;
pub mod types;
mod utils;

use std::{path::Path, path::PathBuf};

use crate::{
    drawer::DrawerBuilder,
    light::LightProps,
    matrix::transform as tr,
    processes::{pipe_to_magick, wait_for_magick},
    Drawer, Matrix, PPMImg,
};

use self::{
    ast::Command,
    ast::Symbol,
    ast::VaryInfo,
    parser::{parse_file, SymTable},
    result::EngineError,
    result::EngineResult,
    utils::{warn_disabled_in_animation, warn_unimpl},
};

fn exec_no_animation(
    commands: Vec<(usize, Command)>,
    basename: &str,
    script: &[String],
    light_props: &SymTable<LightProps>,
) -> EngineResult<()> {
    // let mut magick = pipe_to_magick(vec!["ppm:-", &format!("{}.png", basename)]);
    // let magick_in = magick.stdin.take().unwrap();

    let mut drawer = DrawerBuilder::new(PPMImg::new(500, 500, 255))
        // .with_writer(Box::new(magick_in))
        .build();

    for (line, cmd) in commands {
        match cmd {
            Command::Push => drawer.push_matrix(),
            Command::Pop => drawer.pop_matrix(),
            Command::TransformCmd(transform) => drawer.transform_by(&match transform {
                ast::Transform::Move { values, knob: _ } => tr::mv(values.0, values.1, values.2),
                ast::Transform::Scale { values, knob: _ } => {
                    tr::scale(values.0, values.1, values.2)
                }
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
                } => drawer.add_sphere(center.into(), r, light_props.find(&constants)?),
                ast::Shape::Torus {
                    constants,
                    center,
                    r0,
                    r1,
                    coord,
                } => drawer.add_torus(center.into(), r0, r1, light_props.find(&constants)?),
                ast::Shape::Box {
                    constants,
                    corner,
                    height,
                    width,
                    depth,
                    coord,
                } => drawer.add_box(
                    corner.into(),
                    width,
                    height,
                    depth,
                    light_props.find(&constants)?,
                ),
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
                } => warn_unimpl("mesh", line),
            },
            Command::AnimateCmd(a) => match a {
                ast::Animate::Basename(_) => unreachable!(),
                ast::Animate::SetKnob { name: _, value: _ } => warn_unimpl("set_knob", line),
                ast::Animate::SetAllKnobs(_) => warn_unimpl("set_knobs", line),
                ast::Animate::Tween {
                    start_frame: _,
                    end_frame: _,
                    knoblist0: _,
                    knoblist1: _,
                } => warn_unimpl("tween", line),
                ast::Animate::Frames(_) => unreachable!(),
                ast::Animate::Vary(_) => unreachable!(),
                ast::Animate::SaveKnobList(_) => warn_unimpl("save_knoblist", line),
            },
            Command::LightingCmd(_) => unreachable!(),
            Command::MiscCmd(cmd) => match cmd {
                ast::Misc::SaveCoord(_) => warn_unimpl("save_coord_system", line),
                ast::Misc::Camera { eye: _, aim: _ } => warn_unimpl("camera", line),
                ast::Misc::Save(filepath) => drawer.save(&filepath).or_else(|e| {
                    Err(EngineError::Runtime {
                        line,
                        source: e.into(),
                    })
                })?,
                ast::Misc::GenerateRayfiles => warn_unimpl("generate_rayfiles", line),
                ast::Misc::Focal(_) => warn_unimpl("focal", line),
                ast::Misc::Display => drawer.display(),
            },
        }
    }

    Ok(())
}

fn transform_with_knob(
    knobs: &SymTable<f64>,
    op_symbol: &Option<Symbol>,
    run_with_knob: impl Fn(&f64) -> Matrix,
    run_without_knob: impl Fn() -> Matrix,
) -> EngineResult<Matrix> {
    Ok(match knobs.find(op_symbol) {
        Ok(op_knob) => {
            if let Some(knob) = op_knob {
                run_with_knob(knob)
            } else {
                run_without_knob()
            }
        }
        Err(e) => match e {
            EngineError::SymbolNotFound { name } => {
                eprintln!(
                    "knob {} not found, ignoring this knob and applying static transformation",
                    name
                );
                run_without_knob()
            }
            other => return Err(other),
        },
    })
}

fn exec_once_with_animation(
    commands: &[(usize, Command)],
    script: &[String],
    knobs: &SymTable<f64>,
    drawer: &mut Drawer<PPMImg>,
    light_props: &SymTable<LightProps>,
) -> EngineResult<()> {
    for (line, cmd) in commands {
        match cmd {
            Command::Push => drawer.push_matrix(),
            Command::Pop => drawer.pop_matrix(),
            Command::TransformCmd(transform) => drawer.transform_by(&match transform {
                ast::Transform::Move { values, knob } => transform_with_knob(
                    knobs,
                    knob,
                    |knob| tr::mv(values.0 * knob, values.1 * knob, values.2 * knob),
                    || tr::mv(values.0, values.1, values.2),
                )?,
                ast::Transform::Scale { values, knob } => transform_with_knob(
                    knobs,
                    knob,
                    |knob| tr::scale(values.0 * knob, values.1 * knob, values.2 * knob),
                    || tr::scale(values.0, values.1, values.2),
                )?,
                ast::Transform::Rotate {
                    axis,
                    degrees,
                    knob,
                } => transform_with_knob(
                    knobs,
                    knob,
                    |knob| match axis {
                        'x' => tr::rotatex(knob * degrees),
                        'y' => tr::rotatey(knob * degrees),

                        'z' => tr::rotatez(knob * degrees),
                        _ => unreachable!(),
                    },
                    || match axis {
                        'x' => tr::rotatex(*degrees),
                        'y' => tr::rotatey(*degrees),

                        'z' => tr::rotatez(*degrees),
                        _ => unreachable!(),
                    },
                )?,
            }),
            Command::ShapeCmd(shape) => match shape {
                ast::Shape::Sphere {
                    constants,
                    center,
                    r,
                    coord,
                } => drawer.add_sphere(center.into(), *r, light_props.find(&constants)?),
                ast::Shape::Torus {
                    constants,
                    center,
                    r0,
                    r1,
                    coord,
                } => drawer.add_torus(center.into(), *r0, *r1, light_props.find(&constants)?),
                ast::Shape::Box {
                    constants,
                    corner,
                    height,
                    width,
                    depth,
                    coord,
                } => drawer.add_box(
                    corner.into(),
                    *width,
                    *height,
                    *depth,
                    light_props.find(&constants)?,
                ),
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
                } => warn_unimpl("mesh", *line),
            },
            Command::AnimateCmd(a) => match a {
                ast::Animate::Basename(_) => unreachable!(),
                ast::Animate::SetKnob { name: _, value: _ } => warn_unimpl("set_knob", *line),
                ast::Animate::SetAllKnobs(_) => warn_unimpl("set_knobs", *line),
                ast::Animate::Tween {
                    start_frame: _,
                    end_frame: _,
                    knoblist0: _,
                    knoblist1: _,
                } => warn_unimpl("tween", *line),
                ast::Animate::Frames(_) => unreachable!(),
                ast::Animate::Vary(_) => unreachable!(),
                ast::Animate::SaveKnobList(_) => warn_unimpl("save_knoblist", *line),
            },
            Command::LightingCmd(_) => unreachable!(),
            Command::MiscCmd(cmd) => match cmd {
                ast::Misc::SaveCoord(_) => warn_unimpl("save_coord_system", *line),
                ast::Misc::Camera { eye: _, aim: _ } => warn_unimpl("camera", *line),
                ast::Misc::Save(_) => warn_disabled_in_animation("save"),
                ast::Misc::GenerateRayfiles => warn_unimpl("generate_rayfiles", *line),
                ast::Misc::Focal(_) => warn_unimpl("focal", *line),
                ast::Misc::Display => warn_disabled_in_animation("display"),
            },
        }
    }

    Ok(())
}

/// MDL Interpreter for a single file
pub struct Interpreter {
    filename: PathBuf,
}

/// Config for interpreter to exec script
pub enum ExecContext {
    Animation {
        script: Vec<String>,
        cmd_list: Vec<(usize, Command)>,
        basename: String,
        frames: u32,
        vary_list: Vec<(usize, VaryInfo)>,
        light_props: SymTable<LightProps>,
    },
    NoAnimation {
        script: Vec<String>,
        cmd_list: Vec<(usize, Command)>,
        basename: String,
        light_props: SymTable<LightProps>,
    },
}

impl Interpreter {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        Self {
            filename: path.as_ref().to_path_buf(),
        }
    }

    pub fn run(&self) -> EngineResult<()> {
        match parse_file(&self.filename)? {
            ExecContext::Animation {
                cmd_list,
                basename,
                frames,
                vary_list,
                script,
                light_props,
            } => {
                // second pass, compute all knob values for each frame
                let mut knob_states: Vec<SymTable<f64>> = vec![];

                for cur_frame in 0..frames {
                    let mut table: SymTable<f64> = SymTable::new();
                    for (line, v) in vary_list.iter() {
                        if v.start_frame <= cur_frame && cur_frame <= v.end_frame {
                            let val = (v.end_val - v.start_val)
                                / (v.end_frame as f64 - v.start_frame as f64)
                                * (cur_frame - v.start_frame) as f64 + v.start_val;

                            // override previous vary command if they overlap in frame numbers
                            if table.insert(v.knob.to_owned(), val).is_some() {
                                eprintln!(
                                    "Vary commands cannot overlap. Using vary on line {}: {}",
                                    line,
                                    script.get(*line).unwrap().as_str()
                                );
                            }
                        }
                    }
                    knob_states.push(table);
                }

                let mut magick = pipe_to_magick(vec!["ppm:-", &format!("{}.gif", basename)]);
                let writer = magick.stdin.take().unwrap();
                let mut drawer = DrawerBuilder::new(PPMImg::new(500, 500, 255))
                    .with_writer(Box::new(writer))
                    .build();
                for knob_state in knob_states.iter() {
                    exec_once_with_animation(
                        &cmd_list,
                        &script,
                        knob_state,
                        &mut drawer,
                        &light_props,
                    )?;
                    drawer.flush()?;
                    drawer.reset_stack();
                    drawer.clear();
                }

                drawer.finish()?;
                wait_for_magick(magick);
            }
            ExecContext::NoAnimation {
                script,
                cmd_list,
                basename,
                light_props,
            } => exec_no_animation(cmd_list, basename.as_str(), &script, &light_props)?,
        }

        Ok(())
    }
}
