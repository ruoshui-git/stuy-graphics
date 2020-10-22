use crate::{mdl::ast::ObjConst, vector::Vec3, RGB};

/// Represents lighting configuration
#[derive(Copy, Clone, Debug)]
pub struct LightConfig {
    /// view: view vector
    pub view: Vec3,
    /// ambient: color of ambient light
    pub ambient_color: RGB,
    /// color of directional light source
    pub dir_color: RGB,
    /// The vector from the surface of an object to a point light source ( <x, y, z> )
    pub dir_vec: Vec3,
    /// ambient reflection const
    pub areflect: Vec3,
    /// diffuse reflection const
    pub dreflect: Vec3,
    /// specular reflection const
    pub sreflect: Vec3,
}

#[derive(Debug, Copy, Clone)]
pub enum Light {
    Ambient(RGB),
    Point {
        color: RGB,
        location: Vec3,
        /// fn to compute intensity based on distance; shoud not capture environment
        ///
        /// for directional light, this will be unnecessary and can return 1. for simplicity
        fatt: fn(f64) -> f64,
    },
}

/// aka Constants
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LightProps {
    /// ambient reflection rgb
    pub ka: Vec3,
    /// diffuse reflection rgb
    pub kd: Vec3,
    /// specular reflection rgb
    pub ks: Vec3,
    /// rgb intensity (I guess it's the color of the obj)
    pub intensities: Vec3,
    /// Shininess (i.e. specular hightlight (exponent))
    ///
    /// Defines the focus of specular highlights in the material. Ns values normally range from 0 to 1000, with a high value resulting in a tight, concentrated highlight.
    pub ns: f64,
}

impl From<ObjConst> for LightProps {
    fn from(oc: ObjConst) -> Self {
        Self {
            ka: Vec3(oc.kar, oc.kag, oc.kab),
            kd: Vec3(oc.kdr, oc.kdg, oc.kdb),
            ks: Vec3(oc.ksr, oc.ksg, oc.ksb),
            intensities: Vec3(
                oc.ir.unwrap_or(0.),
                oc.ig.unwrap_or(0.),
                oc.ib.unwrap_or(0.),
            ),
            // Default value
            ns: 10.,
        }
    }
}

pub fn compute_color(
    props: &LightProps,
    lights: &[Light],
    surface_normal: Vec3,
    view_vec: Vec3,
    surface_location: Vec3,
) -> RGB {
    let mut color = Vec3(0., 0., 0.);

    let normaln = surface_normal.norm();
    let viewn = view_vec.norm();

    for light in lights.iter() {
        // lights are additive, so sum up all the effects of light on this surface
        color = (color
            + match light {
                Light::Ambient(ambient) => props.ka.mul_across(Vec3::from(ambient)),
                Light::Point {
                    color: pt_color,
                    location: pt_location,
                    fatt: intensity_from_distance,
                } => {
                    // deal with diffuse and specular reflections here

                    // vector from surface of an object to light location
                    let dirvec: Vec3 = *pt_location - surface_location;
                    let dirvecn: Vec3 = dirvec.norm();

                    let ndotdir: f64 = normaln.dot(dirvecn).max(0.);

                    let idiffuse: Vec3 = Vec3::from(pt_color).mul_across(props.kd) * ndotdir;

                    let ispecular: Vec3 = Vec3::from(pt_color).mul_across(props.ks)
                        * (((2 * normaln * ndotdir - dirvecn) * viewn).max(0.).powf(props.ns));

                    (idiffuse.limit(0., 255.) + ispecular.limit_max(255.))
                        * intensity_from_distance(dirvec.mag())
                }
            }
            .limit(0., 255.))
        .limit_max(255.);
    }

    // TODO: possibely add support for light intensities here by multiplying
    RGB::from(color)
}

pub fn default_lights() -> Vec<Light> {
    vec![
        Light::Ambient(RGB {
            red: 100,
            green: 100,
            blue: 50,
        }),
        Light::Point {
            color: RGB {
                red: 252,
                green: 219,
                blue: 3,
            },
            location: Vec3(250., 500., 0.),
            fatt: fatt::no_effect,
        },
        Light::Point {
            color: RGB {
                red: 44,
                green: 156,
                blue: 73,
            },
            location: Vec3(250., 0., 0.),
            fatt: fatt::no_effect,
        },
    ]
}

/// Point light intensity functions
pub mod fatt {

    /// distance of light has no effect on intensity
    pub fn no_effect(_distance: f64) -> f64 {
        1.
    }
}

impl LightProps {
    pub const DEFAULT_PROPS: Self = Self {
        ka: Vec3(0.3, 0.3, 0.3),
        kd: Vec3(0.5, 0.5, 0.5),
        ks: Vec3(0.5, 0.5, 0.5),
        intensities: Vec3::ZEROS,
        ns: 10.,
    };

    pub const BRASS: Self = Self {
        ka: Vec3(0.329412, 0.223529, 0.027451),
        kd: Vec3(0.780392, 0.568627, 0.113725),
        ks: Vec3(0.992157, 0.941176, 0.807843),
        intensities: Vec3::ZEROS,
        ns: 27.8974,
    };

    pub const POLISHED_COPPER: Self = Self {
        ka: Vec3(0.2295, 0.08825, 0.0275),       // a=1
        kd: Vec3(0.5508, 0.2118, 0.066),         // a=1
        ks: Vec3(0.580594, 0.223257, 0.0695701), // a=1
        intensities: Vec3::ZEROS,
        ns: 51.2,
    };

    pub const GOLD: Self = Self {
        ka: Vec3(0.24725, 0.1995, 0.0745),      // a=1
        kd: Vec3(0.75164, 0.60648, 0.22648),    // a=1
        ks: Vec3(0.628281, 0.555802, 0.366065), // a=1
        intensities: Vec3::ZEROS,
        ns: 51.2,
    };

    pub const POLISHED_GOLD: Self = Self {
        ka: Vec3(0.24725, 0.2245, 0.0645),
        kd: Vec3(0.34615, 0.3143, 0.0903),
        ks: Vec3(0.797357, 0.723991, 0.208006),
        intensities: Vec3::ZEROS,
        ns: 83.2,
    };

    pub const SILVER: Self = Self {
        ka: Vec3(0.19225, 0.19225, 0.19225),    //a=1
        kd: Vec3(0.50754, 0.50754, 0.50754),    //a=1
        ks: Vec3(0.508273, 0.508273, 0.508273), //a=1
        intensities: Vec3::ZEROS,
        ns: 51.2,
    };

    pub const POLISHED_SILVER: Self = Self {
        ka: Vec3(0.23125, 0.23125, 0.23125),
        kd: Vec3(0.2775, 0.2775, 0.2775),
        ks: Vec3(0.773911, 0.773911, 0.773911),
        intensities: Vec3::ZEROS,
        ns: 83.2,
    };
}
