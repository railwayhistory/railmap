//! Rendering dot markers.

use std::f64::consts::PI;
use femtomap::path::Position;
use femtomap::render::canvas;
use kurbo::{Circle, Rect, Point, Shape as _};
use crate::import::eval;
use crate::import::Failed;
use super::super::class::Class;
use super::super::style::Style;
use super::Shape;


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

    pub fn shape(
        &self, style: &Style, canvas: &canvas::Canvas
    ) -> Box<dyn Shape + '_> {
        Box::new(|style: &Style, canvas: canvas::Group| {
            self.render(style, canvas)
        })
    }

    pub fn render(&self, style: &Style, mut canvas: canvas::Group) {
        let (point, _) = self.position.resolve(style);
        canvas.apply(canvas::Matrix::identity().translate(point));

        let u = style.dimensions();
        let radius = self.size.radius() * style.dimensions().dt;
        let sp = style.dimensions().sp;

        if self.casing {
            canvas.apply_outline(
                Circle::new((0., 0.), radius + 1.5 * sp).path_elements(0.1)
            );
            canvas.apply(canvas::Operator::DestinationOut);
            canvas.fill();
            canvas.apply(canvas::Operator::default());
        }

        canvas.apply(style.primary_marker_color(&self.class));
        match self.inner {
            Inner::Fill => {
                canvas.apply_outline(
                    Circle::new((0., 0.), radius).path_elements(0.1)
                );
                canvas.fill();
            }
            Inner::Stroke => {
                canvas.apply_outline(
                    Circle::new((0., 0.), radius - 0.5 * sp).path_elements(0.1)
                );
                canvas.apply_line_width(u.sp);
                canvas.stroke();
            }
            Inner::None => { }
        }
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

