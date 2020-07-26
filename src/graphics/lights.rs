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

        let ndotdir = normaln * dirvecn;

        let iambient = self.ambient_color.into() * self.areflect;
        let idiffuse = self.dir_color * self.dreflect * (ndotdir);
        let ispecular = self.dir_color * self.sreflect * ((2 * normaln * (ndotdir) - dirvecn) * viewn).powi(5);
        
        // (iambient + idiffuse + ispecular).into();
        todo!("Convert vec3 into RGB, limit color")
    }
}