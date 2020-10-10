use crate::graphics::{Canvas, Matrix, RGB};
use std::io;

use super::lights::LightConfig;

/// A procedural interface to simplfy drawing
pub struct Drawer<T: Canvas> {
    stack: Vec<Matrix>,
    canvas: T,
    pub fg_color: RGB,
    pub bg_color: RGB,
    pub light: LightConfig,
}

pub struct DrawerBuilder<T: Canvas> {
    canvas: T,
    fg_color: RGB,
    bg_color: RGB,
    light: LightConfig,
}

impl<T: Canvas> DrawerBuilder<T> {

    /// Fill a drawer
    pub fn new(canvas: T) -> Self {
        Self {
            canvas,
            fg_color: RGB::WHITE,
            bg_color: RGB::BLACK,
            light: LightConfig::TEST_LIGHT,
        }
    }

    pub fn with_fg_color(mut self, color: RGB) -> Self {
        self.fg_color = color;
        self
    }

    pub fn with_bg_color(mut self, color: RGB) -> Self {
        self.bg_color = color;
        self
    }

    pub fn with_light(mut self, light: LightConfig) -> Self {
        self.light = light;
        self
    }

    pub fn build(self) -> Drawer<T> {
        Drawer {
            stack: new_stack(),
            canvas: self.canvas,
            fg_color: self.fg_color,
            bg_color: self.bg_color,
            light: self.light,
        }
    }
}

// helpers
impl<T: Canvas> Drawer<T> {
    pub fn render_edges_with_stack(&mut self, m: &Matrix) {
        self.canvas
            .render_edge_matrix(&(m * self.get_top_matrix()), self.fg_color)
    }

    pub fn render_polygons_with_stack(&mut self, m: &Matrix) {
        self.canvas
            .render_polygon_matrix(&(m * self.get_top_matrix()), self.light)
    }

    fn get_top_matrix(&self) -> &Matrix {
        self.stack
            .last()
            .expect("Error trying to get the last stack")
    }

}

impl<T: Canvas> Drawer<T> {

    /// Create a new drawer with fg_color white and bg_color black
    pub fn new(canvas: T) -> Self {
        DrawerBuilder::new(canvas).build()
    }

    pub fn clear(&mut self) {
        self.canvas.clear(self.bg_color);

        // this will cause unexpected behaviors
        // self.stack = Self::new_stack();
    }

    pub fn reset_stack(&mut self) {
        self.stack = new_stack();
    }

    pub fn save(&self, filepath: &str) -> io::Result<()> {
        self.canvas.save(filepath)
    }

    pub fn display(&self) {
        self.canvas.display();
    }

    pub fn write_to_buf(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        self.canvas.write_to_buf(writer)
    }
}

// one dimensional stuff
impl<T: Canvas> Drawer<T> {
    pub fn draw_line(&mut self, p0: (f64, f64, f64), p1: (f64, f64, f64)) {
        let mut edges = Matrix::new_edge_matrix();
        edges.append_edge(&[p0.0, p0.1, p0.2, p1.0, p1.1, p1.2]);
        self.render_edges_with_stack(&edges);
    }
    pub fn draw_circle(&mut self, c: (f64, f64, f64), r: f64) {
        let mut edges = Matrix::new_edge_matrix();
        edges.add_circle(c, r);
        self.render_edges_with_stack(&edges);
    }

    pub fn draw_hermite(&mut self, p0: (f64, f64), p1: (f64, f64), r0: (f64, f64), r1: (f64, f64)) {
        let mut edges = Matrix::new_edge_matrix();
        edges.add_hermite3(p0, p1, r0, r1);
        self.render_edges_with_stack(&edges);
    }
    pub fn draw_bezier(&mut self, p0: (f64, f64), p1: (f64, f64), p2: (f64, f64), p3: (f64, f64)) {
        let mut edges = Matrix::new_edge_matrix();
        edges.add_bezier3(p0, p1, p2, p3);
        self.render_edges_with_stack(&edges);
    }
}

// transformations
impl<T: Canvas> Drawer<T> {
    pub fn transform_by(&mut self, trans: &Matrix) {
        *self
            .stack
            .last_mut()
            .expect("Error trying to get the last stack") = trans * self.get_top_matrix();
    }
}

// 2d shapes
impl<T: Canvas> Drawer<T> {
    pub fn add_box(&mut self, (x, y, z): (f64, f64, f64), dx: f64, dy: f64, dz: f64) {
        let mut m = Matrix::new_polygon_matrix();
        m.add_box((x, y, z), dx, dy, dz);
        // self.render_polygons_with_stack(&m);
    }
    pub fn add_sphere(&mut self, center: (f64, f64, f64), radius: f64) {
        let mut m = Matrix::new_polygon_matrix();
        m.add_sphere(center, radius);
        // self.render_polygons_with_stack(&m);
    }
    pub fn add_torus(&mut self, center: (f64, f64, f64), radius1: f64, radius2: f64) {
        let mut m = Matrix::new_polygon_matrix();
        m.add_torus(center, radius1, radius2);
        // self.render_polygons_with_stack(&m);
    }
}

// coordinate stack related
impl<T: Canvas> Drawer<T> {
    pub fn push_matrix(&mut self) {
        self.stack.push(self.get_top_matrix().clone());
    }

    pub fn pop_matrix(&mut self) {
        self.stack.pop();
    }
}

/// Make a new matrix stack
fn new_stack() -> Vec<Matrix> {
    vec![Matrix::ident(4)]
}

#[cfg(test)]
mod tests {
        use crate::graphics::PPMImg;

use super::*;
use crate::graphics::utils;
    #[test]
    fn test_line() {
        let mut img = PPMImg::new(500, 500, 255);
        img.draw_line((0.,0.,0.), (100., 100., 100.,), RGB::WHITE);
        
        utils::display_ppm(&img);
    }
}