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

use std::f64::consts::PI;
use crate::canvas::Canvas;
use crate::features::color::Color;
use crate::features::marker::RenderMarker;
use crate::features::path::Position;
use crate::import::ast;
use crate::import::Failed;
use crate::import::eval::{Error, SymbolSet};


//------------ Constants -----------------------------------------------------

const OPEN_GREY: f64 = 0.;
const CLOSED_GREY: f64 = 0.4;
const REMOVED_GREY: f64 = 0.7;


//------------ Units ---------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct Units {
    /// The distance between two parallel tracks.
    dt: f64,

    /// The width of a station symbol.
    sw: f64,

    /// The height of a station symbol.
    sh: f64,

    /// The radius of curves on station symbols.
    ds: f64,

    /// The line width of station symbols.
    sp: f64,

    /// The line width of border symbols.
    bp: f64,
}

impl Units {
    fn new(canvas: &Canvas) -> Self {
        Units {
            dt: super::units::DT * canvas.canvas_bp(),
            sw: 6. * canvas.canvas_bp(),
            sh: 5.8 * canvas.canvas_bp(),
            ds: 0.5 * canvas.canvas_bp(),
            sp: 0.8 * canvas.canvas_bp(),
            bp: 0.6 * canvas.canvas_bp(),
        }
    }
}

//------------ StandardMarker ------------------------------------------------

/// The rendering rule for a standard marker.
pub struct StandardMarker {
    /// Extra rotation in addition to whatever the position dictates.
    rotation: f64,

    /// The color to render the symbol in.
    color: Color,

    /// The index of the marker to use.
    marker: usize
}


impl StandardMarker {
    pub fn create(
        pos: ast::Pos, symbols: SymbolSet, err: &mut Error
    ) -> Result<Self, Failed> {
        let color = if symbols.contains("removed") {
            Color::grey(REMOVED_GREY)
        }
        else if symbols.contains("closed") {
            Color::grey(CLOSED_GREY)
        }
        else {
            Color::grey(OPEN_GREY)
        };
        let rotation = if symbols.contains("top") { 0.5 * PI }
                       else if symbols.contains("left") { PI }
                       else if symbols.contains("bottom") { 1.5 * PI }
                       else { 0. };
        for (index, marker) in MARKERS.iter().enumerate() {
            if symbols.contains(marker.0) {
                return Ok(StandardMarker {
                    rotation, color,
                    marker: index
                })
            }
        }
        err.add(pos, "no reference to a known marker");
        Err(Failed)
    }
}

impl RenderMarker for StandardMarker {
    fn render(&self, canvas: &Canvas, position: &Position) {
        let (point, angle) = position.resolve(canvas);
        canvas.translate(point.x, point.y);
        canvas.rotate(angle + self.rotation);
        self.color.apply(canvas);
        MARKERS[self.marker].1(canvas, Units::new(canvas));
        canvas.identity_matrix();
    }
}


//------------ Markers ------------------------------------------------------

const MARKERS: &[(&'static str, &'static dyn Fn(&Canvas, Units))] = &[
    ("de_bf", &|canvas, u| {
        canvas.move_to(-0.5 * u.sw, 0.);
        canvas.line_to(-0.5 * u.sw, u.sh - u.ds);
        canvas.curve_to(
            -0.5 * u.sw, u.sh - 1.5 * u.ds,
            -0.5 * u.sw + 0.5 * u.ds, u.sh,
            -0.5 * u.sw + u.ds, u.sh
        );
        canvas.line_to(0.5 * u.sw - u.ds, u.sh);
        canvas.curve_to(
            0.5 * u.sw - 0.5 * u.ds, u.sh,
            0.5 * u.sw, u.sh - 0.5 * u.ds,
            0.5 * u.sw, u.sh - u.ds
        );
        canvas.line_to(0.5 * u.sw, 0.);
        canvas.close_path();
        canvas.fill();
    }),

    ("de_hp", &|canvas, u| {
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, 0.);
        canvas.line_to(-0.5 * u.sw + 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, 0.);
        canvas.set_line_width(u.sp);
        canvas.stroke();
    }),

    ("de_abzw", &|canvas, u| {
        canvas.set_line_width(u.sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., 0.5 * u.sh);
        canvas.move_to(-0.5 * u.sw, 0.5 * u.sh);
        canvas.line_to(0.5 * u.sw, 0.5 * u.sh);
        canvas.move_to(-0.5 * u.sw, u.sh);
        canvas.line_to(0., 0.5 * u.sh);
        canvas.line_to(0.5 * u.sw, u.sh);
        canvas.stroke();
    }),

    ("de_dirgr", &|canvas, u| {
        let r = 0.8 * u.dt;
        canvas.set_line_width(u.bp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., 2. * r);
        canvas.stroke();
        canvas.arc(0., 3. * r, r, 0., 2. * PI);
        canvas.stroke();
        canvas.arc(0., 3. * r, 0.5 * r, 0., 2. * PI);
        canvas.fill();
    }),

    ("statdt", &|canvas, u| {
        canvas.set_line_width(u.bp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., u.dt);
        canvas.stroke();
    }),
];

