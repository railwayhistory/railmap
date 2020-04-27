use kurbo::{Point, Rect};
use crate::canvas::{Canvas, FontFace};
use super::color::Color;
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
    pub fn new(position: Position, on_path: bool, layout: Layout) -> Self {
        Label { position, on_path, layout }
    }

    pub fn storage_bounds(&self) -> Rect {
        self.position.storage_bounds()
    }

    pub fn render(&self, canvas: &Canvas) {
        let (point, angle) = self.position.resolve_label(
            canvas, self.on_path
        );
        canvas.translate(point.x, point.y);
        canvas.rotate(angle);

        let font = FinalFont::default();

        // Clear out the background of the label.
        let extent = self.layout.extent(&font, canvas);
        canvas.set_operator(cairo::Operator::Clear);
        canvas.move_to(extent.x0 - canvas.canvas_bp(), extent.y0);
        canvas.line_to(extent.x0 - canvas.canvas_bp(), extent.y1);
        canvas.line_to(extent.x1 + canvas.canvas_bp(), extent.y1);
        canvas.line_to(extent.x1 + canvas.canvas_bp(), extent.y0);
        canvas.close_path();
        canvas.fill();
        canvas.set_operator(cairo::Operator::Over);

        self.layout.render(Point::default(), &font, extent, canvas);
        canvas.identity_matrix();
    }
}


//------------ Layout --------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Layout {
    /// The content of the layout.
    content: Content,

    /// The font to use for the layout.
    font: Font,
}

impl Layout {
    fn new(content: Content, font: Font) -> Self {
        Layout { content, font }
    }

    pub fn vbox(
        halign: Align, valign: Align, font: Font, lines: Stack,
    ) -> Self {
        Layout::new(Content::Vbox(Vbox { halign, valign, lines }), font)
    }

    pub fn hbox(
        halign: Align, valign: Align, font: Font, spans: Stack
    ) -> Self {
        Layout::new(Content::Hbox(Hbox { halign, valign, spans }), font)
    }

    pub fn span(font: Font, content: String) -> Self {
        Layout::new(Content::Span(Span { content }), font)
    }

    fn render(
        &self, point: Point, font: &FinalFont, extent: Rect, canvas: &Canvas
    ) {
        let font = font.update(&self.font);
        match self.content {
            Content::Vbox(ref v) => v.render(point, &font, extent, canvas),
            Content::Hbox(ref v) => v.render(point, &font, extent, canvas),
            Content::Span(ref v) => v.render(point, &font, extent, canvas),
        }
    }

    /// The extent of the layout.
    ///
    /// The values are given relative to the layout’s reference point.
    fn extent(&self, font: &FinalFont, canvas: &Canvas) -> Rect {
        let font = font.update(&self.font);
        match self.content {
            Content::Vbox(ref v) => v.extent(&font, canvas),
            Content::Hbox(ref v) => v.extent(&font, canvas),
            Content::Span(ref v) => v.extent(&font, canvas),
        }
    }
}


//------------ Content -------------------------------------------------------

#[derive(Clone, Debug)]
enum Content {
    Vbox(Vbox),
    Hbox(Hbox),
    Span(Span),
}


//------------ Vbox ----------------------------------------------------------

#[derive(Clone, Debug)]
struct Vbox {
    halign: Align,
    valign: Align,
    lines: Stack,
}

impl Vbox {
    fn render(
        &self, mut point: Point, font: &FinalFont, extent: Rect, canvas: &Canvas
    ) {
        point.y += extent.y0;
        for layout in &self.lines {
            let extent = layout.extent(font, canvas);
            point.y -= extent.y0;
            match self.halign {
                Align::Start => {
                    layout.render(
                        Point::new(
                            point.x + extent.x0, // x0 is negative.
                            point.y
                        ),
                        font, extent, canvas
                    );
                }
                Align::Center => {
                    layout.render(
                        Point::new(
                            point.x - extent.width() / 2. - extent.x0,
                            point.y
                        ),
                        font, extent, canvas
                    );
                }
                Align::Ref => {
                    layout.render(point, font, extent, canvas);
                }
                Align::End => {
                    layout.render(
                        Point::new(
                            point.x - extent.x1,
                            point.y
                        ),
                        font, extent, canvas
                    );
                }
            }
            point.y += extent.y1;
        }
    }

