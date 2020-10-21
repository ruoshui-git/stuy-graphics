use std::fmt;

use crate::light::LightProps;

#[derive(Debug, PartialEq)]
pub enum Type {
    LightProps(LightProps),
    Coord(Coord),
    Knob(Knob),
    KnobList(Vec<Knob>),
}

impl Type {
    pub fn kind(&self) -> Kind {
        match self {
            Type::LightProps(_) => Kind::Const,
            Type::Coord(_) => Kind::Coord,
            Type::Knob(_) => Kind::Knob,
            Type::KnobList(_) => Kind::KnobList,
        }
    }
}

/// Types without the underlying data, used for type checking
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Kind {
    Const,
    Coord,
    Knob,
    KnobList,
}

impl From<&Type> for Kind {
    fn from(t: &Type) -> Self {
        match t {
            Type::LightProps(_) => Kind::Const,
            Type::Coord(_) => Kind::Coord,
            Type::Knob(_) => Kind::Knob,
            Type::KnobList(_) => Kind::KnobList,
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "type::{}",
            match self {
                Kind::Const => {
                    "constants"
                }
                Kind::Coord => {
                    "coord_system"
                }
                Kind::Knob => {
                    "knob"
                }
                Kind::KnobList => {
                    "knoblist"
                }
            }
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Knob(f64);

/// Probably will be unused
#[derive(Debug, PartialEq)]
pub struct Coord;
