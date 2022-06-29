/// Rendering of markers.
///
/// How a marker is rendered is selected via the symbol set passed to the
/// marker. Each known base marker has its own designated symbol. This base
/// can be modified via the following additional symbols:
///
/// *  `:right`, `:top`, `:left`, `:bottom`: Apply the marker to the right,
///    top, left, bottom of the position, respectively. The default of none
///    of these is provided is `:right`.
///
/// *  `:closed`, `:removed`: The entity described by the marker has been
///    closed or removed.

use cairo::Operator;
use kurbo::Rect;
use std::f64::consts::PI;
use crate::render::canvas::Canvas;
use crate::render::path::{Distance, Position};
use super::super::class::Class;
use super::super::style::Style;

//------------ Statdot -------------------------------------------------------

/// The rendering rule for a station dot marker.
pub struct Statdot {
    /// The position the marker is attached to.
    position: Position,

    /// How far to start to the left of the position.
    left: Distance,

    /// How far to end to the right of the position.
    right: Distance,

    /// The feature class.
    class: Class,
}


impl Statdot {
    pub fn new(
        position: Position,
        left: Distance,
        right: Distance,
        class: Class,
    ) -> Self {
        Statdot { position, left, right, class }
    }

    pub fn storage_bounds(&self) -> Rect {
        self.position.storage_bounds()
    }

    pub fn render(&self, style: &Style, canvas: &Canvas) {
        let (point, angle) = self.position.resolve(canvas);
        let left = self.left.resolve(point, canvas);
        let right = self.right.resolve(point, canvas);
        let dt = style.dimensions().dt;
        canvas.translate(point.x, point.y);
        canvas.rotate(angle);

        canvas.arc(0., -left, dt, PI, 2. * PI);
        canvas.arc(0., right, dt, 0., PI);
        canvas.line_to(-1. * dt, -left);

        canvas.set_operator(Operator::Clear);
        canvas.set_line_width(0.7 * dt);
        canvas.stroke_preserve().unwrap();
        canvas.set_operator(Operator::Over);
        style.marker_color(&self.class).apply(canvas);
        canvas.fill().unwrap();
        canvas.identity_matrix();
    }
}

