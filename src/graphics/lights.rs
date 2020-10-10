use crate::graphics::{vector::Vec3, RGB};
///
/// view: view vector
/// ambient: color of ambient light
/// light: (position from )
#[derive(Copy, Clone, Debug)]
pub struct LightConfig {
    pub view: Vec3,
    pub ambient_color: RGB,
    pub dir_color: RGB,
    pub dir_vec: Vec3,
    pub areflect: Vec3,
    pub dreflect: Vec3,
    pub sreflect: Vec3,
}

impl LightConfig {
    pub fn get_color_from_norm(&self, normal: Vec3) -> RGB {
        let normaln = normal.norm();
        let viewn = self.view.norm();
        let dirvecn = self.dir_vec.norm();

        let ndotdir: f64 = normaln * dirvecn;

        let iambient: Vec3 = self.areflect.mul_across(Vec3::from(self.ambient_color));
        let idiffuse: Vec3 = Vec3::from(self.dir_color).mul_across(self.dreflect) * ndotdir;
        let ispecular: Vec3 = Vec3::from(self.dir_color).mul_across(self.sreflect)
            * (((2 * normaln * ndotdir - dirvecn) * viewn).powi(10));

        // dbg!(iambient, idiffuse, ispecular);
        // dbg!(iambient.limit(255.), idiffuse.limit(255.), ispecular.limit(255.));
        (iambient.limit(0., 255.) + idiffuse.limit(0., 255.) + ispecular.limit(0., 255.)).into()
    }

    pub const TEST_LIGHT: Self = Self {
        view: Vec3(0., 0., 1.),
        ambient_color: RGB {
            red: 50,
            green: 50,
            blue: 50,
        },
        dir_color: RGB {
            red: 0,
            green: 255,
            blue: 255,
        },
        dir_vec: Vec3(0.5, 0.75, -1.),
        areflect: Vec3(0.1, 0.1, 0.1),
        dreflect: Vec3(0.5, 0.5, 0.5),
        sreflect: Vec3(0.5, 0.5, 0.5),
    };
}

// pub(crate) fn test_light() -> LightConfig {
//     LightConfig {
//         view: Vec3(0., 0., 1.),
//         ambient_color: RGB::new(50, 50, 50),
//         dir_color: RGB::new(0, 255, 255),
//         dir_vec: Vec3(0.5, 0.75, -1.),
//         areflect: Vec3(0.1, 0.1, 0.1),
//         dreflect: Vec3(0.5, 0.5, 0.5),
//         sreflect: Vec3(0.5, 0.5, 0.5),
//     }
// }
