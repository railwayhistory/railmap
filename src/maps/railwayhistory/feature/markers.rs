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

use kurbo::Rect;
use std::f64::consts::PI;
use crate::import::{ast, eval};
use crate::import::Failed;
use crate::render::canvas::Canvas;
use crate::render::path::Position;
use super::super::class::Class;
use super::super::style::{Dimensions, Style};
use super::super::theme::Railwayhistory;


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
    orientation: Option<f64>,

    /// The feature class.
    class: Class,

    /// The marker to use.
    marker: &'static (dyn Fn(&Canvas, Dimensions) + Sync),
}


impl StandardMarker {
    pub fn from_arg(
        arg: eval::Expression<Railwayhistory>,
        position: Position,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let (mut symbols, pos) = arg.into_symbol_set(err)?;
        let (marker, rotate) = Self::marker_from_symbols(
            &mut symbols, pos, err
        )?;
        let orientation = if rotate {
            Some(Self::rotation_from_symbols(&mut symbols, pos, err)?)
        }
        else {
            None
        };
        let class = Class::from_symbols(&mut symbols);
        symbols.check_exhausted(err)?;
        Ok(StandardMarker { position, orientation, class, marker })
    }

    fn rotation_from_symbols(
        symbols: &mut eval::SymbolSet,
        _pos: ast::Pos,
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

    fn marker_from_symbols(
        symbols: &mut eval::SymbolSet,
        pos: ast::Pos,
        err: &mut eval::Error
    ) -> Result<
        (&'static (dyn Fn(&Canvas, Dimensions) + Sync), bool),
        Failed
    > {
        for (name, marker) in MARKERS {
            if symbols.take(name) {
                return Ok((marker, true))
            }
        }
        for (name, marker) in DOT_MARKERS {
            if symbols.take(name) {
                return Ok((marker, false))
            }
        }
        err.add(pos, "missing marker");
        Err(Failed)
    }

    pub fn storage_bounds(&self) -> Rect {
        self.position.storage_bounds()
    }

    pub fn render(&self, style: &Style, canvas: &Canvas) {
        let (point, angle) = self.position.resolve(canvas);
        canvas.translate(point.x, point.y);
        if let Some(rotation) = self.orientation {
            canvas.rotate(angle + rotation);
        }
        style.primary_marker_color(&self.class).apply(canvas);
        (self.marker)(canvas, style.dimensions());
        canvas.identity_matrix();
    }
}


//------------ Markers ------------------------------------------------------

static MARKERS: &[
    (&'static str, &'static (dyn Fn(&Canvas, Dimensions) + Sync))
] = &[
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

    ("de.abzw.casing", &|canvas, u| {
        canvas.set_operator(cairo::Operator::Clear);
        canvas.set_line_cap(cairo::LineCap::Round);
        canvas.set_line_width(4.0 * u.sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., 0.5 * u.sh);
        canvas.move_to(-0.5 * u.sw, 0.5 * u.sh);
        canvas.line_to(0.5 * u.sw, 0.5 * u.sh);
        canvas.move_to(-0.5 * u.sw, u.sh);
        canvas.line_to(0., 0.5 * u.sh);
        canvas.line_to(0.5 * u.sw, u.sh);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
        canvas.set_line_cap(cairo::LineCap::Butt);
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

    ("de.bf.casing", &|canvas, u| {
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
        canvas.set_operator(cairo::Operator::Clear);
        canvas.set_line_width(3.0 * u.sp);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
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

    ("de.bft.casing", &|canvas, u| {
        canvas.move_to(-0.3 * u.sw + 0.5 * u.sp, 0.);
        canvas.line_to(-0.3 * u.sw + 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.3 * u.sw - 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.3 * u.sw - 0.5 * u.sp, 0.);
        canvas.set_operator(cairo::Operator::Clear);
        canvas.set_line_width(3.0 * u.sp);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
    }),

    ("de.bftk", &|canvas, u| {
        canvas.move_to(-0.2 * u.sw, 0.);
        canvas.line_to(-0.2 * u.sw, u.sh - u.ds);
        canvas.curve_to(
            -0.2 * u.sw, u.sh - 1.5 * u.ds,
            -0.2 * u.sw + 0.5 * u.ds, u.sh,
            -0.2 * u.sw + u.ds, u.sh
        );
        canvas.line_to(0.2 * u.sw - u.ds, u.sh);
        canvas.curve_to(
            0.2 * u.sw - 0.5 * u.ds, u.sh,
            0.2 * u.sw, u.sh - 0.5 * u.ds,
            0.2 * u.sw, u.sh - u.ds
        );
        canvas.line_to(0.2 * u.sw, 0.);
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

    ("de.bft.abzw", &|canvas, u| {
        /*
        canvas.move_to(-0.3 * u.sw, 0.);
        canvas.line_to(-0.3 * u.sw, 0.5 * u.sh);
        canvas.line_to(0.3 * u.sw, 0.5 * u.sh);
        canvas.line_to(0.3 * u.sw, 0.);
        canvas.close_path();
        canvas.fill();
        */
        canvas.set_line_width(u.sp);
        canvas.move_to(-0.15 * u.sw, 0.);
        canvas.line_to(-0.15 * u.sw, 0.5 * u.sh);
        canvas.move_to(0.15 * u.sw, 0.);
        canvas.line_to(0.15 * u.sw, 0.5 * u.sh);
        canvas.move_to(-0.5 * u.sw, 0.5 * u.sh);
        canvas.line_to(0.5 * u.sw, 0.5 * u.sh);
        canvas.move_to(-0.5 * u.sw, u.sh);
        canvas.line_to(0., 0.5 * u.sh);
        canvas.line_to(0.5 * u.sw, u.sh);
        canvas.stroke();
    }),

    ("de.bk", &|canvas, u| {
        canvas.set_line_width(u.sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., u.sh);
        canvas.move_to(-0.5 * u.sw, 0.5 * u.sh);
        canvas.line_to(0.5 * u.sw, 0.5 * u.sh);
        canvas.stroke();
    }),
    ("de.bk.casing", &|canvas, u| {
        canvas.set_line_width(3. * u.sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., u.sh + u.sp);
        canvas.move_to(-0.5 * u.sw - u.sp, 0.5 * u.sh);
        canvas.line_to(0.5 * u.sw + u.sp, 0.5 * u.sh);
        canvas.set_operator(cairo::Operator::Clear);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
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
        canvas.move_to(0., -0.5 * u.dt);
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

    ("de.gbf", &|canvas, u| {
        canvas.set_line_width(u.sp);
        canvas.move_to(-0.5 * u.sw, 0.);
        canvas.line_to(0., u.sh);
        canvas.line_to(0.5 * u.sw, 0.);
        canvas.fill();
    }),

    ("de.hp", &|canvas, u| {
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, 0.);
        canvas.line_to(-0.5 * u.sw + 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, 0.);
        canvas.set_line_width(u.sp);
        canvas.stroke();
    }),

    ("de.hp.casing", &|canvas, u| {
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, 0.);
        canvas.line_to(-0.5 * u.sw + 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, 0.);
        canvas.set_operator(cairo::Operator::Clear);
        canvas.set_line_width(3.0 * u.sp);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
    }),

    ("de.hpext", &|canvas, u| {
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, 0.);
        canvas.line_to(-0.5 * u.sw + 0.5 * u.sp, u.dt);
        canvas.move_to(0.5 * u.sw - 0.5 * u.sp, u.dt);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, 0.);
        canvas.set_line_width(u.sp);
        canvas.stroke();
    }),

    ("de.hpext.casing", &|canvas, u| {
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, 0.);
        canvas.line_to(-0.5 * u.sw + 0.5 * u.sp, u.dt);
        canvas.move_to(0.5 * u.sw - 0.5 * u.sp, u.dt);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, 0.);
        canvas.set_operator(cairo::Operator::Clear);
        canvas.set_line_width(3.0 * u.sp);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
    }),

    ("de.hst", &|canvas, u| {
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, 0.);
        canvas.line_to(-0.5 * u.sw + 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, u.sh - 0.5 * u.sp);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, 0.);
        canvas.move_to(-0.5 * u.sw + 0.5 * u.sp, u.sh);
        canvas.line_to(0., 0.05 * u.sh);
        canvas.line_to(0.5 * u.sw - 0.5 * u.sp, u.sh);
        canvas.set_line_width(u.sp);
        canvas.stroke();
    }),

    ("de.inbf", &|canvas, u| {
        let sh = 2.0 * u.dt;
        canvas.move_to(-0.5 * u.sw, 0.);
        canvas.line_to(-0.5 * u.sw, sh - u.ds);
        canvas.curve_to(
            -0.5 * u.sw, sh - 1.5 * u.ds,
            -0.5 * u.sw + 0.5 * u.ds, sh,
            -0.5 * u.sw + u.ds, sh
        );
        canvas.line_to(0.5 * u.sw - u.ds, sh);
        canvas.curve_to(
            0.5 * u.sw - 0.5 * u.ds, sh,
            0.5 * u.sw, sh - 0.5 * u.ds,
            0.5 * u.sw, sh - u.ds
        );
        canvas.line_to(0.5 * u.sw, 0.);
        canvas.close_path();
        canvas.fill();
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

    ("de.lgr", &|canvas, u| {
        let r = 0.8 * u.dt;
        canvas.set_line_width(u.bp);
        canvas.move_to(0., -0.5 * u.dt);
        canvas.line_to(0., 2. * r);
        canvas.stroke();
        canvas.arc(0., 3. * r, r, 0., 2. * PI);
        canvas.fill();
    }),

    ("de.stw", &|canvas, u| {
        canvas.set_line_width(u.sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., 2. * u.dt);
        canvas.move_to(-u.dt, u.dt);
        canvas.line_to(u.dt, u.dt);
        canvas.stroke();
    }),
    ("de.stw.casing", &|canvas, u| {
        canvas.set_line_width(1.5 * u.sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., 2. * u.dt);
        canvas.move_to(-u.dt, u.dt);
        canvas.line_to(u.dt, u.dt);
        canvas.set_operator(cairo::Operator::Clear);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
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
    ("de.uest.casing", &|canvas, u| {
        canvas.set_operator(cairo::Operator::Clear);
        canvas.set_line_cap(cairo::LineCap::Round);
        canvas.set_line_width(4.0 * u.sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., 0.5 * u.sh);
        canvas.move_to(-0.5 * u.sw, 0.5 * u.sh);
        canvas.line_to(0.5 * u.sw, 0.5 * u.sh);
        canvas.move_to(-0.25 * u.sw, 0.5 * u.sh);
        canvas.line_to(-0.25 * u.sw, u.sh);
        canvas.move_to(0.25 * u.sw, 0.5 * u.sh);
        canvas.line_to(0.25 * u.sw, u.sh);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
        canvas.set_line_cap(cairo::LineCap::Butt);
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

    ("ref", &|canvas, u| {
        canvas.set_line_width(u.bp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., 0.5 * u.sh);
        canvas.stroke();
    }),

    ("refdt", &|canvas, u| {
        canvas.set_line_width(u.bp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., u.dt);
        canvas.stroke();
    }),

    ("statcase", &|canvas, u| {
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
        canvas.set_line_width(2. * u.sp);
        canvas.set_operator(cairo::Operator::Clear);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
    }),

    ("statdt", &|canvas, u| {
        canvas.set_line_width(u.sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., u.dt);
        canvas.stroke();
    }),

    ("tunnel.l", &|canvas, u| {
        canvas.move_to(0., 0.);
        canvas.move_to(0., 0.);
        canvas.line_to(1.0 * u.dt, 0.0);
        canvas.line_to(1.75 * u.dt, -0.75 * u.dt);
        canvas.set_line_width(u.bp);
        canvas.stroke();
    }),
    ("tunnel.dt", &|canvas, u| {
        canvas.set_line_width(u.bp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., u.dt);
        canvas.stroke();
    }),
    ("tunnel.r", &|canvas, u| {
        canvas.move_to(0., 0.);
        canvas.line_to(-1.0 * u.dt, 0.0);
        canvas.line_to(-1.75 * u.dt, -0.75 * u.dt);
        canvas.set_line_width(u.bp);
        canvas.stroke();
    }),
];

static DOT_MARKERS: &[
    (&'static str, &'static (dyn Fn(&Canvas, Dimensions) + Sync))
] = &[
    ("statdot", &|canvas, u| {
        canvas.move_to(0., 0.);
        canvas.arc(0., 0., 0.7 * u.dt, 0., 2.0 * PI);
        canvas.set_line_width(4. * u.sp);
        canvas.set_operator(cairo::Operator::Clear);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
        canvas.move_to(0., 0.);
        canvas.arc(0., 0., 0.7 * u.dt, 0., 2.0 * PI);
        canvas.fill();
    }),


    ("dot.filled", &|canvas, u| {
        canvas.move_to(0., 0.);
        canvas.arc(0., 0., 0.7 * u.dt + 0.5 * u.sp, 0., 2.0 * PI);
        canvas.fill();
    }),
    ("dot.open", &|canvas, u| {
        canvas.move_to(0.7 * u.dt, 0.);
        canvas.arc(0., 0., 0.7 * u.dt, 0., 2.0 * PI);
        canvas.set_line_width(u.sp);
        canvas.stroke();
    }),
    ("dot.casing", &|canvas, u| {
        canvas.move_to(0., 0.);
        canvas.arc(0., 0., 0.7 * u.dt, 0., 2.0 * PI);
        canvas.set_line_width(3. * u.sp);
        canvas.set_operator(cairo::Operator::Clear);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
    }),
    ("dot.filled.casing", &|canvas, u| {
        canvas.move_to(0., 0.);
        canvas.arc(0., 0., 0.7 * u.dt, 0., 2.0 * PI);
        canvas.set_line_width(3. * u.sp);
        canvas.set_operator(cairo::Operator::Clear);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
        canvas.move_to(0., 0.);
        canvas.arc(0., 0., 0.7 * u.dt + 0.5 * u.sp, 0., 2.0 * PI);
        canvas.fill();
    }),
    ("dot.open.casing", &|canvas, u| {
        canvas.move_to(0., 0.);
        canvas.arc(0., 0., 0.7 * u.dt, 0., 2.0 * PI);
        canvas.set_line_width(2. * u.sp);
        canvas.set_operator(cairo::Operator::Clear);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
        canvas.move_to(0.7 * u.dt, 0.);
        canvas.arc(0., 0., 0.7 * u.dt, 0., 2.0 * PI);
        canvas.set_line_width(u.sp);
        canvas.stroke();
    }),

    ("sdot", &|canvas, u| {
        canvas.move_to(0., 0.);
        canvas.arc(0., 0., 0.5 * u.dt, 0., 2.0 * PI);
        canvas.set_line_width(2. * u.sp);
        canvas.set_operator(cairo::Operator::Clear);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
        canvas.move_to(0., 0.);
        canvas.arc(0., 0., 0.5 * u.dt, 0., 2.0 * PI);
        canvas.fill();
    }),
    ("sdot.filled", &|canvas, u| {
        canvas.move_to(0., 0.);
        canvas.arc(0., 0., 0.5 * u.dt, 0., 2.0 * PI);
        canvas.fill();
    }),
    ("sdot.casing", &|canvas, u| {
        canvas.move_to(0., 0.);
        canvas.arc(0., 0., 0.5 * u.dt, 0., 2.0 * PI);
        canvas.set_line_width(2. * u.sp);
        canvas.set_operator(cairo::Operator::Clear);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
    }),
];

