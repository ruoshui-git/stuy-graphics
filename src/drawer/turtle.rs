use super::Canvas;
use crate::{colors::RGB, utils::polar_to_xy};

// A turtle drawer on the plane Z = 0 (no depth)
pub struct Turtle<T: Canvas> {
    x: f64,
    y: f64,
    pub angle_deg: f64,
    pub pen_down: bool,
    pub fg_color: RGB,
    img: T,
}

impl<T: Canvas> Turtle<T> {
    fn new(screen: T, x: f64, y: f64, fg_color: RGB) -> Turtle<T> {
        Turtle {
            x,
            y,
            angle_deg: 0.0,
            pen_down: false,
            img: screen,
            fg_color,
        }
    }

    pub fn forward(&mut self, steps: i32) {
        let (x0, y0) = (self.x, self.y);
        let (dx, dy) = polar_to_xy(steps.into(), self.angle_deg);
        let (x1, y1) = (x0 as f64 + dx, y0 as f64 + dy);
        if self.pen_down {
            self.img
                .draw_line((x0, y0, 0.), (x1, y1, 0.), self.fg_color);
        }
        self.x = x1;
        self.y = y1;
    }

    pub fn turn_rt(&mut self, angle_deg: f64) {
        self.angle_deg = (self.angle_deg + angle_deg) % 360.0;
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        if self.pen_down {
            self.img
                .draw_line((self.x, self.y, 0.), (x, y, 0.), self.fg_color);
        }
        self.x = x;
        self.y = y;
    }

    /// Get the inner Canvas (T) instance
    ///
    /// This method will move the turtle
    pub fn get_canvas(self) -> T {
        self.img
    }
}
