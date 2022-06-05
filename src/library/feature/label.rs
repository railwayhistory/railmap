//! Making and rendering label features.

use kurbo::{Point, Rect};
use crate::canvas::{
    Canvas, FontFamily, FontFace, FontSlant, FontStretch, FontWeight,
};
use crate::features::color::Color;
use crate::features::label::{Label, Layout, RenderSpan};
use crate::features::path::{Distance, Position};
use crate::import::eval;
use crate::import::Failed;
use super::super::class::{Class, Status};

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

    pub fn rebase_properties(&mut self, base: &PropertiesBuilder) {
        self.properties.rebase(base)
    }

    pub fn into_label(
        self, position: Position, on_path: bool, base: Properties
    ) -> Label {
        Label::new(position, on_path, self.into_layout(&base))
    }

    pub fn into_layout(
        self, base: &Properties
    ) -> Layout {
        let properties = base.update(&self.properties);
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
                    HruleSpan::new(width, properties.class).into_rule()
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
    /// Changes to the parent’s font face.
    font: FontFaceBuilder,

    /// Changes to the parent’s font size.
    size: Option<FontSize>,

    /// Changes to the parent’s class.
    class: Class,
}

impl PropertiesBuilder {
    pub fn from_arg(
        arg: eval::Expression,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.into_symbol_set(err)?.0;
        let res = Self::from_symbols(&mut symbols);
        symbols.check_exhausted(err)?;
        Ok(res)
    }

    pub fn from_symbols(symbols: &mut eval::SymbolSet) -> Self {
        let mut res = PropertiesBuilder {
            font: FontFaceBuilder::from_symbols(symbols),
            size: FontSize::from_symbols(symbols),
            class: Class::from_symbols(symbols),
        };
        if symbols.take("former") {
            res.class.set_status(Status::Removed)
        }
        res
    }

    pub fn with_size(size: FontSize) -> Self {
        PropertiesBuilder {
            size: Some(size),
            .. Default::default()
        }
    }

    /// Injects `base` before this builder.
    ///
    /// Uses all changes in `base` for things not changed by `self`.
    pub fn rebase(&mut self, base: &PropertiesBuilder) {
        self.font = base.font.update(self.font);
        if self.size.is_none() {
            self.size = base.size
        }
        self.class = base.class.update(&self.class)
    }
}


//------------ FontFaceBuilder -----------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
struct FontFaceBuilder {
    family: Option<FontFamily>,
    stretch: Option<FontStretch>,
    slant: Option<FontSlant>,
    weight: Option<FontWeight>,
}

impl FontFaceBuilder {
    fn from_symbols(symbols: &mut eval::SymbolSet) -> Self {
        use self::FontSlant::*;
        use self::FontStretch::*;
        use self::FontWeight::*;

        FontFaceBuilder {
            family: None,
            stretch: {
                if symbols.take("condensed") { Some(Condensed) }
                else if symbols.take("regular") { Some(Regular) }
                else { None }
            },
            slant: {
                if symbols.take("italic") { Some(Italic) }
                else if symbols.take("designation") { Some(Italic) }
                else if symbols.take("upright") { Some(Upright) }
                else { None }
            },
            weight: {
                if symbols.take("bold") { Some(Bold) }
                else if symbols.take("light") { Some(Light) }
                else if symbols.take("book") { Some(Book) }
                else { None }
            },
        }
    }

    fn update(self, face: FontFaceBuilder) -> FontFaceBuilder {
        FontFaceBuilder {
            family: if let Some(family) = face.family {
                Some(family)
            }
            else {
                self.family
            },
            stretch: if let Some(stretch) = face.stretch {
                Some(stretch)
            }
            else {
                self.stretch
            },
            slant: if let Some(slant) = face.slant {
                Some(slant)
            }
            else {
                self.slant
            },
            weight: if let Some(weight) = face.weight {
                Some(weight)
            }
            else {
                self.weight
            }
        }
    }

    fn apply(self, face: FontFace) -> FontFace {
        FontFace::new(
            self.family.unwrap_or(face.family),
            self.stretch.unwrap_or(face.stretch),
            self.slant.unwrap_or(face.slant),
            self.weight.unwrap_or(face.weight),
        )
    }
}


//------------ Properties ----------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Properties {
    linenum: bool,
    face: FontFace,
    size: FontSize,
    class: Class,
}

impl Properties {
    pub fn with_class(class: Class) -> Self {
        Properties {
            class,
            .. Default::default()
        }
    }

    pub fn with_size(size: FontSize) -> Self {
        Properties {
            size,
            .. Default::default()
        }
    }

    pub fn from_arg(
        arg: eval::Expression,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.into_symbol_set(err)?.0;
        let res = PropertiesBuilder::from_symbols(&mut symbols);
        symbols.check_exhausted(err)?;
        Ok(Self::default().update(&res))
    }

    pub fn update(&self, update: &PropertiesBuilder) -> Self {
        Properties {
            linenum: false,
            face: update.font.apply(self.face),
            size: update.size.unwrap_or(self.size),
            class: self.class.update(&update.class),
        }
    }

    pub fn enable_linenum(&mut self) {
        self.linenum = true
    }

    pub fn apply_font(&self, canvas: &Canvas) {
        canvas.apply_font(self.face, self.size.size());
    }

    pub fn apply_color(&self, canvas: &Canvas) {
        canvas.style().label_color(&self.class).apply(canvas);
    }
}


//------------ FontSize ------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum FontSize {
    Xsmall,
    Small,
    Medium,
    Large,
    Xlarge,
    Badge,
}

impl FontSize {
    pub fn size(self) -> f64 {
        use self::FontSize::*;

        match self {
            Xsmall => 5.,
            Small => 6.,
            Medium => 7.,
            Large => 9.,
            Xlarge => 11.,
            Badge => 5.5,
        }
    }

    pub fn from_symbols(symbols: &mut eval::SymbolSet) -> Option<Self> {
        use self::FontSize::*;

        if symbols.take("xsmall") { Some(Xsmall) }
        else if symbols.take("small") { Some(Small) }
        else if symbols.take("medium") { Some(Medium) }
        else if symbols.take("large") { Some(Large) }
        else if symbols.take("xlarge") { Some(Xlarge) }
        else if symbols.take("badgesize") { Some(Badge) }
        else { None }
    }
}

impl Default for FontSize {
    fn default() -> Self {
        FontSize::Medium
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
        self.properties.apply_font(canvas);

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
                self.properties.apply_font(canvas);
                Color::WHITE.apply(canvas);
                canvas.set_line_width(self.properties.size.size());
                canvas.set_line_cap(cairo::LineCap::Butt);
                canvas.set_line_join(cairo::LineJoin::Bevel);
                canvas.move_to(point.x, point.y);
                canvas.text_path(&self.text);
                canvas.stroke();
                canvas.set_line_join(join);
                canvas.set_line_cap(cap);
            }
            0 => {
                self.properties.apply_font(canvas);
                self.properties.apply_color(canvas);
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
    class: Class,
}

impl HruleSpan {
    fn new(width: Distance, class: Class) -> Self {
        HruleSpan { width, class }
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
            canvas.style().label_color(&self.class).apply(canvas);
            canvas.stroke()
        }
    }
}

