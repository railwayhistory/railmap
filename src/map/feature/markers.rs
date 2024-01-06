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

use std::collections::HashMap;
use std::f64::consts::PI;
use femtomap::path::Position;
use femtomap::render::{
    Canvas, Color, DashPattern, Group, LineCap, Matrix, Operator
};
use kurbo::Rect;
use lazy_static::lazy_static;
use crate::import::eval;
use crate::import::Failed;
use crate::theme::{Style as _};
use super::super::class::Class;
use super::super::style::{Dimensions, Style};
use super::Shape;


const CASING_COLOR: Color = Color::rgba(1., 1., 1., 0.7);


//------------ StandardMarker ------------------------------------------------

/// The rendering rule for a standard marker.
pub struct StandardMarker {
    /// The position the marker is attached to.
    position: Position,

    /// Orientation of the marker.
    ///
    /// If this in `None` the marker doesn’t need to be oriented at all.
    /// Otherwise the value is the angle to be added to rotation from the
    /// position.
    orientation: f64,

    /// The feature class.
    class: Class,

    /// The marker to use.
    marker: Marker,
}


impl StandardMarker {
    pub fn from_arg(
        mut symbols: eval::SymbolSet,
        position: Position,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let orientation = Self::rotation_from_symbols(&mut symbols, err)?;
        let class = Class::from_symbols(&mut symbols);
        let pos = symbols.pos();
        let marker = match symbols.take_final(err)? {
            Some(marker) => marker,
            None => {
                err.add(pos, "missing marker");
                return Err(Failed)
            }
        };
        let marker = match MARKERS.get(marker.as_str()) {
            Some(marker) => *marker,
            None => {
                err.add(pos, "missing marker");
                return Err(Failed)
            }
        };
        Ok(StandardMarker { position, orientation, class, marker })
    }

    fn rotation_from_symbols(
        symbols: &mut eval::SymbolSet,
        _err: &mut eval::Error
    ) -> Result<f64, Failed> {
        if symbols.take("top") {
            Ok(1.5 * PI)
        }
        else if symbols.take("left") {
            Ok(PI)
        }
        else if symbols.take("bottom") {
            Ok(0.5 * PI)
        }
        else if symbols.take("right") {
            Ok(0.)
        }
        else {
            Ok(0.)
            /*
            err.add(pos, "missing orientation");
            Err(Failed)
                */
        }
    }

    pub fn class(&self) -> &Class {
        &self.class
    }

    pub fn storage_bounds(&self) -> Rect {
        self.position.storage_bounds()
    }

    pub fn shape(
        &self, style: &Style, canvas: &Canvas
    ) -> Box<dyn Shape + '_> {
        Box::new(|style: &Style, canvas: &mut Canvas| {
            self.render(style, canvas)
        })
    }

    fn render(&self, style: &Style, canvas: &mut Canvas) {
        let mut canvas = canvas.sketch().into_group();
        let (point, angle) = self.position.resolve(style);
        canvas.apply(
            Matrix::identity().translate(
                point
            ).rotate(angle + self.orientation)
        );
        canvas.apply(style.primary_marker_color(&self.class));
        if style.detail() >= 4. {
            (self.marker.large)(&mut canvas, style.dimensions())
        }
        else {
            (self.marker.small)(&mut canvas, style.dimensions())
        }
    }
}


//------------ Marker --------------------------------------------------------

#[derive(Clone, Copy)]
struct Marker {
    large: RenderFn,
    small: RenderFn,
}

type RenderFn = &'static (
    dyn Fn(&mut Group, Dimensions) + Sync
);


macro_rules! markers {
    (
        $(
            ( $( $name:expr ),* ) => ( $( $closure:expr ),* )
        ),*
    )
    => {
        lazy_static! {
            static ref MARKERS: HashMap<&'static str, Marker> = {
                let mut set = HashMap::new();
                $(
                    let marker = make_marker!( $( $closure, )* );
                    $(
                        set.insert($name, marker);
                    )*
                )*
                set
            };
        }
    }
}

macro_rules! make_marker {
    ( $large:expr, ) => {
        Marker { large: &$large, small: &$large }
    };
    ( $large:expr, $small:expr, ) => {
        Marker { large: &$large, small: &$small }
    }
}


//------------ Actual Markers ------------------------------------------------