    fn extent(&self, font: &FinalFont, canvas: &Canvas) -> Rect {
        let mut res = Rect::default();
        let mut top = None;
        for layout in &self.lines {
            let extent = layout.extent(font, canvas);
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
        res
    }
}


//------------ Hbox ----------------------------------------------------------

#[derive(Clone, Debug)]
struct Hbox {
    halign: Align,
    valign: Align,
    spans: Vec<Layout>,
}

impl Hbox {
    fn render(
        &self, mut point: Point, font: &FinalFont, extent: Rect, canvas: &Canvas
    ) {
        point.x += extent.x0;
        for layout in &self.spans {
            let extent = layout.extent(font, canvas);
            point.x -= extent.x0;
            match self.valign {
                Align::Start => {
                    layout.render(
                        Point::new(
                            point.x,
                            point.y - extent.y0
                        ),
                        font, extent, canvas
                    )
                }
                Align::Center => {
                    layout.render(
                        Point::new(
                            point.x,
                            point.y - 0.5 * extent.height() - extent.y0
                        ),
                        font, extent, canvas
                    )
                }
                Align::Ref => {
                    layout.render(point, font, extent, canvas)
                }
                Align::End => {
                    layout.render(
                        Point::new(
                            point.x,
                            point.y - extent.y1
                        ),
                        font, extent, canvas
                    )
                }
            }
            point.x += extent.x1;
        }
    }

    fn extent(&self, font: &FinalFont, canvas: &Canvas) -> Rect {
        let mut res = Rect::default();
        let mut left = None;
        for layout in &self.spans {
            let extent = layout.extent(font, canvas);
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
        res
    }
}


//------------ Span ----------------------------------------------------------

#[derive(Clone, Debug)]
struct Span {
    content: String,
}

impl Span {
    fn render(
        &self, point: Point, font: &FinalFont, _extent: Rect, canvas: &Canvas
    ) {
        font.apply(canvas);
        canvas.move_to(point.x, point.y);
        canvas.show_text(&self.content);
    }

    fn extent(&self, font: &FinalFont, canvas: &Canvas) -> Rect {
        font.apply(canvas);

        // We take the width from the text extents and the height from the
        // font extents. This assumes that the text is one line exactly.
        let text = canvas.text_extents(&self.content);
        let font = canvas.font_extents();
 
        // The font height may be bigger than ascent plus descent so we correct
        // the descent for this.
        let top = -font.ascent;
        let bottom = top + font.height;

        // For the width, we use the text’s x_advance. This should consider the
        // intended width instead of the inked width.
        let left = 0.;
        let right = text.x_advance;

        Rect::new(left, top, right, bottom)
    }
}


//------------ Align ---------------------------------------------------------

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


//------------ Font ----------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Font {
    face: Option<FontFace>,
    color: Option<Color>,
    size: Option<f64>,
}

impl Font {
    pub fn new(
        face: Option<FontFace>, color: Option<Color>, size: Option<f64>
    ) -> Self {
        Font { face, color, size }
    }

    pub fn normal(color: Color, size: f64) -> Self {
        Font::new(None, Some(color), Some(size))
    }

    pub fn black(size: f64) -> Self {
        Self::new(None, Some(Color::BLACK), Some(size))
    }
}

//------------ FinalFont -----------------------------------------------------

/// A font ready to be applied to a canvas.
#[derive(Clone, Debug)]
struct FinalFont {
    face: FontFace,
    color: Color,
    size: f64,
}

impl Default for FinalFont {
    fn default() -> Self {
        FinalFont {
            face: FontFace::default(),
            color: Color::BLACK,
            size: 10.
        }
    }
}

impl FinalFont {
    fn apply(&self, canvas: &Canvas) {
        canvas.apply_font(Default::default(), self.size);
        self.color.apply(canvas);
    }

    fn update(&self, other: &Font) -> Self {
        let mut res = self.clone();
        if let Some(face) = other.face {
            res.face = face
        }
        if let Some(color) = other.color {
            res.color = color
        }
        if let Some(size) = other.size {
            res.size = size
        }
        res
    }
}


//------------ Stack ---------------------------------------------------------

pub type Stack = Vec<Layout>;

