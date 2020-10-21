use indicatif::ProgressBar;

use crate::{light::LightProps, matrix::transform as tr, Drawer, Matrix, PPMImg};

use super::{
    ast::{self, Command, Symbol},
    parser::SymTable,
    result::{EngineError, EngineResult},
    utils::{warn_disabled_in_animation, warn_unimpl},
};

pub(crate) fn exec_no_animation(
    commands: Vec<(usize, Command)>,
    script: &[String],
    light_props: &SymTable<LightProps>,
    drawer: &mut Drawer<PPMImg>,
    pgbar: &ProgressBar,
) -> EngineResult<()> {
    // let mut magick = pipe_to_magick(vec!["ppm:-", &format!("{}.png", basename)]);
    // let magick_in = magick.stdin.take().unwrap();

    for (line, cmd) in commands {
        pgbar.set_message("Rendering image");
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
                ast::Misc::Save(filepath) => {
                    pgbar.set_message("Saving image with magick");
                    let status = drawer.save(&filepath).map_err(|e| EngineError::Runtime {
                        line,
                        source: e.into(),
                    })?;
                    pgbar.println(&format!("File \"{}\" saved. `magick` {}", filepath, status));
                }
                ast::Misc::GenerateRayfiles => warn_unimpl("generate_rayfiles", line),
                ast::Misc::Focal(_) => warn_unimpl("focal", line),
                ast::Misc::Display => {
                    pgbar.set_message("Displaying image");
                    drawer.display();
                }
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

pub(crate) fn exec_once_with_animation(
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