markers! {
    ("de.abzw", "junction") => (
        |canvas: &mut Group, u: Dimensions| {
            chevron(canvas,
                0.5 * u.sw - 0.5 * u.sp,
                0.25 * u.sh + 0.5 * u.sp, 0.75 * u.sh - 0.5 * u.sp
            );
            chevron(canvas,
                0.5 * u.sw - 0.5 * u.sp,
                0.5 * u.sh + 0.5 * u.sp, 1.0 * u.sh - 0.5 * u.sp
            );
            canvas.move_to(0., 0.);
            canvas.line_to(0., 0.25 * u.sh);
            canvas.move_to(0., 0.5 * u.sh + u.sp);
            canvas.line_to(0., u.sh - 0.5 * u.sp);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        },
        |canvas: &mut Group, u: Dimensions| {
            chevron(canvas, 0.4 * u.sw, 0.5 * u.sp, u.sh);
            canvas.fill()
        }
    ),
    ("de.abzw.casing", "junction.casing") => (
        |canvas: &mut Group, u: Dimensions| {
            chevron(canvas,
                0.5 * u.sw - 0.5 * u.sp,
                0.25 * u.sh + 0.5 * u.sp, 0.75 * u.sh - 0.5 * u.sp
            );
            chevron(canvas,
                0.5 * u.sw - 0.5 * u.sp,
                0.5 * u.sh + 0.5 * u.sp, 1.0 * u.sh - 0.5 * u.sp
            );
            canvas.move_to(0., 0.);
            canvas.line_to(0., 0.25 * u.sh);
            canvas.move_to(0., 0.5 * u.sh + u.sp);
            canvas.line_to(0., u.sh - 0.5 * u.sp);
            canvas.apply_line_width(u.csp);
            canvas.apply(CASING_COLOR);
            //canvas.apply(Operator::DestinationOut);
            stroke_round(canvas);
        },
        |canvas: &mut Group, u: Dimensions| {
            junction_small_casing(canvas, u)
        }
    ),
    ("de.abzw.first", "junction.first") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, 0.75 * u.sh - 0.5 * u.sp);
            canvas.line_to(0., 0.25 * u.sh + 0.5 * u.sp);
            canvas.line_to(0., 0.);
            canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, 1.0 * u.sh - 0.5 * u.sp);
            canvas.line_to(0., 0.5 * u.sh + 0.5 * u.sp);
            canvas.line_to(0., 1.0 * u.sh - 0.5 * u.sp);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        }
    ),
    ("de.abzw.second", "junction.second") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.move_to(0.5 * u.sw - 0.5 * u.sp, 0.75 * u.sh - 0.5 * u.sp);
            canvas.line_to(0., 0.25 * u.sh + 0.5 * u.sp);
            canvas.line_to(0., 0.);
            canvas.move_to(0.5 * u.sw - 0.5 * u.sp, 1.0 * u.sh - 0.5 * u.sp);
            canvas.line_to(0., 0.5 * u.sh + 0.5 * u.sp);
            canvas.line_to(0., 1.0 * u.sh - 0.5 * u.sp);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        }
    ),

    ("de.anst") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.move_to(0., 0.);
            canvas.line_to(0., u.sh - 0.5 * u.sp);
            canvas.move_to(-0.3 * u.sw, u.sh - 0.5 * u.sp);
            canvas.line_to(0.3 * u.sw, u.sh - 0.5 * u.sp);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        },
        |canvas: &mut Group, u: Dimensions| {
            canvas.move_to(0., 0.);
            canvas.line_to(0., u.sh - 0.75 * u.sp);
            canvas.move_to(-0.3 * u.sw, u.sh - 0.75 * u.sp);
            canvas.line_to(0.3 * u.sw, u.sh - 0.75 * u.sp);
            canvas.apply_line_width(1.5 * u.sp);
            stroke_round(canvas)
        }
    ),

    ("de.aw") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.apply_line_width(2. * u.sp);
            canvas.new_path();
            canvas.arc(0., 0.5 * u.sh, 0.5 * u.sh - u.sp, 0., 2. * PI);
            let seg = PI * (0.5 * u.sh - u.sp) / 6.;
            canvas.apply(DashPattern::new([seg, seg], 0.));
            canvas.stroke();
            canvas.apply(DashPattern::empty());
            canvas.apply_line_width(u.sp);
            canvas.new_path();
            canvas.arc(0., 0.5 * u.sh, 0.5 * u.sh - u.sp, 0., 2. * PI);
            canvas.fill()
        }
    ),

    ("de.awanst") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.move_to(0., 0.);
            canvas.line_to(0., u.sh - 0.5 * u.sp);

            canvas.move_to(-0.3 * u.sw, 0.7 * u.sh - 0.5 * u.sp);
            canvas.line_to(0.3 * u.sw, 0.7 * u.sh - 0.5 * u.sp);
            canvas.move_to(-0.3 * u.sw, u.sh - 0.5 * u.sp);
            canvas.line_to(0.3 * u.sw, u.sh - 0.5 * u.sp);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        },
        |canvas: &mut Group, u: Dimensions| {
            canvas.move_to(0., 0.);
            canvas.line_to(0., u.sh - 0.75 * u.sp);
            canvas.move_to(-0.3 * u.sw, u.sh - 0.75 * u.sp);
            canvas.line_to(0.3 * u.sw, u.sh - 0.75 * u.sp);
            canvas.apply_line_width(1.5 * u.sp);
            stroke_round(canvas)
        }
    ),

    ("de.bbf", "servicestation") => (
        |canvas: &mut Group, u: Dimensions| {
            let hsp = 0.5 * u.sp;
            canvas.move_to(-0.5 * u.sw + hsp, 2.5 * u.sp);
            canvas.line_to(-0.5 * u.sw + hsp, u.sh - 0.5 * u.sp);
            canvas.line_to(0.5 * u.sw - hsp, u.sh - 0.5 * u.sp);
            canvas.line_to(0.5 * u.sw - hsp, 2.5 * u.sp);
            canvas.close_path();
            canvas.apply_line_width(u.sp);
            canvas.stroke();

            canvas.new_path();
            canvas.move_to(-0.5 * u.sw + hsp, 2.5 * u.sp);
            canvas.line_to(-0.5 * u.sw + hsp, u.sh - 0.5 * u.sp);
            canvas.line_to(0.5 * u.sw - hsp, 2.5 * u.sp);
            canvas.close_path();
            canvas.fill()
        },
        |canvas: &mut Group, u: Dimensions| {
            stop_small(canvas, u);

            let hsp = 0.5 * u.sp;
            canvas.move_to(-0.5 * u.sw + hsp, 1.75 * u.sp);
            canvas.line_to(-0.5 * u.sw + hsp, u.sh - hsp);
            canvas.line_to(0.5 * u.sw, 1.75 * u.sp);
            canvas.close_path();
            canvas.fill()
        }
    ),

    ("de.bf", "de.kbf", "station") => (
        |canvas: &mut Group, u: Dimensions| {
            let hsp = 0.5 * u.sp;
            canvas.move_to(-0.5 * u.sw + hsp, 2.5 * u.sp);
            canvas.line_to(-0.5 * u.sw + hsp, u.sh - hsp);
            canvas.line_to(0.5 * u.sw - hsp, u.sh - hsp);
            canvas.line_to(0.5 * u.sw - hsp, 2.5 * u.sp);
            canvas.close_path();
            canvas.apply_line_width(u.sp);
            canvas.stroke();

            let hsp = 2. * u.sp;
            canvas.new_path();
            canvas.move_to(-0.5 * u.sw + hsp, 2.5 * u.sp);
            canvas.line_to(-0.5 * u.sw + hsp, u.sh - hsp);
            canvas.line_to(0.5 * u.sw - hsp, u.sh - hsp);
            canvas.line_to(0.5 * u.sw - hsp, 2.5 * u.sp);
            canvas.close_path();
            canvas.fill()
        },
        |canvas: &mut Group, u: Dimensions| {
            station_small(canvas, u)
        }
    ),
    ("de.kbf") => (
        |canvas: &mut Group, u: Dimensions| {
            let hsp = 0.5 * u.sp;
            canvas.move_to(-0.5 * u.sw + hsp, 2.5 * u.sp);
            canvas.line_to(-0.5 * u.sw + hsp, u.sh - hsp);
            canvas.line_to(0.5 * u.sw - hsp, u.sh - hsp);
            canvas.line_to(0.5 * u.sw - hsp, 2.5 * u.sp);
            canvas.close_path();
            canvas.apply_line_width(u.sp);
            canvas.stroke();

            let hsp = 2. * u.sp;
            canvas.new_path();
            canvas.move_to(-0.5 * u.sw + hsp, 2.5 * u.sp);
            canvas.line_to(-0.5 * u.sw + hsp, u.sh - hsp);
            canvas.line_to(0.5 * u.sw - hsp, u.sh - hsp);
            canvas.line_to(0.5 * u.sw - hsp, 2.5 * u.sp);
            canvas.close_path();
            canvas.fill()
        },
        |canvas: &mut Group, u: Dimensions| {
            station_xsmall(canvas, u)
        }
    ),
    ("de.bf.casing", "station.casing") => (
        |canvas: &mut Group, u: Dimensions| {
            station_casing(canvas, u)
        },
        |canvas: &mut Group, u: Dimensions| {
            station_small_casing(canvas, u)
        }
    ),
    ("de.bf.first", "station.first") => (
        |canvas: &mut Group, u: Dimensions| {
            let hsp = 0.5 * u.sp;
            canvas.move_to(-0.5 * u.sw + hsp, 2.5 * u.sp);
            canvas.line_to(-0.5 * u.sw + hsp, u.sh - hsp);
            canvas.line_to(0., u.sh - hsp);
            canvas.line_to(0., 2.5 * u.sp);
            canvas.close_path();
            canvas.apply_line_width(u.sp);
            canvas.stroke();

            let hsp = 2. * u.sp;
            canvas.new_path();
            canvas.move_to(-0.5 * u.sw + hsp, 2.5 * u.sp);
            canvas.line_to(-0.5 * u.sw + hsp, u.sh - hsp);
            canvas.line_to(0., u.sh - hsp);
            canvas.line_to(0., 2.5 * u.sp);
            canvas.close_path();
            canvas.fill()
        }
    ),
    ("de.bf.second", "station.second") => (
        |canvas: &mut Group, u: Dimensions| {
            let hsp = 0.5 * u.sp;
            canvas.move_to(0.5 * u.sw - hsp, 2.5 * u.sp);
            canvas.line_to(0.5 * u.sw - hsp, u.sh - hsp);
            canvas.line_to(0., u.sh - hsp);
            canvas.line_to(0., 2.5 * u.sp);
            canvas.close_path();
            canvas.apply_line_width(u.sp);
            canvas.stroke();

            let hsp = 2. * u.sp;
            canvas.new_path();
            canvas.move_to(0.5 * u.sw - hsp, 2.5 * u.sp);
            canvas.line_to(0.5 * u.sw - hsp, u.sh - hsp);
            canvas.line_to(0., u.sh - hsp);
            canvas.line_to(0., 2.5 * u.sp);
            canvas.close_path();
            canvas.fill()
        }
    ),

    ("de.bft") => (
        |canvas: &mut Group, u: Dimensions| {
            chevron(canvas,
                0.5 * u.sw - 0.5 * u.sp,
                0.25 * u.sh + 0.5 * u.sp, 0.75 * u.sh - 0.5 * u.sp
            );
            canvas.move_to(0., 0.);
            canvas.line_to(0., 0.25 * u.sh);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas);
            canvas.new_path();
            chevron(canvas,
                0.5 * u.sw,
                0.5 * u.sh, 1.0 * u.sh
            );
            canvas.close_path();
            canvas.fill();
        }
    ),
    ("de.bft.casing") => (
        |canvas: &mut Group, u: Dimensions| {
            chevron(canvas,
                0.5 * u.sw - 0.5 * u.sp,
                0.25 * u.sh + 0.5 * u.sp, 0.75 * u.sh - 0.5 * u.sp
            );
            chevron(canvas,
                0.5 * u.sw - 0.5 * u.sp,
                0.5 * u.sh + 0.5 * u.sp, 1.0 * u.sh - 0.5 * u.sp
            );
            canvas.move_to(0., 0.);
            canvas.line_to(0., 0.25 * u.sh);
            canvas.move_to(0., 0.5 * u.sh + u.sp);
            canvas.line_to(0., u.sh - 0.5 * u.sp);
            canvas.apply_line_width(u.csp);
            canvas.apply(CASING_COLOR);
            stroke_round(canvas)
        }
    ),
    ("de.bft.first") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.move_to(0., 0.);
            canvas.line_to(0., 0.25 * u.sh);
            canvas.line_to(-0.5 * u.sw + 0.5 * u.sp, 0.75 * u.sh - 0.5 * u.sp);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas);

            canvas.new_path();
            canvas.move_to(0., 0.5 * u.sh);
            canvas.line_to(0., 1.0 * u.sh);
            canvas.line_to(-0.5 * u.sw, 1.0 * u.sh);
            canvas.close_path();
            canvas.fill()
        }
    ),
    ("de.bft.second") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.move_to(0., 0.);
            canvas.line_to(0., 0.25 * u.sh);
            canvas.line_to(0.5 * u.sw - 0.5 * u.sp, 0.75 * u.sh - 0.5 * u.sp);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas);

            canvas.new_path();
            canvas.move_to(0., 0.5 * u.sh);
            canvas.line_to(0., 1.0 * u.sh);
            canvas.line_to(0.5 * u.sw, 1.0 * u.sh);
            canvas.close_path();
            canvas.fill()
        }
    ),

    ("de.bk", "block") => (
        |canvas: &mut Group, u: Dimensions| {
            chevron(canvas,
                0.5 * u.sw - 0.5 * u.sp,
                0.3 * u.sh + 0.5 * u.sp, 0.8 * u.sh - 0.5 * u.sp
            );
            canvas.move_to(0., 0.);
            canvas.line_to(0., 0.3 * u.sh);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        },
        |canvas: &mut Group, u: Dimensions| {
            chevron(canvas,
                0.4 * u.sw - 0.5 * u.sp, 0.5 * u.sp, u.sh - 0.5 * u.sp,
            );
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        }
    ),
    ("de.bk.casing", "block.casing") => (
        |canvas: &mut Group, u: Dimensions| {
            chevron(canvas,
                0.5 * u.sw - 0.5 * u.sp,
                0.3 * u.sh + 0.5 * u.sp, 0.8 * u.sh - 0.5 * u.sp
            );
            canvas.move_to(0., 0.);
            canvas.line_to(0., 0.3 * u.sh);
            canvas.apply_line_width(u.csp);
            stroke_round(canvas)
        },
        |canvas: &mut Group, u: Dimensions| {
            chevron(canvas,
                0.4 * u.sw - 0.5 * u.sp, 0.5 * u.sp, u.sh - 0.5 * u.sp,
            );
            canvas.apply_line_width(u.csp);
            stroke_round(canvas)
        }
    ),
    ("de.bk.first", "block.first") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.move_to(0., 0.);
            canvas.line_to(0., 0.3 * u.sh);
            canvas.line_to(-0.5 * u.sw + 0.5 * u.sp, 0.8 * u.sh - 0.5 * u.sp);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        }
    ),
    ("de.bk.second", "block.second") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.move_to(0., 0.);
            canvas.line_to(0., 0.3 * u.sh);
            canvas.line_to(0.5 * u.sw - 0.5 * u.sp, 0.8 * u.sh - 0.5 * u.sp);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        }
    ),

    ("de.bw") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.apply_line_width(2. * u.sp);
            canvas.new_path();
            canvas.arc(0., 0.5 * u.sh, 0.5 * u.sh - u.sp, 0., 2. * PI);
            let seg = PI * (0.5 * u.sh - u.sp) / 6.;
            canvas.apply(DashPattern::new([seg, seg], 0.));
            canvas.stroke();
            canvas.apply(DashPattern::empty());
            canvas.apply_line_width(u.sp);
            canvas.new_path();
            canvas.arc(0., 0.5 * u.sh, 0.5 * u.sh - 1.5 * u.sp, 0., 2. * PI);
            canvas.stroke();
            canvas.arc(0., 0.5 * u.sh, 0.15 * u.sh, 0., 2. * PI);
            canvas.fill()
        }
    ),

    ("de.dirgr") => (
        |canvas: &mut Group, u: Dimensions| {
            let r = 0.8 * u.dt;
            canvas.apply_line_width(u.bp);
            canvas.move_to(0., -0.5 * u.dt);
            canvas.line_to(0., 2. * r);
            canvas.stroke();
            canvas.arc(0., 3. * r, r, 0., 2. * PI);
            canvas.stroke();
            canvas.arc(0., 3. * r, 0.5 * r, 0., 2. * PI);
            canvas.fill()
        },
        |canvas: &mut Group, u: Dimensions| {
            let r = 0.25 * u.sh;
            canvas.arc(0., 3. * r, r, 0., 2. * PI);
            canvas.fill();
            canvas.apply_line_width(u.bp);
            canvas.move_to(0., -0.5 * u.dt);
            canvas.line_to(0., 2. * r);
            canvas.stroke();
        }
    ),

    ("de.dkst") => (
        |canvas: &mut Group, u: Dimensions| {
            chevron(canvas,
                0.5 * u.sw - 0.5 * u.sp,
                0.3 * u.sh + 0.5 * u.sp, 0.8 * u.sh - 0.5 * u.sp
            );
            canvas.move_to(0., 0.);
            canvas.line_to(0., u.sh);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        },
        |canvas: &mut Group, u: Dimensions| {
            chevron(canvas,
                0.4 * u.sw - 0.5 * u.sp, 0.5 * u.sp, u.sh - 0.5 * u.sp,
            );
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        }
    ),

    ("de.est") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.apply_line_width(2. * u.sp);
            canvas.new_path();
            canvas.arc(0., 0.5 * u.sh, 0.5 * u.sh - u.sp, 0., 2. * PI);
            let seg = PI * (0.5 * u.sh - u.sp) / 6.;
            canvas.apply(DashPattern::new([seg, seg], 0.));
            canvas.stroke();
            canvas.apply(DashPattern::empty());
            canvas.apply_line_width(u.sp);
            canvas.new_path();
            canvas.arc(0., 0.5 * u.sh, 0.5 * u.sh - 1.5 * u.sp, 0., 2. * PI);
            canvas.stroke()
        }
    ),

    ("de.exbf") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, u.sh + 1.5 * u.sp);
            canvas.line_to(0.5 * u.sw - 0.5 * u.sp, u.sh + 1.5 * u.sp);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        }
    ),

    ("de.gbf", "goodsstation") => (
        |canvas: &mut Group, u: Dimensions| {
            let hsp = 0.5 * u.sp;
            canvas.move_to(-0.5 * u.sw + hsp, 2.5 * u.sp);
            canvas.line_to(-0.5 * u.sw + hsp, 0.7 * u.sh);
            canvas.line_to(0., u.sh - hsp);
            canvas.line_to(0.5 * u.sw - hsp, 0.7 * u.sh);
            canvas.line_to(0.5 * u.sw - hsp, 2.5 * u.sp);
            canvas.close_path();
            canvas.apply_line_width(u.sp);
            canvas.stroke();

            canvas.new_path();
            let hsp = 2. * u.sp;
            canvas.move_to(-0.5 * u.sw + hsp, 2.5 * u.sp);
            canvas.line_to(-0.5 * u.sw + hsp, 0.8 * u.sh - 1.5 * u.sp);
            canvas.line_to(0., u.sh - 2. * u.sp);
            canvas.line_to(0.5 * u.sw - hsp, 0.8 * u.sh -  1.5 * u.sp);
            canvas.line_to(0.5 * u.sw - hsp, 2.5 * u.sp);
            canvas.close_path();
            canvas.fill()
        },
        |canvas: &mut Group, u: Dimensions| {
            let hsp = 0.5 * u.sp;
            canvas.move_to(-0.5 * u.sw, 1.75 * u.sp - hsp);
            canvas.line_to(-0.5 * u.sw, 0.7 * u.sh);
            canvas.line_to(0., u.sh);
            canvas.line_to(0.5 * u.sw, 0.7 * u.sh);
            canvas.line_to(0.5 * u.sw, 1.75 * u.sp - hsp);
            canvas.close_path();
            canvas.fill()
        }
    ),

    ("de.hp", "de.khp", "stop") => (
        |canvas: &mut Group, u: Dimensions| {
            stop(canvas, u)
        },
        |canvas: &mut Group, u: Dimensions| {
            stop_small(canvas, u)
        }
    ),
    ("de.khp") => (
        |canvas: &mut Group, u: Dimensions| {
            stop(canvas, u)
        },
        |canvas: &mut Group, u: Dimensions| {
            stop_xsmall(canvas, u)
        }
    ),
    ("de.hp.casing", "stop.casing") => (
        |canvas: &mut Group, u: Dimensions| {
            station_casing(canvas, u)
        },
        |canvas: &mut Group, u: Dimensions| {
            station_small_casing(canvas, u)
        }
    ),
    ("de.hp.first", "stop.first") => (
        |canvas: &mut Group, u: Dimensions| {
            let hsp = 0.5 * u.sp;
            canvas.move_to(-0.5 * u.sw + hsp, 2.5 * u.sp);
            canvas.line_to(-0.5 * u.sw + hsp, u.sh - 0.5 * u.sp);
            canvas.line_to(0., u.sh - 0.5 * u.sp);
            canvas.line_to(0., 2.5 * u.sp);
            canvas.close_path();
            canvas.apply_line_width(u.sp);
            canvas.stroke()
        }
    ),
    ("de.hp.second", "stop.second") => (
        |canvas: &mut Group, u: Dimensions| {
            let hsp = 0.5 * u.sp;
            canvas.move_to(0.5 * u.sw - hsp, 2.5 * u.sp);
            canvas.line_to(0.5 * u.sw - hsp, u.sh - 0.5 * u.sp);
            canvas.line_to(0., u.sh - 0.5 * u.sp);
            canvas.line_to(0., 2.5 * u.sp);
            canvas.close_path();
            canvas.apply_line_width(u.sp);
            canvas.stroke()
         }
    ),

    ("de.hp.bft") => (
        |canvas: &mut Group, u: Dimensions| {
            stop(canvas, u);
            chevron(canvas,
                0.5 * u.sw - 0.5 * u.sp,
                0.25 * u.sh + 0.5 * u.sp, 0.75 * u.sh - 0.5 * u.sp
            );
            canvas.apply_line_width(u.sp);
            stroke_round(canvas);
            chevron(canvas,
                0.5 * u.sw,
                0.5 * u.sh, 1.0 * u.sh
            );
            canvas.close_path();
            canvas.fill()
        }
    ),

    ("de.hst") => (
        |canvas: &mut Group, u: Dimensions| {
            let hsp = 0.5 * u.sp;
            canvas.move_to(-0.5 * u.sw + hsp, 2.5 * u.sp);
            canvas.line_to(-0.5 * u.sw + hsp, u.sh - 0.5 * u.sp);
            canvas.line_to(0.5 * u.sw - hsp, u.sh - 0.5 * u.sp);
            canvas.line_to(0.5 * u.sw - hsp, 2.5 * u.sp);
            canvas.close_path();
            canvas.move_to(0., 2.5 * u.sp);
            canvas.line_to(0., u.sh - 0.5 * u.sp);
            canvas.close_path();
            canvas.apply_line_width(u.sp);
            canvas.stroke()
        },
        |canvas: &mut Group, u: Dimensions| {
            stop_small(canvas, u);
            canvas.move_to(0., 1.75 * u.sp);
            canvas.line_to(0., u.sh - 0.5 * u.sp);
            canvas.apply_line_width(0.8 * u.sp);
            canvas.stroke()
        }
    ),

    ("de.inbf") => (
        |canvas: &mut Group, u: Dimensions| {
            let hsp = 0.5 * u.sp;
            canvas.move_to(-0.5 * u.sw + hsp, 0.5 * u.dt + hsp);
            canvas.line_to(-0.5 * u.sw + hsp, 2.5 * u.dt - hsp);
            canvas.line_to(0.5 * u.sw - hsp, 2.5 * u.dt - hsp);
            canvas.line_to(0.5 * u.sw - hsp, 0.5 * u.dt + hsp);
            canvas.close_path();
            canvas.stroke();
            canvas.new_path();
            let hsp = 2. * u.sp;
            canvas.move_to(-0.5 * u.sw + hsp, 1.1 * u.dt);
            canvas.line_to(-0.5 * u.sw + hsp, 1.9 * u.dt);
            canvas.line_to(0.5 * u.sw - hsp, 1.9 * u.dt);
            canvas.line_to(0.5 * u.sw - hsp, 1.1 * u.dt);
            canvas.close_path();
            canvas.fill()
        },
        |canvas: &mut Group, u: Dimensions| {
            let hsp = 0.5 * u.sp;
            canvas.move_to(-0.5 * u.sw, 1.75 * u.sp - hsp);
            canvas.line_to(-0.5 * u.sw, u.sh - 1.75 * u.sp + hsp);
            canvas.line_to(0.5 * u.sw, u.sh - 1.75 * u.sp + hsp);
            canvas.line_to(0.5 * u.sw, 1.75 * u.sp - hsp);
            canvas.close_path();
            canvas.fill();
        }
    ),

    ("de.ldst") => (
        |canvas: &mut Group, u: Dimensions| {
            let hsp = 0.5 * u.sp;
            canvas.move_to(-0.5 * u.sw + hsp, 2.5 * u.sp);
            canvas.line_to(-0.5 * u.sw + hsp, 0.7 * u.sh);
            canvas.line_to(0., u.sh - hsp);
            canvas.line_to(0.5 * u.sw + hsp, 0.7 * u.sh);
            canvas.line_to(0.5 * u.sw + hsp, 2.5 * u.sp);
            canvas.close_path();
            canvas.apply_line_width(u.sp);
            canvas.stroke()
        },
        |canvas: &mut Group, u: Dimensions| {
            let hsp = 0.5 * u.sp;
            canvas.move_to(-0.5 * u.sw + hsp, 1.75 * u.sp);
            canvas.line_to(-0.5 * u.sw + hsp, 0.6 * u.sh);
            canvas.line_to(0., u.sh - hsp);
            canvas.line_to(0.5 * u.sw - hsp, 0.6 * u.sh);
            canvas.line_to(0.5 * u.sw - hsp, 1.75 * u.sp);
            canvas.close_path();
            canvas.apply_line_width(u.sp);
            canvas.stroke()
        }
    ),

    ("de.lgr") => (
        |canvas: &mut Group, u: Dimensions| {
            let r = 0.8 * u.dt;
            canvas.arc(0., 3. * r, r, 0., 2. * PI);
            canvas.fill();
            canvas.apply_line_width(u.bp);
            canvas.move_to(0., -0.5 * u.dt);
            canvas.line_to(0., 2. * r);
            canvas.stroke();
        },
        |canvas: &mut Group, u: Dimensions| {
            let r = 0.25 * u.sh;
            canvas.arc(0., 3. * r, r, 0., 2. * PI);
            canvas.fill();
            canvas.apply_line_width(u.bp);
            canvas.move_to(0., -0.5 * u.dt);
            canvas.line_to(0., 2. * r);
            canvas.stroke();
        }
    ),

    ("de.stw", "signalbox") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.apply_line_width(u.sp);
            canvas.move_to(0., 0.);
            canvas.line_to(0., 2. * u.dt);
            canvas.move_to(-u.dt, u.dt);
            canvas.line_to(u.dt, u.dt);
            canvas.stroke()
        }
    ),

    ("de.stw.casing", "signalbox.casing") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.apply_line_width(1.5 * u.sp);
            canvas.move_to(0., 0.);
            canvas.line_to(0., 2. * u.dt);
            canvas.move_to(-u.dt, u.dt);
            canvas.line_to(u.dt, u.dt);
            canvas.apply(Operator::DestinationOut);
            canvas.stroke();
        }
    ),

    ("de.uest", "crossover") => (
        |canvas: &mut Group, u: Dimensions| {
            chevron(canvas,
                0.5 * u.sw - 0.5 * u.sp,
                0.3 * u.sh + 0.5 * u.sp, 0.8 * u.sh - 0.5 * u.sp
            );
            canvas.move_to(0., 0.);
            canvas.line_to(0., u.sh - u.sp);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        },
        |canvas: &mut Group, u: Dimensions| {
            chevron(canvas,
                0.4 * u.sw - 0.5 * u.sp, 0.5 * u.sp, u.sh - 0.5 * u.sp,
            );
            canvas.line_to(-0.4 * u.sw + 0.5 * u.sp, u.sh - 0.5 * u.sp);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        }
    ),
    ("de.uest.casing", "crossover.casing") => (
        |canvas: &mut Group, u: Dimensions| {
            chevron(canvas,
                0.5 * u.sw - 0.5 * u.sp,
                0.3 * u.sh + 0.5 * u.sp, 0.8 * u.sh - 0.5 * u.sp
            );
            canvas.move_to(0., 0.);
            canvas.line_to(0., u.sh - u.sp);
            canvas.apply_line_width(u.csp);
            canvas.apply(Operator::DestinationOut);
            stroke_round(canvas);
        },
        |canvas: &mut Group, u: Dimensions| {
            junction_small_casing(canvas, u)
        }
    ),
    ("de.uest.first", "crossover.first") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.move_to(
                -0.5 * u.sw + 0.5 * u.sp, 0.8 * u.sh - 0.5 * u.sp
            );
            canvas.line_to(0., 0.3 * u.sh + 0.5 * u.sp);
            canvas.move_to(0., 0.);
            canvas.line_to(0., u.sh - u.sp);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        }
    ),
    ("de.uest.second", "crossover.second") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.move_to(
                0.5 * u.sw - 0.5 * u.sp, 0.8 * u.sh - 0.5 * u.sp
            );
            canvas.line_to(0., 0.3 * u.sh + 0.5 * u.sp);
            canvas.move_to(0., 0.);
            canvas.line_to(0., u.sh - u.sp);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        }
    ),

    ("de.zst", "de.kzst") => (
        |canvas: &mut Group, u: Dimensions| {
            stop(canvas, u);
            canvas.arc(
                0., u.sh - 0.5 * (u.sh - 2. * u.sp),
                0.5 * (u.sh - 5. * u.sp),
                0., 2. * PI,
            );
            canvas.fill()
        },
        |canvas: &mut Group, u: Dimensions| {
            stop_small(canvas, u)
        }
    ),

    ("ltd-stop") => (
        |canvas: &mut Group, u: Dimensions| {
            let hsp = 0.5 * u.sp;
            stop(canvas, u);
            canvas.move_to(-0.5 * u.sw + hsp, 2.5 * u.sp);
            canvas.line_to(0.5 * u.sw - hsp, u.sh - 0.5 * u.sp);
            canvas.apply_line_width(u.sp);
            stroke_round(canvas)
        },
        |canvas: &mut Group, u: Dimensions| {
            stop_small(canvas, u)
        }
    ),

    ("ref") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.apply_line_width(u.bp);
            canvas.move_to(0., 0.);
            canvas.line_to(0., 0.5 * u.sh);
            canvas.stroke()
        }
    ),
    ("refdt") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.apply_line_width(u.bp);
            canvas.move_to(0., 0.);
            canvas.line_to(0., u.dt);
            canvas.stroke()
        }
    ),
    ("statdt") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.apply_line_width(u.sp);
            canvas.move_to(0., 0.);
            canvas.line_to(0., u.dt);
            canvas.stroke()
        }
    ),

    ("tunnel.l") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.move_to(0., 0.);
            canvas.move_to(0., 0.);
            canvas.line_to(1.0 * u.dt, 0.0);
            canvas.line_to(1.75 * u.dt, -0.75 * u.dt);
            canvas.apply_line_width(u.bp);
            canvas.stroke()
        }
    ),
    ("tunnel.r") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.move_to(0., 0.);
            canvas.line_to(-1.0 * u.dt, 0.0);
            canvas.line_to(-1.75 * u.dt, -0.75 * u.dt);
            canvas.apply_line_width(u.bp);
            canvas.stroke()
        }
    ),
    ("tunnel.dt") => (
        |canvas: &mut Group, u: Dimensions| {
            canvas.apply_line_width(u.bp);
            canvas.move_to(0., 0.);
            canvas.line_to(0., u.dt);
            canvas.stroke()
        }
    )
}


