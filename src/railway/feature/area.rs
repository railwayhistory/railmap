//! Rendering an area.

use femtomap::world;
use femtomap::path::Trace;
use femtomap::render::Canvas;
use crate::railway::class::Railway;
use crate::railway::style::Style;
use super::{AnyShape, Category, Group, Feature};

//------------ AreaContour ---------------------------------------------------

/// A contour drawing an area.
pub struct AreaContour {
    class: Railway,
    trace: Trace,
}

impl AreaContour {
    pub fn new(class: Railway, trace: Trace) -> Self {
        AreaContour { class, trace }
    }
}

impl Feature for AreaContour {
    fn storage_bounds(&self) -> world::Rect {
        self.trace.storage_bounds()
    }

    fn group(&self) -> Group {
        Group::with_railway(Category::Back, &self.class)
    }

    fn shape(
        &self, style: &Style, _canvas: &Canvas
    ) -> AnyShape {
        let color = style.track_color(&self.class);
        let outline = self.trace.outline(style);

        AnyShape::single_stage(move |_style: &Style, canvas: &mut Canvas| {
            let mut canvas = canvas.sketch();
            canvas.apply(&outline);
            canvas.apply(color);
            canvas.fill();
        })
    }
}


//------------ PlatformContour -----------------------------------------------

/// A contour drawing an area.
pub struct PlatformContour {
    class: Railway,
    trace: Trace,
}

impl PlatformContour {
    pub fn new(class: Railway, trace: Trace) -> Self {
        PlatformContour { class, trace }
    }
}

impl Feature for PlatformContour {
    fn storage_bounds(&self) -> world::Rect {
        self.trace.storage_bounds()
    }

    fn group(&self) -> Group {
        Group::with_railway(Category::Back, &self.class)
    }

    fn shape(
        &self, style: &Style, _canvas: &Canvas
    ) -> AnyShape {
        let color = style.track_color(&self.class);
        let outline = self.trace.outline(style);

        AnyShape::single_stage(move |_style: &Style, canvas: &mut Canvas| {
            let mut canvas = canvas.sketch();
            canvas.apply(&outline);
            canvas.apply(color);
            canvas.fill();
        })
    }
}

