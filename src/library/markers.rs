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
use crate::features::marker::RenderMarker;
use crate::features::path::Position;
use crate::import::ast;
use crate::import::Failed;
use crate::import::eval::{Error, SymbolSet};
use super::colors::Palette;
use super::units;


//------------ Units ---------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct Units {
    /// The distance between two parallel tracks.
    dt: f64,

    /// The width of a station symbol.
    sw: f64,

    /// The height of a station symbol.
    sh: f64,

    /// The width of a reduced size station symbol.
    ksw: f64,

    /// The height of a reduced size station symbol.
    ksh: f64,

    /// The radius of curves on station symbols.
    ds: f64,

    /// The line width of station symbols.
    sp: f64,

    /// The line width of border symbols.
    bp: f64,
}

impl Units {
    fn new(canvas: &Canvas) -> Self {
        if canvas.detail() > 3 {
            Units {
                dt:  units::DT * canvas.canvas_bp(),
                sw:  units::SW * canvas.canvas_bp(),
                sh:  units::SH * canvas.canvas_bp(),
                ksw: 0.8 * units::SW * canvas.canvas_bp(),
                ksh: 0.8 * units::SH * canvas.canvas_bp(),
                ds:  units::SH * canvas.canvas_bp() * 0.05,
                sp:  0.8 * canvas.canvas_bp(),
                bp:  0.6 * canvas.canvas_bp(),
            }
        }
        else {
            let base = units::SSW * canvas.canvas_bp();
            Units {
                dt: units::DT * canvas.canvas_bp(),
                sw: base,
                sh: 0.96 * base,
                ksw: base,
                ksh: 0.96 * base,
                ds: 0.05 * base,
                sp: 0.4 * canvas.canvas_bp(),
                bp: 0.4 * canvas.canvas_bp(),
            }
        }
    }
}


//------------ StandardMarker ------------------------------------------------

/// The rendering rule for a standard marker.
pub struct StandardMarker {
    /// Extra rotation in addition to whatever the position dictates.
    rotation: f64,

    /// The palette to use for rendering the symbol.
    palette: Palette,

    /// The index of the marker to use.
    marker: usize
}