//------------ Component Functions -------------------------------------------

fn station_small(
    canvas: &mut Group, u: Dimensions
) {
    let hsp = 0.5 * u.sp;
    canvas.move_to(-0.5 * u.sw, 1.75 * u.sp - hsp);
    canvas.line_to(-0.5 * u.sw, u.sh);
    canvas.line_to(0.5 * u.sw, u.sh);
    canvas.line_to(0.5 * u.sw, 1.75 * u.sp - hsp);
    canvas.close_path();

    /*
    top_ds_rect(
        canvas, -0.5 * u.sw, 0.5 * u.sw, 0., u.sh, u.ds,
    );
    */
    canvas.fill();
    canvas.new_path();
}

fn station_xsmall(
    canvas: &mut Group, u: Dimensions
) {
    let sp = 0.8 * u.sp;
    let hsp = 0.5 * u.sp;
    let x = 0.4 * u.sw;
    let (y0, y1) = (2.6 * sp - hsp, 0.9 * u.sh);
    canvas.move_to(-x, y0);
    canvas.line_to(-x, y1);
    canvas.line_to(x, y1);
    canvas.line_to(x, y0);
    canvas.close_path();

    /*
    top_ds_rect(
        canvas, -0.5 * u.sw, 0.5 * u.sw, 0., u.sh, u.ds,
    );
    */
    canvas.fill();
    canvas.new_path();
}

