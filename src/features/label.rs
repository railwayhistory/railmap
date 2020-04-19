use std::sync::Arc;
use kurbo::{Point, Rect};
use crate::canvas::Canvas;
use super::color::Color;
use super::path::Position;


//------------ Label ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Label {
    /// The position the label is attached to.
    position: Position,

    /// The layout to render
    layout: Layout,
}

impl Label {
    pub fn new(position: Position, layout: Layout) -> Self {
        Label { position, layout }
    }

    pub fn storage_bounds(&self) -> Rect {
        self.position.storage_bounds()
    }

    pub fn render(&self, canvas: &Canvas) {
        let (point, angle) = self.position.resolve_label(canvas);
        canvas.translate(point.x, point.y);
        canvas.rotate(angle);
        self.layout.render(Point::default(), canvas);
        canvas.identity_matrix();
    }
}


//------------ Layout --------------------------------------------------------

#[derive(Clone, Debug)]
pub enum Layout {
    Vbox(Vbox),
    Hbox(Hbox),
    Span(Span),
}

impl Layout {
    fn render(&self, point: Point, canvas: &Canvas) {
        match *self {
            Layout::Vbox(ref value) => value.render(point, canvas),
            Layout::Hbox(ref value) => value.render(point, canvas),
            Layout::Span(ref value) => value.render(point, canvas),
        }
    }

    /// The extent of the layout.
    ///
    /// The values are given relative to the layout’s reference point.
    fn extent(&self, canvas: &Canvas) -> Rect {
        match *self {
            Layout::Vbox(ref value) => value.extent(canvas),
            Layout::Hbox(ref value) => value.extent(canvas),
            Layout::Span(ref value) => value.extent(canvas),
        }
    }
}


//------------ Vbox ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Vbox {
    align: Align,
    lines: Vec<Layout>,
}

impl Vbox {
    pub fn new(align: Align, lines: Vec<Layout>) -> Self {
        Vbox { align, lines }
    }

    fn render(&self, mut point: Point, canvas: &Canvas) {
        for layout in &self.lines {
            let extent = layout.extent(canvas);
            point.y += extent.y1;
            match self.align {
                Align::Start => {
                    layout.render(
                        Point::new(
                            point.x + extent.x0, // x0 is negative.
                            point.y
                        ),
                        canvas
                    );
                }
                Align::Center => {
                    layout.render(
                        Point::new(
                            point.x - extent.width() / 2. - extent.x0,
                            point.y
                        ),
                        canvas
                    );
                }
                Align::Ref => {
                    layout.render(point, canvas);
                }
                Align::End => {
                    layout.render(
                        Point::new(
                            point.x - extent.x1,
                            point.y
                        ),
                        canvas
                    );
                }
            }
            point.y -= extent.y0; // y0 should be negative.
        }
    }

    fn extent(&self, canvas: &Canvas) -> Rect {
        let mut res = Rect::default();
        for layout in &self.lines {
            let extent = layout.extent(canvas);
            res.y1 += extent.height();
            match self.align {
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
        res
    }
}


//------------ Hbox ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Hbox {
    align: Align,
    spans: Vec<Layout>,
}

impl Hbox {
    pub fn new(align: Align, spans: Vec<Layout>) -> Self {
        Hbox { align, spans }
    }

    fn render(&self, mut point: Point, canvas: &Canvas) {
        for layout in &self.spans {
            let extent = layout.extent(canvas);
            point.x -= extent.x0;
            match self.align {
                Align::Start => {
                    layout.render(
                        Point::new(
                            point.x,
                            point.y - extent.y1
                        ),
                        canvas
                    )
                }
                Align::Center => {
                    layout.render(
                        Point::new(
                            point.x,
                            point.y - extent.height() / 2. - extent.y0
                        ),
                        canvas
                    )
                }
                Align::Ref => {
                    layout.render(point, canvas)
                }
                Align::End => {
                    layout.render(
                        Point::new(
                            point.x,
                            point.y - extent.y0
                        ),
                        canvas
                    )
                }
            }
            point.x += extent.x1;
        }
    }

    fn extent(&self, canvas: &Canvas) -> Rect {
        let mut res = Rect::default();
        for layout in &self.spans {
            let extent = layout.extent(canvas);
            res.x1 += extent.width();
            match self.align {
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
        res
    }
}


//------------ Span ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Span {
    font: Font,
    content: String,
}

impl Span {
    pub fn new(font: Font, content: String) -> Self {
        Span { font, content }
    }

    fn render(&self, point: Point, canvas: &Canvas) {
        canvas.set_font_face(canvas.fira());
        canvas.set_font_size(self.font.0.size * canvas.canvas_bp());
        self.font.0.color.apply(canvas);
        canvas.move_to(point.x, point.y);
        canvas.show_text(&self.content);
    }

    fn extent(&self, canvas: &Canvas) -> Rect {
        canvas.set_font_face(canvas.fira());
        canvas.set_font_size(self.font.0.size * canvas.canvas_bp());

        // We take the width from the text extents and the height from the
        // font extens. This assumes that the text is one line exactly.
        let text = canvas.text_extents(&self.content);
        let font = canvas.font_extents();
 
        // First, y axis is upwards, so negative height is below the base line
        // and positive height is above.
        //
        // The font height may be bigger than ascent plus descent so we correct
        // the descent for this.
        let top = font.ascent;
        let bottom = font.ascent - font.height;

        // For the width, we use the text’s x_advance. This should consider the
        // intended width instead of the inked width.
        let left = 0.;
        let right = text.x_advance;

        Rect::new(left, bottom, right, top)
    }
}


//------------ Align ------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum Align {
    Start,
    Center,
    Ref,
    End
}

impl Align {
    pub fn try_from_str(s: &str) -> Option<Align> {
        match s {
            "start" => Some(Align::Start),
            "center" => Some(Align::Center),
            "ref" => Some(Align::Ref),
            "end" => Some(Align::End),
            _ => None,         
        }
    }
}


#[derive(Clone, Debug)]
pub struct Font(Arc<FontInfo>);

#[derive(Clone, Debug)]
pub struct FontInfo {
    color: Color,
    size: f64,
}

impl FontInfo {
    pub fn new(color: Color, size: f64) -> Self {
        FontInfo { color, size }
    }

    pub fn into_font(self) -> Font {
        Font(Arc::new(self))
    }
}

