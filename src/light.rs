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
    pub areflect: Vec3,
    /// diffuse reflection rgb
    pub dreflect: Vec3,
    /// specular reflection rgb
    pub sreflect: Vec3,
    /// rgb intensity (I guess it's the color of the obj)
    pub intensities: Vec3,
}

impl From<ObjConst> for LightProps {
    fn from(oc: ObjConst) -> Self {
        Self {
            areflect: Vec3(oc.kar, oc.kag, oc.kab),
            dreflect: Vec3(oc.kdr, oc.kdg, oc.kdb),
            sreflect: Vec3(oc.ksr, oc.ksg, oc.ksb),
            intensities: Vec3(
                oc.ir.unwrap_or(0.),
                oc.ig.unwrap_or(0.),
                oc.ib.unwrap_or(0.),
            ),
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
                Light::Ambient(ambient) => props.areflect.mul_across(Vec3::from(ambient)),
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

                    let idiffuse: Vec3 = Vec3::from(pt_color).mul_across(props.dreflect) * ndotdir;

                    let ispecular: Vec3 = Vec3::from(pt_color).mul_across(props.sreflect)
                        * (((2 * normaln * ndotdir - dirvecn) * viewn).max(0.).powi(10));

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
            location: Vec3(250., 0., 0.,),
            fatt: fatt::no_effect,
        }
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
        areflect: Vec3(0.3, 0.3, 0.3),
        dreflect: Vec3(0.5, 0.5, 0.5),
        sreflect: Vec3(0.5, 0.5, 0.5),
        intensities: Vec3(0., 0., 0.),
    };
}

// impl LightConfig {
//     pub fn get_color_from_norm(&self, surface_normal: Vec3) -> RGB {
//         let normaln = surface_normal.norm();
//         let viewn = self.view.norm();
//         let dirvecn = self.dir_vec.norm();

//         let ndotdir: f64 = normaln.dot(dirvecn).max(0.);

//         let iambient: Vec3 = self.areflect.mul_across(Vec3::from(self.ambient_color));
//         let idiffuse: Vec3 = Vec3::from(self.dir_color).mul_across(self.dreflect) * ndotdir;
//         let ispecular: Vec3 = Vec3::from(self.dir_color).mul_across(self.sreflect)
//             * (((2 * normaln * ndotdir - dirvecn) * viewn).max(0.).powi(10));

//         (iambient.limit(0., 255.) + idiffuse.limit(0., 255.) + ispecular.limit(0., 255.)).into()
//     }

//     pub const TEST_LIGHT: Self = Self {
//         view: Vec3(0., 0., 1.),
//         ambient_color: RGB {
//             red: 50,
//             green: 50,
//             blue: 50,
//         },
//         dir_color: RGB {
//             red: 252,
//             green: 219,
//             blue: 3,
//         },
//         dir_vec: Vec3(0.5, 0.75, 1.),
//         areflect: Vec3(0.1, 0.1, 0.1),
//         dreflect: Vec3(0.7, 0.7, 0.7),
//         sreflect: Vec3(0.5, 0.5, 0.5),
//     };
// }
