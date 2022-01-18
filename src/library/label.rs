//! Making and rendering label features.

use kurbo::{Point, Rect};
use crate::canvas::{Canvas, FontFace};
use crate::features::color::Color;
use crate::features::label::{Label, Layout, RenderSpan};
use crate::features::path::{Distance, Position};

pub use crate::features::label::Align;


//------------ LayoutBuilder -------------------------------------------------

#[derive(Clone, Debug)]
pub struct LayoutBuilder {
    /// The content of the builder.
    content: ContentBuilder,

    /// The properties of the builder.
    properties: PropertiesBuilder,
}

#[derive(Clone, Debug)]
enum ContentBuilder {
    Vbox {
        halign: Align,
        valign: Align,
        lines: StackBuilder,
    },
    Hbox {
        halign: Align,
        valign: Align,
        spans: StackBuilder,
    },
    Span {
        content: String,
    },
    Hrule {
        width: Distance,
    },
}

impl LayoutBuilder {
    fn new(content: ContentBuilder, properties: PropertiesBuilder) -> Self {
        LayoutBuilder { content, properties }
    }

    pub fn vbox(
        halign: Align, valign: Align, properties: PropertiesBuilder,
        lines: StackBuilder
    ) -> Self {
        LayoutBuilder::new(
            ContentBuilder::Vbox { halign, valign, lines },
            properties
        )
    }

    pub fn hbox(
        halign: Align, valign: Align, properties: PropertiesBuilder,
        spans: StackBuilder
    ) -> Self {
        LayoutBuilder::new(
            ContentBuilder::Hbox { halign, valign, spans },
            properties
        )
    }

    pub fn span(
        content: String, properties: PropertiesBuilder
    ) -> Self {
        LayoutBuilder::new(
            ContentBuilder::Span { content },
            properties
        )
    }

    pub fn hrule(
        width: Distance, properties: PropertiesBuilder
    ) -> Self {
        LayoutBuilder::new(
            ContentBuilder::Hrule { width },
            properties
        )
    }

    pub fn properties_mut(&mut self) -> &mut PropertiesBuilder {
        &mut self.properties
    }

    pub fn into_label(
        self, position: Position, on_path: bool, base: Properties
    ) -> Label {
        Label::new(position, on_path, self.into_layout(&base))
    }

    pub fn into_layout(
        self, base: &Properties
    ) -> Layout {
        let properties = base.updated(&self.properties);
        match self.content {
            ContentBuilder::Vbox { halign, valign, lines } => {
                Layout::vbox(
                    halign, valign,
                    lines.into_iter().map(|line| {
                        line.into_layout(&properties)
                    }).collect(),
                )
            }
            ContentBuilder::Hbox { halign, valign, spans } => {
                Layout::hbox(
                    halign, valign,
                    spans.into_iter().map(|line| {
                        line.into_layout(&properties)
                    }).collect(),
                )
            }
            ContentBuilder::Span { content } => {
                Layout::span(TextSpan::new(content, properties).into_rule())
            }
            ContentBuilder::Hrule { width } => {
                Layout::span(
                    HruleSpan::new(width, properties.font.color).into_rule()
                )
            }
        }
    }
}


//------------ StackBuilder --------------------------------------------------

pub type StackBuilder = Vec<LayoutBuilder>;


//------------ PropertiesBuilder ---------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct PropertiesBuilder {
    /// Changes to the parent’s font.
    font: FontBuilder,

    /// The background for this layout.
    ///
    /// If `None`, it is inherited from the parent.
    background: Option<Background>,
}

impl PropertiesBuilder {
    pub fn set_background(&mut self, background: Background) -> &mut Self { 
        self.background = Some(background);
        self
    }

    pub fn font_mut(&mut self) -> &mut FontBuilder {
        &mut self.font
    }

    pub fn into_properties(self) -> Properties {
        Properties::default().updated(&self)
    }
}

impl From<FontBuilder> for PropertiesBuilder {
    fn from(font: FontBuilder) -> PropertiesBuilder {
        PropertiesBuilder {
            font,
            background: None
        }
    }
}

impl From<Background> for PropertiesBuilder {
    fn from(background: Background) -> Self {
        PropertiesBuilder {
            font: Default::default(),
            background: Some(background)
        }
    }
}


//------------ FontBuilder ---------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct FontBuilder {
    face: Option<FontFace>,
    color: Option<Color>,
    size: Option<f64>,
}

impl FontBuilder {
    pub fn new(
        face: Option<FontFace>, color: Option<Color>, size: Option<f64>
    ) -> Self {
        FontBuilder { face, color, size }
    }

    pub fn normal(color: Color, size: f64) -> Self {
        Self::new(None, Some(color), Some(size))
    }

    pub fn bold(color: Color, size: f64) -> Self {
        Self::new(FontFace::bold(), Some(color), Some(size))
    }