fn station_casing(
    canvas: &mut Group, u: Dimensions
) {
    let hsp = 0.5 * (u.csp - u.sp);
    canvas.move_to(-0.5 * u.sw - hsp, 2.5 * u.sp - hsp);
    canvas.line_to(-0.5 * u.sw - hsp, u.sh + hsp);
    canvas.line_to(0.5 * u.sw + hsp, u.sh + hsp);
    canvas.line_to(0.5 * u.sw + hsp, 2.5 * u.sp - hsp);
    canvas.close_path();
    canvas.apply(CASING_COLOR);
    canvas.fill();
}

fn station_small_casing(
    canvas: &mut Group, u: Dimensions
) {
    let hsp = 0.5 * u.sp;
    canvas.move_to(-0.5 * u.sw, 1.75 * u.sp - hsp);
    canvas.line_to(-0.5 * u.sw, u.sh);
    canvas.line_to(0.5 * u.sw, u.sh);
    canvas.line_to(0.5 * u.sw, 1.75 * u.sp - hsp);
    canvas.close_path();
    canvas.apply(CASING_COLOR);
    canvas.apply_line_width(u.csp);
    /*
    top_ds_rect(canvas,
        -0.5 * u.sw + 0.5 * u.sp,
        0.5 * u.sw - 0.5 * u.sp,
        0.,
        u.sh - 0.5 * u.sp,
        u.ds,
    );
    canvas.apply_line_width(u.csp);
    */
    stroke_round(canvas)
}

