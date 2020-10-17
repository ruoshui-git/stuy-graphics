
use std::str::FromStr;

use nom::{IResult, branch::alt, bytes::complete::{is_not, tag, take_while1}, character::complete::{alpha1, alphanumeric1, multispace0, one_of}, combinator::{map_res, opt, recognize, value}, error::ParseError, multi::many0, number::complete::double, sequence::{pair, preceded, terminated, tuple}};

use super::types::ObjConst;

#[derive(Debug, PartialEq)]
pub(crate) enum Command {
    Push,
    Pop,
    TransformCmd(Transform),
    ShapeCmd(Shape),
    AnimateCmd(Animate),
    LightingCmd(Lighting),
    MiscCmd(Misc),
}

fn parse_cmd(i: &str) -> IResult<&str, Command> {
    let (i, cmd) = alt((
        parse_push,
        parse_pop,
        parse_tr_cmd,
        parse_shape_cmd,
        parse_animate_cmd,
        parse_lighting_cmd,
        parse_misc_cmb,
    ))(i)?;
    Ok((i, cmd))
}

/// Parses a single line; Returns None in as data if only comment is preset
pub(crate) fn parse_line(i: &str) -> IResult<&str, Option<Command>> {
    terminated(opt(parse_cmd), lws(parse_comment))(i)
}

fn parse_comment(i: &str) -> IResult<&str, (&str, &str)> {
    pair(tag("//"), is_not("\n\r"))(i)
}

#[derive(Debug, PartialEq)]
pub(crate) enum Transform {
    Move {
        values: Point,
        knob: Option<Symbol>,
    },
    Scale {
        values: Point,
        knob: Option<Symbol>,
    },
    Rotate {
        axis: char,
        degrees: f64,
        knob: Option<Symbol>,
    },
}

#[derive(Debug, PartialEq)]
pub(crate) enum Shape {
    Sphere {
        constants: Option<Symbol>,
        center: Point,
        r: f64,
        coord: Option<Symbol>,
    },
    Torus {
        constants: Option<Symbol>,
        center: Point,
        r0: f64,
        r1: f64,
        coord: Option<Symbol>,
    },
    Box {
        constants: Option<Symbol>,
        corner: Point,
        height: f64,
        width: f64,
        depth: f64,
        coord: Option<Symbol>,
    },
    Line {
        constants: Option<Symbol>,
        point0: Point,
        coord0: Option<Symbol>,
        point1: Point,
        coord1: Option<Symbol>,
    },
    Mesh {
        constants: Option<Symbol>,
        filename: String,
        coord: Option<Symbol>,
    },
}

#[derive(Debug, PartialEq)]
pub(crate) struct Point(pub(crate) f64, pub(crate) f64, pub(crate) f64);

