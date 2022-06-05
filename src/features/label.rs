//! Arrangements of text placed on the map.

use std::{cmp, fmt};
use std::sync::Arc;
use kurbo::{Point, Rect};
use crate::canvas::Canvas;
use crate::import::eval::SymbolSet;
use super::path::Position;

//------------ Label ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Label {
    /// The position the label is attached to.
    position: Position,

    /// Is the position’s base direction along the path?
    ///
    /// If this is `false`, the base direction is to the right.
    on_path: bool,

    /// The layout to render
    layout: Layout,
}

impl Label {
    pub fn new(
        position: Position, on_path: bool, layout: Layout
    ) -> Self {
        Label { position, on_path, layout }
    }

    pub fn storage_bounds(&self) -> Rect {
        self.position.storage_bounds()
    }

    pub fn render(&self, canvas: &Canvas) {
        let (extent, depth) = self.layout.extent(canvas);
        if depth == 0 {
            return
        }

        let (point, angle) = self.position.resolve_label(
            canvas, self.on_path
        );
        canvas.translate(point.x, point.y);
        canvas.rotate(angle);

        for depth in (0..depth).rev() {
            self.layout.render(
               canvas, depth, Point::default(), extent, extent
            );
        }
        canvas.identity_matrix();
    }
}


//------------ Layout --------------------------------------------------------

/// A layout is an arrangement of text.
///
/// The layout can contain final content – a span, an hrule, or a vrule – or
/// a sequence of other layouts arranged horizontally or vertically – an hbox
/// or vbox, respecitvely.
///
/// For a given canvas, a layout can determine its _extent_ which describes
/// how far the layout’s content would stretch away from a central point
/// called the _anchor_ in all four directions. The extent is used to stack
/// layouts: multiple layouts are placed in such a way that their extents
/// touch.
#[derive(Clone, Debug)]
pub struct Layout {
    /// The content of the layout.
    content: Content,
}

#[derive(Clone, Debug)]
enum Content {
    Vbox(Vbox),
    Hbox(Hbox),
    Span(Span),
}

impl Layout {
    pub fn vbox(halign: Align, valign: Align, lines: Vec<Layout>) -> Self {
        Self::new(Content::Vbox(Vbox::new(halign, valign, lines)))
    }

    pub fn hbox(halign: Align, valign: Align, spans: Vec<Layout>) -> Self {
        Self::new(Content::Hbox(Hbox::new(halign, valign, spans)))
    }

    pub fn span(rule: SpanRule) -> Self {
        Self::new(Content::Span(Span::new(rule)))
    }

    fn new(content: Content) -> Self {
        Layout { content }
    }

    fn render(
        &self, canvas: &Canvas, depth: usize, point: Point,
        extent: Rect, outer: Rect,
    ) {
        match self.content {
            Content::Vbox(ref v)
                => v.render(canvas, depth, point, extent),
            Content::Hbox(ref v)
                => v.render(canvas, depth, point, extent),
            Content::Span(ref v)
                => v.render(canvas, depth, point, extent, outer),
        }
    }

    /// The extent of the layout.
    ///
    /// The values are given relative to the layout’s reference point.
    fn extent(&self, canvas: &Canvas) -> (Rect, usize) {
        match self.content {
            Content::Vbox(ref v) => v.extent(canvas),
            Content::Hbox(ref v) => v.extent(canvas),
            Content::Span(ref v) => v.extent(canvas),
        }
    }
}


//------------ Vbox ----------------------------------------------------------

#[derive(Clone, Debug)]
struct Vbox {
    halign: Align,
    valign: Align,
    lines: Vec<Layout>,
}

impl Vbox {
    fn new(halign: Align, valign: Align, lines: Vec<Layout>) -> Self {
        Vbox { halign, valign, lines }
    }

    fn render(
        &self, canvas: &Canvas, depth: usize, point: Point, extent: Rect
    ) {
        let outer = extent;
        self.render_op(canvas, point, extent, |layout, point, extent| {
            layout.render(canvas, depth, point, extent, outer)
        })
    }