fn stop(
    canvas: &mut Group, u: Dimensions
)  {
    let hsp = 0.5 * u.sp;
    canvas.move_to(-0.5 * u.sw + hsp, 2.5 * u.sp);
    canvas.line_to(-0.5 * u.sw + hsp, u.sh - 0.5 * u.sp);
    canvas.line_to(0.5 * u.sw - hsp, u.sh - 0.5 * u.sp);
    canvas.line_to(0.5 * u.sw - hsp, 2.5 * u.sp);
    canvas.close_path();
    canvas.apply_line_width(u.sp);
    canvas.stroke();
    canvas.new_path();
}

fn stop_small(
    canvas: &mut Group, u: Dimensions
) {
    let hsp = 0.5 * u.sp;
    canvas.move_to(-0.5 * u.sw + hsp, 1.75 * u.sp);
    canvas.line_to(-0.5 * u.sw + hsp, u.sh - hsp);
    canvas.line_to(0.5 * u.sw - hsp, u.sh - hsp);
    canvas.line_to(0.5 * u.sw - hsp, 1.75 * u.sp);
    canvas.close_path();

    /*
    top_ds_rect(canvas,
        -0.5 * u.sw + 0.5 * u.sp,
        0.5 * u.sw - 0.5 * u.sp,
        0.,
        u.sh - 0.5 * u.sp,
        u.ds,
    );
    */
    canvas.apply_line_width(u.sp);
    stroke_round(canvas);
    canvas.new_path();
}

