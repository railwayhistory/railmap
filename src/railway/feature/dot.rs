//! Rendering dot markers.

use femtomap::world;
use femtomap::import::eval::{EvalErrors, Failed, SymbolSet};
use femtomap::path::Position;
use femtomap::render::{Canvas, Color, LineWidth};
use kurbo::{Circle, Point};
use crate::railway::class::Railway;
use crate::railway::import::eval::{Scope, ScopeExt};
use crate::railway::style::Style;
use super::{AnyShape, Category, Group, Feature, Shape, Stage, StageSet};


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
        scope: &Scope,
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
        let class = Railway::from_symbols(arg, scope);
        arg.check_exhausted(err)?;
        Ok(Some(DotMarker { position, class, size, inner, casing }))
    }

    pub fn from_arg(
        mut arg: SymbolSet,
        position: Position,
        scope: &Scope,
        err: &mut EvalErrors,
    ) -> Result<Self, Failed> {
        let size = Size::from_symbols(&mut arg);
        let class = Railway::from_symbols(&mut arg, scope);
        let inner = Inner::from_symbols(&class, &mut arg);
        let casing = Self::casing_from_symbols(&mut arg);
        arg.check_exhausted(err)?;
        Ok(DotMarker { position, class, size, inner, casing })
    }

    pub fn km_from_arg(
        mut arg: SymbolSet,
        position: Position,
        scope: &Scope,
        err: &mut EvalErrors,
    ) -> Result<Self, Failed> {
        let size = Size::km_from_symbols(&mut arg);
        let class = Railway::from_symbols(&mut arg, scope);
        let inner = Inner::from_symbols(&class, &mut arg);
        let casing = Self::casing_from_symbols(&mut arg);
        arg.check_exhausted(err)?;
        Ok(DotMarker { position, class, size, inner, casing })
    }

    pub fn from_position(
        position: Position,
        scope: &Scope,
    ) -> Result<Self, Failed> {
        Ok(DotMarker {
            position,
            class: scope.railway().clone(),
            size: Size::default(),
            inner: Inner::from_scope(scope),
            casing: true
        })
    }

    pub fn km_from_position(
        position: Position,
        scope: &Scope,
    ) -> Result<Self, Failed> {
        Ok(DotMarker {
            position,
            class: scope.railway().clone(),
            size: Size::Small,
            inner: Inner::from_scope(scope),
            casing: true
        })
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
}

impl Feature for DotMarker {
    fn storage_bounds(&self) -> world::Rect {
        self.position.storage_bounds()
    }

    fn group(&self) -> Group {
        Group::with_railway(Category::Marker, &self.class)
    }

    fn shape(
        &self, style: &Style, _canvas: &Canvas
    ) -> AnyShape {
        DotShape {
            feature: self,
            center: self.position.resolve(style).0
        }.into()
    }
}


//------------ Shape ---------------------------------------------------------

struct DotShape<'a> {
    feature: &'a DotMarker,
    center: Point,
}

impl<'a> Shape<'a> for DotShape<'a> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        match stage {
            Stage::MarkerCasing => {
                if self.feature.casing {
                    let mut sketch = canvas.sketch();
                    sketch.apply(
                        Circle::new(
                            self.center,
                            self.feature.size.radius(style)
                                + style.measures().class_track(
                                    &self.feature.class
                                ) * 0.5
                        )
                    );
                    //sketch.apply(Operator::DestinationOut);
                    sketch.apply(Color::WHITE);
                    sketch.fill();
                }
            }
            Stage::MarkerBase => {
                let radius = self.feature.size.radius(style);
                let sp = style.measures().class_track(&self.feature.class);
                match self.feature.inner {
                    Inner::Fill => {
                        canvas.sketch().apply(
                            Circle::new(self.center, radius)
                        ).apply(
                            style.primary_marker_color(&self.feature.class)
                        ).fill();
                    }
                    Inner::Stroke => {
                        canvas.sketch().apply(
                            Circle::new(self.center, radius - 0.3 * sp)
                        ).apply(
                            style.primary_marker_color(
                                &self.feature.class
                            )
                        ).apply(
                            LineWidth(0.6 * sp)
                        ).stroke();
                    }
                    Inner::None => { }
                }
            }
            _ => { }
        }
    }

    fn stages(&self) -> StageSet {
        let res = StageSet::from(Stage::MarkerBase);
        if self.feature.casing {
            res.add(Stage::MarkerCasing)
        }
        else {
            res
        }
    }
}


//------------ Size ----------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
enum Size {
    Small,
    #[default]
    Medium,
    Large,

}

impl Size {
    fn km_from_symbols(symbols: &mut SymbolSet) -> Self {
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
            Size::Small
        }
    }

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

    fn radius(self, style: &Style) -> f64 {
        style.measures().dt() * match self {
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
        else if !class.pax().is_full() {
            Inner::Stroke
        }
        else {
            Inner::Fill
        }
    }

    fn from_scope(scope: &Scope) -> Self {
        let class = scope.railway();
        if !class.pax().is_full() {
            Inner::Stroke
        }
        else {
            Inner::Fill
        }
    }
}