    fn render_op<F: Fn(&Layout, Point, Rect)>(
        &self, canvas: &Canvas, mut point: Point, extent: Rect, op: F
    ) {
        point.y += extent.y0;
        for layout in &self.lines {
            let (extent, _) = layout.extent(canvas);
            point.y -= extent.y0;
            match self.halign {
                Align::Start => {
                    op(
                        layout, 
                        Point::new(
                            point.x + extent.x0, // x0 is negative.
                            point.y
                        ),
                        extent
                    );
                }
                Align::Center => {
                    op(
                        layout, 
                        Point::new(
                            point.x - extent.width() / 2. - extent.x0,
                            point.y
                        ),
                        extent
                    );
                }
                Align::Ref => {
                    op(layout, point, extent);
                }
                Align::End => {
                    op(
                        layout, 
                        Point::new(
                            point.x - extent.x1,
                            point.y
                        ),
                        extent
                    );
                }
            }
            point.y += extent.y1;
        }
    }

    fn extent(&self, canvas: &Canvas) -> (Rect, usize) {
        let mut res = Rect::default();
        let mut max_depth = 0;
        let mut top = None;
        for layout in &self.lines {
            let (extent, depth) = layout.extent(canvas);
            max_depth = cmp::max(max_depth, depth);
            res.y1 += extent.height();
            if top.is_none() {
                top = Some(extent.y0);
            }
            match self.halign {
                Align::Start => {
                    res.x1 = res.x1.max(extent.width());
                }
                Align::Center => {
                    let width = extent.width() / 2.;
                    res.x0 = res.x0.min(-width);
                    res.x1 = res.x1.max(width);
                }
                Align::Ref => {
                    res.x0 = res.x0.min(extent.x0);
                    res.x1 = res.x1.max(extent.x1);
                }
                Align::End => {
                    res.x0 = res.x0.min(-extent.width())
                }
            }
        }
        match self.valign {
            Align::Start => { }
            Align::Center => {
                res.y0 = -res.y1 / 2.;
                res.y1 = res.y1 / 2.
            }
            Align::Ref => {
                if let Some(top) = top {
                    res.y0 = top;
                    res.y1 += top;
                }
            }
            Align::End => {
                res.y0 = -res.y1;
                res.y1 = 0.
            }
        }
        (res, max_depth)
    }
}


//------------ Hbox ----------------------------------------------------------

/// A sequence of layouts stacked horizontally.
#[derive(Clone, Debug)]
struct Hbox {
    halign: Align,
    valign: Align,
    spans: Vec<Layout>,
}

impl Hbox {
    fn new(halign: Align, valign: Align, spans: Vec<Layout>) -> Self {
        Hbox { halign, valign, spans }
    }

    fn render(
        &self, canvas: &Canvas, depth: usize, point: Point, extent: Rect
    ) {
        let outer = extent;
        self.render_op(canvas, point, extent, |layout, point, extent| {
            layout.render(canvas, depth, point, extent, outer)
        });
    }

    fn render_op<F: Fn(&Layout, Point, Rect)>(
        &self, canvas: &Canvas, mut point: Point, extent: Rect, op: F
    ) {
        point.x += extent.x0;
        for layout in &self.spans {
            let (extent, _) = layout.extent(canvas);
            point.x -= extent.x0;
            match self.valign {
                Align::Start => {
                    op(
                        layout,
                        Point::new(
                            point.x,
                            point.y - extent.y0
                        ),
                        extent
                    )
                }
                Align::Center => {
                    op(
                        layout,
                        Point::new(
                            point.x,
                            point.y - 0.5 * extent.height() - extent.y0
                        ),
                        extent
                    )
                }
                Align::Ref => {
                    op(layout, point, extent)
                }
                Align::End => {
                    op(
                        layout,
                        Point::new(
                            point.x,
                            point.y - extent.y1
                        ),
                        extent
                    )
                }
            }
            point.x += extent.x1;
        }
    }

