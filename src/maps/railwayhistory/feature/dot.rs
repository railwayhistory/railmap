//! Rendering dot markers.

use std::f64::consts::PI;
use kurbo::Rect;
use crate::import::eval;
use crate::import::Failed;
use crate::render::canvas::Canvas;
use crate::render::path::Position;
use super::super::class::Class;
use super::super::style::Style;


//------------ DotMarker -----------------------------------------------------

/// The rendering rule for a dot marker.
pub struct DotMarker {
    /// The position the marker is attached to.
    position: Position,

    /// The feature class.
    class: Class,

    /// The size of the dot
    size: Size,

    /// Is the dot filled or what?
    inner: Inner,

    /// Is there casing around the dot?
    casing: bool,
}

impl DotMarker {
    pub fn try_from_arg(
        arg: &mut eval::SymbolSet,
        position: Position,
        err: &mut eval::Error,
    ) -> Result<Option<Self>, Failed> {
        let (size, inner, casing) = if arg.take("statdot") {
            (Size::Medium, Inner::Fill, true)
        }
        else if arg.take("dot.filled") {
            (Size::Medium, Inner::Fill, false)
        }
        else if arg.take("dot.casing") {
            (Size::Medium, Inner::None, true)
        }
        else {
            return Ok(None)
        };
        let class = Class::from_symbols(arg);
        arg.check_exhausted(err)?;
        Ok(Some(DotMarker { position, class, size, inner, casing }))
    }

    pub fn from_arg(
        mut arg: eval::SymbolSet,
        position: Position,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let size = Size::from_symbols(&mut arg);
        let inner = Inner::from_symbols(&mut arg);
        let casing = Self::casing_from_symbols(&mut arg);
        let class = Class::from_symbols(&mut arg);
        arg.check_exhausted(err)?;
        Ok(DotMarker { position, class, size, inner, casing })
    }

    fn casing_from_symbols(
        symbols: &mut eval::SymbolSet,
    ) -> bool {
        if symbols.take("casing") {
            true
        }
        else if symbols.take("over") {
            false
        }
        else {
            true
        }
    }

    pub fn class(&self) -> &Class {
        &self.class
    }

    pub fn storage_bounds(&self) -> Rect {
        self.position.storage_bounds()
    }

    pub fn render(&self, style: &Style, canvas: &Canvas) {
        let (point, _) = self.position.resolve(canvas, style);
        canvas.translate(point.x, point.y);
        style.primary_marker_color(&self.class).apply(canvas);
        let u = style.dimensions();
        let radius = self.size.radius() * style.dimensions().dt;
        let sp = style.dimensions().sp;

        if self.casing {
            canvas.move_to(0., 0.);
            canvas.arc(0., 0., radius, 0., 2.0 * PI);
            canvas.set_line_width(3. * sp);
            canvas.set_operator(cairo::Operator::Clear);
            canvas.stroke().unwrap();
            canvas.set_operator(cairo::Operator::Over);
        }
        match self.inner {
            Inner::Fill => {
                canvas.move_to(0., 0.);
                canvas.arc(0., 0., radius, 0., 2.0 * PI);
                canvas.fill().unwrap();
            }
            Inner::Stroke => {
                canvas.move_to(radius - 0.5 * sp, 0.);
                canvas.arc(0., 0., radius - 0.5 * sp, 0., 2.0 * PI);
                canvas.set_line_width(u.sp);
                canvas.stroke().unwrap();
            }
            Inner::None => { }
        }

        canvas.identity_matrix();
    }
}


//------------ Size ----------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum Size {
    Small,
    Medium,
    Large,

}

impl Size {
    fn from_symbols(symbols: &mut eval::SymbolSet) -> Self {
        if symbols.take("small") {
            Size::Small
        }
        else if symbols.take("large") {
            Size::Large
        }
        else if symbols.take("medium") {
            Size::Medium
        }
        else {
            Size::Medium
        }
    }

    fn radius(self) -> f64 {
        match self {
            Self::Small => 0.5,
            Self::Medium => 0.7,
            Self::Large => 1.0,
        }
    }
}


//------------ Inner ---------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum Inner {
    None,
    Fill,
    Stroke, 
}

impl Inner {
    fn from_symbols(symbols: &mut eval::SymbolSet) -> Self {
        if symbols.take("filled") {
            Inner::Fill
        }
        else if symbols.take("open") {
            Inner::Stroke
        }
        else if symbols.contains("casing") {
            Inner::None
        }
        else {
            Inner::Fill
        }
    }
}