    pub fn defaults(&mut self, font: &FontBuilder) {
        if self.face.is_none() {
            self.face = font.face
        }
        if self.color.is_none() {
            self.color = font.color
        }
        if self.size.is_none() {
            self.size = font.size
        }
    }
}


//------------ Properties ----------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Properties {
    font: Font,
    background: Background
}

impl Properties {
    pub fn new(font: Font, background: Background) -> Self {
        Properties { font, background }
    }

    pub fn updated(&self, update: &PropertiesBuilder) -> Self {
        Properties {
            font: self.font.update(&update.font),
            background: update.background.unwrap_or_else(|| self.background),
        }
    }
}

impl From<Background> for Properties {
    fn from(background: Background) -> Self {
        Properties::new(Font::default(), background)
    }
}


//------------ Font ----------------------------------------------------------

/// A font ready to be applied to a canvas.
#[derive(Clone, Debug)]
pub struct Font {
    face: FontFace,
    color: Color,
    size: f64,
}

impl Default for Font {
    fn default() -> Self {
        Font {
            face: FontFace::default(),
            color: Color::BLACK,
            size: 10.
        }
    }
}

impl Font {
    fn apply(&self, canvas: &Canvas) {
        canvas.apply_font(self.face, self.size);
        self.color.apply(canvas);
    }

    pub fn update(&self, other: &FontBuilder) -> Self {
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

    /*
    fn clearance(&self, canvas: &Canvas) -> Point {
        Point::new(canvas.canvas_bp(), canvas.canvas_bp())
    }
    */
}


//------------ Background ----------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum Background {
    /// Don’t do anything to the background.
    Transparent,

    /// Clear the background.
    Clear,

    /// Fill the background with the given color
    Fill(Color),
}

impl Default for Background {
    fn default() -> Self {
        Background::Transparent
    }
}


//------------ TextSpan ------------------------------------------------------

/// The rendering rule for a span of text.
struct TextSpan {
    text: String,
    properties: Properties,
}

impl TextSpan {
    fn new(text: String, properties: Properties) -> Self {
        TextSpan { text, properties }
    }
}

impl RenderSpan for TextSpan {
    fn extent(&self, canvas: &Canvas) -> (Rect, usize) {
        self.properties.font.apply(canvas);

        // We take the width from the text extents and the height from the
        // font extents. This assumes that the text is one line exactly.
        let text = canvas.text_extents(&self.text);
        let font = canvas.font_extents();
 
        // The font height may be bigger than ascent plus descent so we correct
        // the descent for this.
        let top = -font.ascent;
        let bottom = top + font.height;

        // For the width, we use the text’s x_advance. This should consider the
        // intended width instead of the inked width.
        let left = 0.;
        let right = text.x_advance;

        (Rect::new(left, top, right, bottom), 2)
    }

    fn render(
        &self, canvas: &Canvas, depth: usize, point: Point,
        _extent: Rect, _outer: Rect,
    ) {
        match depth {
            1 =>  {
                let cap = canvas.get_line_cap();
                let join = canvas.get_line_join();
                self.properties.font.apply(canvas);
                Color::WHITE.apply(canvas);
                canvas.set_line_width(self.properties.font.size);
                canvas.set_line_cap(cairo::LineCap::Butt);
                canvas.set_line_join(cairo::LineJoin::Bevel);
                canvas.move_to(point.x, point.y);
                canvas.text_path(&self.text);
                canvas.stroke();
                canvas.set_line_join(join);
                canvas.set_line_cap(cap);
            }
            0 => {
                self.properties.font.apply(canvas);
                canvas.move_to(point.x, point.y);
                canvas.show_text(&self.text);
            }
            _ => { }
        }
    }
}



//------------ HruleSpan -----------------------------------------------------

/// The rendering rule for a horizontal bar
struct HruleSpan {
    width: Distance,
    color: Color,
}

impl HruleSpan {
    fn new(width: Distance, color: Color) -> Self {
        HruleSpan { width, color }
    }

    fn resolved_width(&self, canvas: &Canvas) -> f64 {
        self.width.canvas.map(|width| width * canvas.canvas_bp()).unwrap_or(0.)
    }
}

impl RenderSpan for HruleSpan {
    fn extent(&self, canvas: &Canvas) -> (Rect, usize) {
        let height = self.resolved_width(canvas) / 2.;
        (Rect::new(0., -height, 0., height), 1)
    }

    fn render(
        &self, canvas: &Canvas, depth: usize, point: Point,
        _extent: Rect, outer: Rect,
    ) {
        if depth == 0 {
            canvas.set_line_width(self.resolved_width(canvas));
            canvas.move_to(point.x + outer.x0, point.y);
            canvas.line_to(point.x + outer.x1, point.y);
            self.color.apply(canvas);
            canvas.stroke()
        }
    }
}