    fn extent(&self, canvas: &Canvas) -> (Rect, usize) {
        let mut res = Rect::default();
        let mut max_depth = 0;
        let mut left = None;
        for layout in &self.spans {
            let (extent, depth) = layout.extent(canvas);
            max_depth = cmp::max(max_depth, depth);
            res.x1 += extent.width();
            if left.is_none() {
                left = Some(extent.x0);
            }
            match self.valign {
                Align::Start => {
                    res.y1 = res.y1.max(extent.height());
                }
                Align::Center => {
                    let height = extent.height() / 2.;
                    res.y0 = res.y0.min(-height);
                    res.y1 = res.y1.max(height);
                }
                Align::Ref => {
                    res.y0 = res.y0.min(extent.y0);
                    res.y1 = res.y1.max(extent.y1);
                }
                Align::End => {
                    res.y0 = res.y0.min(-extent.height());
                }
            }
        }
        match self.halign {
            Align::Start => { }
            Align::Center => {
                res.x0 = -res.x1 / 2.;
                res.x1 = res.x1 / 2.
            }
            Align::Ref => {
                if let Some(left) = left {
                    res.x0 = left;
                    res.x1 += left;
                }
            }
            Align::End => {
                res.x0 = -res.x1;
                res.x1 = 0.
            }
        }
        (res, max_depth)
    }
}


//------------ Span ----------------------------------------------------------

/// A run of text rendered with the same properties.
///
/// For the moment, we only support horizontal left-to-right text with no line
/// breaks.
///
/// The extent is anchored at the base line and the start of the first
/// character. It goes up by the font’s ascent, down by the font’s line height
/// minus the ascent. It does not goes left but the full advance to the right.
#[derive(Clone, Debug)]
struct Span {
    rule: SpanRule,
}

impl Span {
    fn new(rule: SpanRule) -> Span {
        Span { rule }
    }

    fn extent(&self, canvas: &Canvas) -> (Rect, usize) {
        self.rule.0.extent(canvas)
    }

    fn render(
        &self, canvas: &Canvas, depth: usize,
        point: Point, extent: Rect, outer: Rect,
    ) {
        self.rule.0.render(canvas, depth, point, extent, outer)
    }
}


pub trait RenderSpan: Send + Sync + 'static {
    /// Returns the extent and depth of the span.
    ///
    /// The extent describes the natural spread of the span on the
    /// canvas away from the anchor point. The depth describes the number
    /// of rendering rounds the span needs to properly render its content.
    fn extent(&self, canvas: &Canvas) -> (Rect, usize);

    /// Renders one round of the span.
    ///
    /// This method will be called multiple times starting with the
    /// maximum depth of the entire layout and then with decreasing depths.
    /// Thus, the depth value may be larger than the depth the span
    /// returned itself in `extent`. The span is allowed to draw at these
    /// depths as well.
    ///
    /// Note that the smallest depth is 0. I.e., if you returned 2 in
    /// `extent` for your depth and there is no spans with greater depth,
    /// the `render` method will be called with depth 1 first and then with
    /// depth 0 again.
    fn render(
        &self, canvas: &Canvas, depth: usize, point: Point,
        extent: Rect, outer: Rect,
    );

    fn into_rule(self) -> SpanRule
    where Self: Sized {
        SpanRule(Arc::new(self))
    }
}

#[derive(Clone)]
pub struct SpanRule(Arc<dyn RenderSpan>);

impl fmt::Debug for SpanRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SpanRule(...)")
    }
}


//------------ Align ---------------------------------------------------------

/// How layouts are stacked in a box.
#[derive(Clone, Copy, Debug)]
pub enum Align {
    /// The upper or left extent is aligned.
    Start,

    /// The center of each layout is aligned.
    Center,

    /// The anchors of the layouts are aligned.
    Ref,

    /// The lower or right extens is aligned.
    End
}

impl Align {
    pub fn h_from_symbols(symbols: &mut SymbolSet) -> Option<Align> {
        if symbols.take("left") {
            Some(Align::Start)
        }
        else if symbols.take("center") {
            Some(Align::Center)
        }
        else if symbols.take("sep") {
            Some(Align::Ref)
        }
        else if symbols.take("right") {
            Some(Align::End)
        }
        else {
            None
        }
    }

    pub fn v_from_symbols(symbols: &mut SymbolSet) -> Option<Align> {
        if symbols.take("top") {
            Some(Align::Start)
        }
        else if symbols.take("middle") {
            Some(Align::Center)
        }
        else if symbols.take("base") {
            Some(Align::Ref)
        }
        else if symbols.take("bottom") {
            Some(Align::End)
        }
        else {
            None
        }
    }
}

