#![allow(dead_code)]

use std::cmp;
use std::convert;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct RGB {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

// Constructor and some useful "constants"
impl RGB {
    pub const WHITE: RGB = RGB {
        red: 255,
        green: 255,
        blue: 255,
    };

    pub const BLACK: RGB = RGB {
        red: 0,
        green: 0,
        blue: 0,
    };

    pub fn gray(depth: u16) -> Self {
        RGB {
            red: depth,
            green: depth,
            blue: depth,
        }
    }

    pub fn new(red: u16, green: u16, blue: u16) -> Self {
        RGB { red, green, blue }
    }
}

/// Hue, Saturation, Luminosity
///
/// range: [0, 1]
#[derive(Copy, Clone)]
pub struct HSL {
    pub h: f64,
    pub s: f64,
    pub l: f64,
}

impl convert::From<HSL> for RGB {
    // https://en.wikipedia.org/wiki/HSL_and_HSV#HSL_to_RGB_alternative
    fn from(hsl: HSL) -> RGB {
        let hue = (hsl.h * 360.0).round() as i32;
        let a = hsl.s * hsl.l.min(1.0 - hsl.l);
        let f = |n: i32| {
            hsl.l
                - a * (-1.0f64).max(
                    ((n + hue / 30) % 12 - 3)
                        .min(9 - (n + hue / 30) % 12)
                        .min(1) as f64,
                )
        };
        let (red, green, blue) = (
            f(0).round() as i32,
            f(8).round() as i32,
            f(4).round() as i32,
        );
        assert!(red == 1 || red == 0);
        assert!(green == 1 || green == 0);
        assert!(blue == 1 || blue == 0);
        RGB {
            red: (f(0) * 255.0) as u16,
            green: (f(8) * 255.0) as u16,
            blue: (f(4) * 255.0) as u16,
        }
    }
}

fn fmax2(a: f64, b: f64, prec: i32) -> f64 {
    let fprec = prec as f64;
    cmp::max((a * fprec).round() as i32, (b * fprec).round() as i32) as f64 / fprec
}

fn fmin2(a: f64, b: f64, prec: i32) -> f64 {
    let fprec = prec as f64;
    cmp::min((a * fprec).round() as i32, (b * fprec).round() as i32) as f64 / fprec
}

fn fmin3(a: f64, b: f64, c: f64, prec: i32) -> f64 {
    fmin2(fmin2(a, b, prec), c, prec)
}

// impl From<Vec3> for RGB {

// }
