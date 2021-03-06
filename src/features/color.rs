/// Colors.

use crate::canvas::Canvas;


/// A color.
#[derive(Clone, Copy, Debug)]
pub struct Color {
    red: f64,
    green: f64,
    blue: f64,
    alpha: f64
}

impl Color {
    pub const fn rgb(red: f64, green: f64, blue: f64) -> Self {
        Color { red, green, blue, alpha: 1. }
    }

    pub const fn rgba(red: f64, green: f64, blue: f64, alpha: f64) -> Self {
        Color { red, green, blue, alpha }
    }

    pub const fn grey(level: f64) -> Self {
        Color::rgb(level, level, level)
    }

    pub fn apply(&self, canvas: &Canvas) {
        canvas.set_source_rgba(self.red, self.green, self.blue, self.alpha)
    }
}

impl Color {
    pub const BLACK: Color = Color::rgb(0., 0., 0.);
    pub const RED: Color = Color::rgb(1., 0., 0.);
}
