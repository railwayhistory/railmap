/// A feature drawing a marker.
use std::fmt;
use std::sync::Arc;
use kurbo::Rect;
use crate::canvas::Canvas;
use super::path::Position;

//------------ Marker --------------------------------------------------------

/// A feature drawing a marker.
///
pub struct Marker {
    /// The position the marker is attached to.
    position: Position,

    /// The rendering rule for the marker.
    rule: MarkerRule,
}

impl Marker {
    pub fn new(position: Position, rule: MarkerRule) -> Self {
        Marker { position, rule }
    }

    pub fn storage_bounds(&self) -> Rect {
        self.position.storage_bounds()
    }

    pub fn render(&self, canvas: &Canvas) {
        self.rule.0.render(canvas, &self.position)
    }
}


//------------ RenderMarker --------------------------------------------------

pub trait RenderMarker: Send + Sync + 'static {
    fn render(&self, canvas: &Canvas, position: &Position);

    fn into_rule(self) -> MarkerRule
    where Self: Sized {
        MarkerRule(Arc::new(self))
    }
}

impl<F: Fn(&Canvas, &Position) + Send + Sync + 'static> RenderMarker for F {
    fn render(&self, canvas: &Canvas, position: &Position) {
        (*self)(canvas, position)
    }
}


//------------ MarkerRule ----------------------------------------------------

#[derive(Clone)]
pub struct MarkerRule(Arc<dyn RenderMarker>);

impl fmt::Debug for MarkerRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MarkerRule(...)")
    }
}

impl<R: RenderMarker> From<R> for MarkerRule {
    fn from(rule: R) -> Self {
        MarkerRule(Arc::new(rule))
    }
}