impl Into<(f64, f64, f64)> for Point {
    fn into(self) -> (f64, f64, f64) {
        (self.0, self.1, self.2)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Animate {
    Basename(String),
    SetKnob {
        name: Symbol,
        value: f64,
    },
    SetAllKnobs(f64),
    Tween {
        start_frame: u32,
        end_frame: u32,
        knoblist0: Symbol,
        knoblist1: Symbol,
    },
    Frames(u32),
    Vary {
        knob: Symbol,
        start_frame: u32,
        end_frame: u32,
        start_val: f64,
        end_val: f64,
    },
    SaveKnobList(Symbol),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Lighting {
    Light { color: Rgb, location: Point },
    Ambient(Rgb),
    Constants { name: Symbol, value: ObjConst },
    Shading(ShadingMode),
}

#[derive(Debug, PartialEq)]
pub(crate) struct Rgb {
    r: f64,
    g: f64,
    b: f64,
}

impl From<Point> for Rgb {
    fn from(p: Point) -> Self {
        Self {
            r: p.0,
            g: p.1,
            b: p.2,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct Symbol(pub(crate) String);

impl Symbol {
    fn from_opt(obj: Option<&str>) -> Option<Self> {
        obj.map(|f| Symbol(f.to_owned()))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ShadingMode {
    Wireframe,
    Flat,
    Gouraud,
    Phong,
    Raytrace,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Misc {
    SaveCoord(Symbol),
    Camera { eye: Point, aim: Point },
    Save(String),
    GenerateRayfiles,
    Focal(f64),
    Display,
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes leading whitespace, returning the output of `inner`.
fn lws<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl Fn(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    // delimited(multispace0, inner, multispace0)
    preceded(multispace0, inner)
}

fn triple_float(input: &str) -> IResult<&str, Point> {
    let (input, x) = lws(double)(input)?;
    let (input, y) = lws(double)(input)?;
    let (input, z) = lws(double)(input)?;
    Ok((input, Point(x, y, z)))
}

/// Parsing a symbol that starts with a letter and may contain underscores, letters and numbers
fn symbol(input: &str) -> IResult<&str, &str> {
    recognize(pair(alpha1, many0(alt((alphanumeric1, tag("_"))))))(input)
}

fn opt_symbol(i: &str) -> IResult<&str, Option<Symbol>> {
    let (i, s) = opt(lws(symbol))(i)?;
    Ok((i, Symbol::from_opt(s)))
}

fn uint(i: &str) -> IResult<&str, u32> {
    map_res(take_while1(|c: char| c.is_digit(10)), u32::from_str)(i)
}

fn parse_push(input: &str) -> IResult<&str, Command> {
    let (input, _) = lws(tag("push"))(input)?;
    Ok((input, Command::Push))
}

fn parse_pop(input: &str) -> IResult<&str, Command> {
    let (input, _) = lws(tag("pop"))(input)?;
    Ok((input, Command::Pop))
}

fn parse_move(input: &str) -> IResult<&str, Transform> {
    let (input, _) = lws(tag("move"))(input)?;
    let (input, point) = triple_float(input)?;
    let (input, knob) = lws(opt_symbol)(input)?;
    Ok((
        input,
        Transform::Move {
            values: point,
            knob,
        },
    ))
}

fn parse_scale(input: &str) -> IResult<&str, Transform> {
    let (input, _) = lws(tag("scale"))(input)?;
    let (input, point) = triple_float(input)?;
    let (input, knob) = lws(opt_symbol)(input)?;
    Ok((
        input,
        Transform::Scale {
            values: point,
            knob: knob,
        },
    ))
}

fn parse_rotate(input: &str) -> IResult<&str, Transform> {
    let (input, _) = lws(tag("rotate"))(input)?;
    let (input, axis) = lws(one_of("xyz"))(input)?;
    let (input, degrees) = lws(double)(input)?;
    let (input, knob) = lws(opt_symbol)(input)?;
    Ok((
        input,
        Transform::Rotate {
            axis,
            degrees,
            knob,
        },
    ))
}

fn parse_tr_cmd(input: &str) -> IResult<&str, Command> {
    let (input, tr) = alt((parse_move, parse_rotate, parse_scale))(input)?;
    Ok((input, Command::TransformCmd(tr)))
}

fn parse_sphere(input: &str) -> IResult<&str, Shape> {
    let (input, _) = lws(tag("sphere"))(input)?;
    let (input, constants) = opt_symbol(input)?;
    let (input, center) = lws(triple_float)(input)?;
    let (input, r) = lws(double)(input)?;
    let (input, coord) = opt_symbol(input)?;
    Ok((
        input,
        Shape::Sphere {
            constants,
            center,
            r,
            coord,
        },
    ))
}

fn parse_torus(input: &str) -> IResult<&str, Shape> {
    let (input, _) = lws(tag("torus"))(input)?;
    let (input, constants) = opt_symbol(input)?;
    let (input, center) = triple_float(input)?;
    let (input, r0) = lws(double)(input)?;
    let (input, r1) = lws(double)(input)?;
    let (input, coord) = opt_symbol(input)?;
    Ok((
        input,
        Shape::Torus {
            constants: constants,
            center,
            r0,
            r1,
            coord: coord,
        },
    ))
}

fn parse_box(i: &str) -> IResult<&str, Shape> {
    let (i, _) = lws(tag("box"))(i)?;
    let (i, c) = opt_symbol(i)?;
    let (i, p0) = triple_float(i)?;
    let (i, dims) = triple_float(i)?;
    let (i, cor) = opt_symbol(i)?;
    Ok((
        i,
        Shape::Box {
            constants: c,
            corner: p0,
            height: dims.0,
            width: dims.1,
            depth: dims.2,
            coord: cor,
        },
    ))
}

fn parse_line_shape(i: &str) -> IResult<&str, Shape> {
    let (i, _) = lws(tag("line"))(i)?;
    let (i, c) = opt_symbol(i)?;
    let (i, p0) = triple_float(i)?;
    let (i, cor0) = opt_symbol(i)?;
    let (i, p1) = triple_float(i)?;
    let (i, cor1) = opt_symbol(i)?;
    Ok((
        i,
        Shape::Line {
            constants: c,
            point0: p0,
            coord0: cor0,
            point1: p1,
            coord1: cor1,
        },
    ))
}

fn parse_mesh(i: &str) -> IResult<&str, Shape> {
    let (i, _) = lws(tag("mesh"))(i)?;
    let (i, c) = opt_symbol(i)?;
    let (i, filename) = lws(symbol)(i)?;
    let (i, coord) = opt_symbol(i)?;
    Ok((
        i,
        Shape::Mesh {
            constants: c,
            filename: filename.to_owned(),
            coord,
        },
    ))
}

fn parse_shape_cmd(i: &str) -> IResult<&str, Command> {
    let (i, shape) = alt((
        parse_sphere,
        parse_torus,
        parse_box,
        parse_line_shape,
        parse_mesh,
    ))(i)?;
    Ok((i, Command::ShapeCmd(shape)))
}

fn parse_basename(i: &str) -> IResult<&str, Animate> {
    let (i, _) = lws(tag("basename"))(i)?;
    let (i, name) = lws(symbol)(i)?;
    Ok((i, Animate::Basename(name.to_owned())))
}

fn parse_set_knob(i: &str) -> IResult<&str, Animate> {
    let (i, _) = lws(tag("set"))(i)?;
    let (i, name) = lws(symbol)(i)?;
    let (i, value) = lws(double)(i)?;
    Ok((
        i,
        Animate::SetKnob {
            name: Symbol(name.to_owned()),
            value,
        },
    ))
}

fn parse_save_knobs(i: &str) -> IResult<&str, Animate> {
    let (i, _) = lws(tag("save_knobs"))(i)?;
    let (i, knoblist) = lws(symbol)(i)?;
    Ok((i, Animate::SaveKnobList(Symbol(knoblist.to_owned()))))
}

fn parse_tween(i: &str) -> IResult<&str, Animate> {
    let (i, _) = lws(tag("tween"))(i)?;
    let (i, start_frame) = lws(uint)(i)?;
    let (i, end_frame) = lws(uint)(i)?;
    let (i, knoblist0) = lws(symbol)(i)?;
    let (i, knoblist1) = lws(symbol)(i)?;
    Ok((
        i,
        Animate::Tween {
            start_frame,
            end_frame,
            knoblist0: Symbol(knoblist0.to_owned()),
            knoblist1: Symbol(knoblist1.to_owned()),
        },
    ))
}

fn parse_num_frames(i: &str) -> IResult<&str, Animate> {
    let (i, (_, num)) = pair(lws(tag("frames")), lws(uint))(i)?;
    Ok((i, Animate::Frames(num)))
}

fn parse_vary(i: &str) -> IResult<&str, Animate> {
    let (i, (knob, start_frame, end_frame, start_val, end_val)) =
        tuple((lws(symbol), lws(uint), lws(uint), lws(double), lws(double)))(i)?;
    Ok((
        i,
        Animate::Vary {
            knob: Symbol(knob.to_owned()),
            start_frame,
            end_frame,
            start_val,
            end_val,
        },
    ))
}

fn parse_set_all_knobs(i: &str) -> IResult<&str, Animate> {
    let (i, (_, value)) = pair(lws(tag("setknobs")), lws(double))(i)?;
    Ok((i, Animate::SetAllKnobs(value)))
}

fn parse_animate_cmd(i: &str) -> IResult<&str, Command> {
    let (i, animate) = alt((
        parse_basename,
        parse_set_knob,
        parse_save_knobs,
        parse_tween,
        parse_num_frames,
        parse_vary,
        parse_set_all_knobs,
    ))(i)?;
    Ok((i, Command::AnimateCmd(animate)))
}

fn parse_light(i: &str) -> IResult<&str, Lighting> {
    let (i, (_, (triple, location))) =
        pair(lws(tag("light")), pair(triple_float, triple_float))(i)?;
    Ok((
        i,
        Lighting::Light {
            color: triple.into(),
            location,
        },
    ))
}

fn parse_ambient(i: &str) -> IResult<&str, Lighting> {
    let (i, (_, triple)) = pair(lws(tag("ambient")), lws(triple_float))(i)?;
    Ok((i, Lighting::Ambient(triple.into())))
}

fn parse_constants(i: &str) -> IResult<&str, Lighting> {
    let (i, _) = lws(tag("constants"))(i)?;
    let (i, (name, kr, kg, kb, ir, ig, ib)): (
        &str,
        (
            &str,
            Point,
            Point,
            Point,
            Option<f64>,
            Option<f64>,
            Option<f64>,
        ),
    ) = tuple((
        lws(symbol),
        triple_float,
        triple_float,
        triple_float,
        opt(lws(double)),
        opt(lws(double)),
        opt(lws(double)),
    ))(i)?;

    Ok((
        i,
        Lighting::Constants {
            name: Symbol(name.to_owned()),
            value: ObjConst {
                kar: kr.0,
                kdr: kr.1,
                ksr: kr.2,
                kag: kg.0,
                kdg: kg.1,
                ksg: kg.2,
                kab: kb.0,
                kdb: kb.1,
                ksb: kb.2,
                ir,
                ig,
                ib,
            },
        },
    ))
}

fn parse_shading(i: &str) -> IResult<&str, Lighting> {
    let (i, mode) = lws(alt((
        value(ShadingMode::Wireframe ,tag("wireframe")),
        value(ShadingMode::Flat,tag("flat")),
        value(ShadingMode::Gouraud,tag("gouraud")),
        value(ShadingMode::Phong,tag("phong")),
        value(ShadingMode::Raytrace,tag("raytrace")),
    )))(i)?;
    Ok((i, Lighting::Shading(mode)))
}

fn parse_lighting_cmd(i: &str) -> IResult<&str, Command> {
    let (i, lighting) = alt((parse_light, parse_ambient, parse_constants, parse_shading))(i)?;
    Ok((i, Command::LightingCmd(lighting)))
}

fn parse_save_cor(i: &str) -> IResult<&str, Misc> {
    let (i, (_, name)) = pair(lws(tag("save_coord_system")), lws(symbol))(i)?;
    Ok((i, Misc::SaveCoord(Symbol(name.to_owned()))))
}

fn parse_cam(i: &str) -> IResult<&str, Misc> {
    let (i, (_, eye, aim)) = tuple((tag("camera"), triple_float, triple_float))(i)?;
    Ok((i, Misc::Camera { eye, aim }))
}

fn parse_save_file(i: &str) -> IResult<&str, Misc> {
    let (i, (_, filename)) = pair(lws(tag("save")), lws(symbol))(i)?;
    Ok((i, Misc::Save(filename.to_owned())))
}

fn parse_gen_rayfiles(i: &str) -> IResult<&str, Misc> {
    let (i, _) = lws(tag("generate_rayfiles"))(i)?;
    Ok((i, Misc::GenerateRayfiles))
}

fn parse_focal(i: &str) -> IResult<&str, Misc> {
    let (i, (_, value)) = pair(lws(tag("focal")), lws(double))(i)?;
    Ok((i, Misc::Focal(value)))
}

fn parse_display(i: &str) -> IResult<&str, Misc> {
    let (i, _) = lws(tag("display"))(i)?;
    Ok((i, Misc::Display))
}

fn parse_misc_cmb(i: &str) -> IResult<&str, Command> {
    let (i, misc) = alt((
        parse_save_cor,
        parse_cam,
        parse_save_file,
        parse_gen_rayfiles,
        parse_focal,
        parse_display,
    ))(i)?;
    Ok((i, Command::MiscCmd(misc)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_commands_parse() {
        let string = "scale .2 3.2 4.2 kooo";
        let (_, cmd) = parse_scale(string).unwrap();
        assert_eq!(
            Transform::Scale {
                values: Point(0.2, 3.2, 4.2),
                knob: Some(Symbol(String::from("kooo")))
            },
            cmd
        );
    }
}
