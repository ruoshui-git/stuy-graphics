
#[derive(Debug, PartialEq)]
pub(crate) enum Type {
    ObjConst(ObjConst),
    Coord(Coord),
    Knob(Knob),
    KnobList(Vec<Knob>),
}

#[derive(Debug, PartialEq)]
pub(crate) struct ObjConst {
    pub(crate) kar: f64,
    pub(crate) kdr: f64,
    pub(crate) ksr: f64,
    pub(crate) kag: f64,
    pub(crate) kdg: f64,
    pub(crate) ksg: f64,
    pub(crate) kab: f64,
    pub(crate) kdb: f64,
    pub(crate) ksb: f64,
    pub(crate) ir: Option<f64>,
    pub(crate) ig: Option<f64>,
    pub(crate) ib: Option<f64>,
}


#[derive(Debug, PartialEq)]
pub(crate) struct Knob(f64);


/// Probably will be unused
#[derive(Debug, PartialEq)]
pub(crate) struct Coord;