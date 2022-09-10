//! The feature for routes.

use kurbo::Rect;
use crate::import::Failed;
use crate::import::eval::{self, ArgumentList, SymbolSet};
use crate::render::canvas::Canvas;
use crate::render::path::Trace;
use super::super::class::Class;
use super::super::style::Style;
use super::super::theme::Overnight;


//------------ RouteContour --------------------------------------------------

/// The contour of a route.
pub struct RouteContour {
    class: Class,
    combination: Option<Combination>,
    trace: Trace,
}

impl RouteContour {
    pub fn from_args(
        args: ArgumentList<Overnight>,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let [symbols, trace] = args.into_positionals(err)?;
        let symbols = symbols.into_symbol_set(err);
        let trace = trace.into_path(err)?.0;
        let mut symbols = symbols?;
        let class = Class::from_symbols(&mut symbols);
        let combination = Combination::from_symbols(&mut symbols);
        symbols.check_exhausted(err)?;
        Ok(RouteContour { class, combination, trace })
    }

    pub fn storage_bounds(&self) -> Rect {
        self.trace.storage_bounds()
    }

    pub fn render(&self, style: &Style, canvas: &Canvas) {
        canvas.set_line_width(style.dimensions().line_width);
        if let Some(combination) = self.combination {
            combination.set_dash(style, canvas);
        }
        style.route_color(&self.class).apply(canvas);
        self.trace.apply(canvas, style);
        canvas.stroke().unwrap();
        canvas.set_dash(&[], 0.);
    }
}


//------------- Combination --------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct Combination {
    /// The overall number of combined routes.
    len: usize,

    /// Our position in the combined routes.
    pos: usize,
}

impl Combination {
    fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
        if symbols.take("onetwo") {
            Some(Combination { len: 2, pos: 0 })
        }
        else if symbols.take("twotwo") {
            Some(Combination { len: 2, pos: 1 })
        }
        else if symbols.take("onethree") {
            Some(Combination { len: 3, pos: 0 })
        }
        else if symbols.take("twothree") {
            Some(Combination { len: 3, pos: 1 })
        }
        else if symbols.take("threethree") {
            Some(Combination { len: 3, pos: 2 })
        }
        else {
            None
        }
    }

    fn set_dash(&self, style: &Style, canvas: &Canvas) {
        let seg = style.dimensions().seg;
        canvas.set_dash(
            &[seg, (self.len - 1) as f64 * seg],
            (self.pos as f64) * seg
        )
    }
}