fn stop_xsmall(
    canvas: &mut Group, u: Dimensions
) {
    let sp = 0.8 * u.sp;
    let hsp = 0.5 * sp;
    let x = 0.4 * u.sw - hsp;
    let (y0, y1) = (2.6 * sp, 0.9 * u.sh - hsp);
    canvas.move_to(-x, y0);
    canvas.line_to(-x, y1);
    canvas.line_to(x, y1);
    canvas.line_to(x, y0);
    canvas.close_path();

    canvas.apply_line_width(sp);
    stroke_round(canvas);
    canvas.new_path();
}

fn junction_small_casing(
    canvas: &mut Group, u: Dimensions
)  {
    chevron(canvas, 0.4 * u.sw - 0.5 * u.sp, 0., u.sh - 0.5 * u.sp);
    canvas.apply_line_width(u.csp);
    canvas.stroke()
}


//------------ Helper Functions ----------------------------------------------

fn chevron(canvas: &mut Group, x: f64, y0: f64, y1: f64) {
    canvas.move_to(-x, y1);
    canvas.line_to(0., y0);
    canvas.line_to(x, y1);
}

fn stroke_round(canvas: &mut Group) {
    canvas.apply(LineCap::Round);
    canvas.stroke();
    canvas.apply(LineCap::Butt);
}

fn top_ds_rect(canvas: &mut Group, x0: f64, x1: f64, y0: f64, y1: f64, ds: f64) {
    let hds = 0.5 * ds; // half ds

    canvas.move_to(x0, y0);
    canvas.line_to(x0, y1 - ds);
    canvas.curve_to(x0, y1 - hds,   x0 + hds, y1,   x0 + ds, y1);
    canvas.line_to(x1 - ds, y1);
    canvas.curve_to(x1 - hds, y1,   x1, y1 - hds,   x1, y1 - ds);
    canvas.line_to(x1, y0);
}