impl StandardMarker {
    pub fn create(
        pos: ast::Pos, symbols: SymbolSet, err: &mut Error
    ) -> Result<Self, Failed> {
        let rotation = if symbols.contains("top") { 1.5 * PI }
                       else if symbols.contains("left") { PI }
                       else if symbols.contains("bottom") { 0.5 * PI }
                       else { 0. };
        for (index, marker) in MARKERS.iter().enumerate() {
            if symbols.contains(marker.0) {
                return Ok(StandardMarker {
                    rotation,
                    palette: Palette::from_symbols(&symbols),
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
        self.palette.stroke.apply(canvas);
        MARKERS[self.marker].1(canvas, Units::new(canvas));
        canvas.identity_matrix();
    }
}


//------------ Markers ------------------------------------------------------

const MARKERS: &[(&'static str, &'static dyn Fn(&Canvas, Units))] = &[
    ("de.abzw", &|canvas, u| {
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

    ("de.anst", &|canvas, u| {
        canvas.set_line_width(u.sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., 0.5 * u.sh);
        canvas.move_to(-0.5 * u.sw, u.sh);
        canvas.line_to(0., 0.5 * u.sh);
        canvas.line_to(0.5 * u.sw, u.sh);
        canvas.stroke();
    }),

    ("de.aw", &|canvas, u| {
        canvas.set_line_width(2. * u.sp);
        canvas.new_path();
        canvas.arc(0., 0.5 * u.sh, 0.5 * u.sh - u.sp, 0., 2. * PI);
        let seg = PI * (0.5 * u.sh - u.sp) / 6.;
        canvas.set_dash(&[seg, seg], 0.);
        canvas.stroke();
        canvas.set_dash(&[], 0.);
        canvas.set_line_width(u.sp);
        canvas.new_path();
        canvas.arc(0., 0.5 * u.sh, 0.5 * u.sh - u.sp, 0., 2. * PI);
        canvas.fill();
    }),

    ("de.awanst", &|canvas, u| {
        canvas.set_line_width(u.sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., u.sh);
        canvas.move_to(-0.5 * u.sw, u.sh);
        canvas.line_to(0., 0.5 * u.sh);
        canvas.line_to(0.5 * u.sw, u.sh);
        canvas.stroke();
    }),

    ("de.bbf", &|canvas, u| {
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, 0.);
        canvas.line_to(-0.5 * u.sw + 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, 0.);
        canvas.set_line_width(u.sp);
        canvas.stroke();
        canvas.move_to(0.5 * u.sw - 0.5 * u.sp, 0.);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, u.sw - 0.5 * u.sp);
        canvas.line_to(-0.5 * u.sw + 0.5 * u.sp, 0.);
        canvas.close_path();
        canvas.fill();
    }),

    ("de.bf", &|canvas, u| {
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

    ("de.bft", &|canvas, u| {
        canvas.move_to(-0.3 * u.sw + 0.5 * u.sp, 0.);
        canvas.line_to(-0.3 * u.sw + 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.3 * u.sw - 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.3 * u.sw - 0.5 * u.sp, 0.);
        canvas.set_line_width(u.sp);
        canvas.stroke();
        canvas.move_to(0.3 * u.sw - 0.5 * u.sp, 0.);
        canvas.line_to(0.3 * u.sw - 0.5 * u.sp, u.sw - 0.5 * u.sp);
        canvas.line_to(-0.3 * u.sw + 0.5 * u.sp, 0.);
        canvas.close_path();
        canvas.fill();
    }),

    ("de.bftp", &|canvas, u| {
        canvas.move_to(-0.3 * u.sw, 0.);
        canvas.line_to(-0.3 * u.sw, u.sh - u.ds);
        canvas.curve_to(
            -0.3 * u.sw, u.sh - 1.5 * u.ds,
            -0.3 * u.sw + 0.5 * u.ds, u.sh,
            -0.3 * u.sw + u.ds, u.sh
        );
        canvas.line_to(0.3 * u.sw - u.ds, u.sh);
        canvas.curve_to(
            0.3 * u.sw - 0.5 * u.ds, u.sh,
            0.3 * u.sw, u.sh - 0.5 * u.ds,
            0.3 * u.sw, u.sh - u.ds
        );
        canvas.line_to(0.3 * u.sw, 0.);
        canvas.close_path();
        canvas.fill();
    }),

    ("de.bk", &|canvas, u| {
        canvas.set_line_width(u.sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., u.sh);
        canvas.move_to(-0.5 * u.sw, 0.5 * u.sh);
        canvas.line_to(0.5 * u.sw, 0.5 * u.sh);
        canvas.stroke();
    }),

    ("de.bw", &|canvas, u| {
        canvas.set_line_width(2. * u.sp);
        canvas.new_path();
        canvas.arc(0., 0.5 * u.sh, 0.5 * u.sh - u.sp, 0., 2. * PI);
        let seg = PI * (0.5 * u.sh - u.sp) / 6.;
        canvas.set_dash(&[seg, seg], 0.);
        canvas.stroke();
        canvas.set_dash(&[], 0.);
        canvas.set_line_width(u.sp);
        canvas.new_path();
        canvas.arc(0., 0.5 * u.sh, 0.5 * u.sh - 1.5 * u.sp, 0., 2. * PI);
        canvas.stroke();
        canvas.arc(0., 0.5 * u.sh, 0.15 * u.sh, 0., 2. * PI);
        canvas.fill();
    }),

    ("de.dirgr", &|canvas, u| {
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

    ("de.dkst", &|canvas, u| {
        canvas.set_line_width(u.sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., 0.5 * u.sh);
        canvas.move_to(-0.5 * u.sw, 0.5 * u.sh);
        canvas.line_to(0.5 * u.sw, 0.5 * u.sh);
        canvas.move_to(-0.5 * u.sw, 0.5 * u.sh);
        canvas.line_to(0., u.sh);
        canvas.line_to(0.5 * u.sw, 0.5 * u.sh);
        canvas.stroke();
    }),

    ("de.est", &|canvas, u| {
        canvas.set_line_width(2. * u.sp);
        canvas.new_path();
        canvas.arc(0., 0.5 * u.sh, 0.5 * u.sh - u.sp, 0., 2. * PI);
        let seg = PI * (0.5 * u.sh - u.sp) / 6.;
        canvas.set_dash(&[seg, seg], 0.);
        canvas.stroke();
        canvas.set_dash(&[], 0.);
        canvas.set_line_width(u.sp);
        canvas.new_path();
        canvas.arc(0., 0.5 * u.sh, 0.5 * u.sh - 1.5 * u.sp, 0., 2. * PI);
        canvas.stroke();
    }),

    ("de.hp", &|canvas, u| {
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, 0.);
        canvas.line_to(-0.5 * u.sw + 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, 0.);
        canvas.set_line_width(u.sp);
        canvas.stroke();
    }),

    ("de.hpext", &|canvas, u| {
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, 0.);
        canvas.line_to(-0.5 * u.sw + 0.5 * u.sp, u.dt);
        canvas.move_to(0.5 * u.sw - 0.5 * u.sp, u.dt);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, 0.);
        canvas.set_line_width(u.sp);
        canvas.stroke();
    }),

    ("de.hst", &|canvas, u| {
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, 0.);
        canvas.line_to(-0.5 * u.sw + 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, 0.);
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, u.sh);
        canvas.line_to(0., 0.);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, u.sh);
        canvas.set_line_width(u.sp);
        canvas.stroke();
    }),

    ("de.kabzw", &|canvas, u| {
        canvas.set_line_width(u.sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., 0.5 * u.ksh);
        canvas.move_to(-0.5 * u.ksw, 0.5 * u.ksh);
        canvas.line_to(0.5 * u.ksw, 0.5 * u.ksh);
        canvas.move_to(-0.5 * u.ksw, u.ksh);
        canvas.line_to(0., 0.5 * u.ksh);
        canvas.line_to(0.5 * u.ksw, u.ksh);
        canvas.stroke();
    }),

    ("de.kbf", &|canvas, u| {
        canvas.move_to(-0.5 * u.ksw, 0.);
        canvas.line_to(-0.5 * u.ksw, u.ksh - u.ds);
        canvas.curve_to(
            -0.5 * u.ksw, u.ksh - 1.5 * u.ds,
            -0.5 * u.ksw + 0.5 * u.ds, u.ksh,
            -0.5 * u.ksw + u.ds, u.ksh
        );
        canvas.line_to(0.5 * u.ksw - u.ds, u.ksh);
        canvas.curve_to(
            0.5 * u.ksw - 0.5 * u.ds, u.ksh,
            0.5 * u.ksw, u.ksh - 0.5 * u.ds,
            0.5 * u.ksw, u.ksh - u.ds
        );
        canvas.line_to(0.5 * u.ksw, 0.);
        canvas.close_path();
        canvas.fill();
    }),

    ("de.khp", &|canvas, u| {
        canvas.move_to(-0.5 * u.ksw + 0.5 * u.sp, 0.);
        canvas.line_to(-0.5 * u.ksw + 0.5 * u.sp, u.ksh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.ksw - 0.5 * u.sp, u.ksh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.ksw - 0.5 * u.sp, 0.);
        canvas.set_line_width(u.sp);
        canvas.stroke();
    }),

    ("de.kzst", &|canvas, u| {
        canvas.move_to(-0.5 * u.ksw + 0.5 * u.sp, 0.);
        canvas.line_to(-0.5 * u.ksw + 0.5 * u.sp, u.ksh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.ksw - 0.5 * u.sp, u.ksh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.ksw - 0.5 * u.sp, 0.);
        canvas.move_to(-0.5 * u.ksw + 0.5 * u.sp, 0.5 * u.sp);
        canvas.set_line_width(u.sp);
        canvas.stroke();
        canvas.move_to(-0.5 * u.ksw + 0.5 * u.sp, u.ksh - 0.5 * u.sp);
        canvas.line_to(0., 0.3 * u.ksh);
        canvas.line_to(0.5 * u.ksw - 0.5 * u.sp, u.ksh - 0.5 * u.sp);
        canvas.close_path();
        canvas.fill();
    }),

    ("de.ldst", &|canvas, u| {
        canvas.set_line_width(u.sp);
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, 0.);
        canvas.line_to(0., u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, 0.);
        canvas.stroke();
    }),

    ("de.stw", &|canvas, u| {
        canvas.set_line_width(u.sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., 2. * u.dt);
        canvas.move_to(-u.dt, u.dt);
        canvas.line_to(u.dt, u.dt);
        canvas.stroke();
    }),

    ("de.uest", &|canvas, u| {
        canvas.set_line_width(u.sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., 0.5 * u.sh);
        canvas.move_to(-0.5 * u.sw, 0.5 * u.sh);
        canvas.line_to(0.5 * u.sw, 0.5 * u.sh);
        canvas.move_to(-0.25 * u.sw, 0.5 * u.sh);
        canvas.line_to(-0.25 * u.sw, u.sh);
        canvas.move_to(0.25 * u.sw, 0.5 * u.sh);
        canvas.line_to(0.25 * u.sw, u.sh);
        canvas.stroke();
    }),

    ("de.zst", &|canvas, u| {
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, 0.);
        canvas.line_to(-0.5 * u.sw + 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, 0.);
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, 0.5 * u.sp);
        canvas.set_line_width(u.sp);
        canvas.stroke();
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0., 0.3 * u.sh);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.close_path();
        canvas.fill();
    }),


    ("statdt", &|canvas, u| {
        canvas.set_line_width(u.bp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., u.dt);
        canvas.stroke();
    }),
];

