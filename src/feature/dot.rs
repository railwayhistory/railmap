//! Rendering dot markers.

use femtomap::world;
use femtomap::import::eval::{EvalErrors, Failed, SymbolSet};
use femtomap::path::Position;
use femtomap::render::{Canvas, LineWidth, Operator};
use kurbo::Circle;
use crate::class::Railway;
use crate::style::Style;
use super::{AnyShape, Category, Group, Feature};


//------------ DotMarker -----------------------------------------------------

/// The rendering rule for a dot marker.
pub struct DotMarker {
    /// The position the marker is attached to.
    position: Position,

    /// The feature class.
    class: Railway,

    /// The size of the dot
    size: Size,

    /// Is the dot filled or what?
    inner: Inner,

    /// Is there casing around the dot?
    casing: bool,
}

impl DotMarker {
    pub fn guide(class: Railway, position: Position) -> Self {
        Self {
            position, class,
            size: Size::Small,
            inner: Inner::Fill,
            casing: false,
        }
    }

    pub fn try_from_arg(
        arg: &mut SymbolSet,
        position: Position,
        err: &mut EvalErrors,
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
        let class = Railway::from_symbols(arg);
        arg.check_exhausted(err)?;
        Ok(Some(DotMarker { position, class, size, inner, casing }))
    }

    pub fn from_arg(
        mut arg: SymbolSet,
        position: Position,
        err: &mut EvalErrors,
    ) -> Result<Self, Failed> {
        let size = Size::from_symbols(&mut arg);
        let class = Railway::from_symbols(&mut arg);
        let inner = Inner::from_symbols(&class, &mut arg);
        let casing = Self::casing_from_symbols(&mut arg);
        arg.check_exhausted(err)?;
        Ok(DotMarker { position, class, size, inner, casing })
    }

    fn casing_from_symbols(
        symbols: &mut SymbolSet,
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

    pub fn class(&self) -> &Railway {
        &self.class
    }

    pub fn render(&self, style: &Style, canvas: &mut Canvas) {
        let (point, _) = self.position.resolve(style);
        let radius = self.size.radius() * style.units().dt;
        let sp = style.units().sp;

        if self.casing {
            let mut sketch = canvas.sketch();
            sketch.apply(
                Circle::new(point, radius + 2. * sp)
            );
            sketch.apply(Operator::DestinationOut);
            sketch.fill();
        }

        match self.inner {
            Inner::Fill => {
                canvas.sketch().apply(
                    Circle::new(point, radius)
                ).apply(
                    style.primary_marker_color(&self.class)
                ).fill();
            }
            Inner::Stroke => {
                canvas.sketch().apply(
                    Circle::new(point, radius - 0.75 * sp)
                ).apply(
                    style.primary_marker_color(
                        &self.class
                    )
                ).apply(
                    LineWidth(1.5 * sp)
                ).stroke();
            }
            Inner::None => { }
        }
    }
}

impl Feature for DotMarker {
    fn storage_bounds(&self) -> world::Rect {
        self.position.storage_bounds()
    }

    fn group(&self) -> Group {
        Group::with_railway(Category::Marker, &self.class)
    }

    fn shape(
        &self, _style: &Style, _canvas: &Canvas
    ) -> AnyShape {
        AnyShape::single_stage(|style: &Style, canvas: &mut Canvas| {
            self.render(style, canvas)
        })
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
    fn from_symbols(symbols: &mut SymbolSet) -> Self {
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

#[derive(Clone, Copy, Debug, Default)]
enum Inner {
    None,
    #[default]
    Fill,
    Stroke, 
}

impl Inner {
    fn from_symbols(class: &Railway, symbols: &mut SymbolSet) -> Self {
        if symbols.take("filled") {
            Inner::Fill
        }
        else if symbols.take("open") {
            Inner::Stroke
        }
        else if symbols.contains("casing") {
            Inner::None
        }
        else if class.status().is_open() && !class.pax().is_full() {
            Inner::Stroke
        }
        else {
            Inner::Fill
        }
    }
}

