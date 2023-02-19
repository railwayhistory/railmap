//! Making and rendering label features.

use kurbo::{Point, Rect};
use crate::render::canvas::{
    Canvas, FontFamily, FontFace, FontSlant, FontStretch, FontWeight,
};
use crate::render::color::Color;
use crate::render::label::{Align, Label, Layout};
use crate::render::path::Position;
use crate::import::eval;
use crate::import::Failed;
use super::super::class::Class;
use super::super::style::Style;
use super::super::theme::Overnight;
use super::Feature;


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
    Hrule,
    Vrule,
    Frame {
        content: Box<LayoutBuilder>,
    }
}

impl LayoutBuilder {
    fn new(content: ContentBuilder, properties: PropertiesBuilder) -> Self {
        LayoutBuilder { content, properties }
    }

    pub fn vbox(
        halign: Align, valign: Align, properties: PropertiesBuilder,
        lines: impl Into<StackBuilder>
    ) -> Self {
        LayoutBuilder::new(
            ContentBuilder::Vbox { halign, valign, lines: lines.into() },
            properties
        )
    }

    pub fn hbox(
        halign: Align, valign: Align, properties: PropertiesBuilder,
        spans: impl Into<StackBuilder>
    ) -> Self {
        LayoutBuilder::new(
            ContentBuilder::Hbox { halign, valign, spans: spans.into() },
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
        properties: PropertiesBuilder
    ) -> Self {
        LayoutBuilder::new(
            ContentBuilder::Hrule,
            properties
        )
    }

    pub fn vrule(
        properties: PropertiesBuilder
    ) -> Self {
        LayoutBuilder::new(
            ContentBuilder::Vrule,
            properties
        )
    }

    pub fn frame(
        properties: PropertiesBuilder,
        content: LayoutBuilder,
    ) -> Self {
        LayoutBuilder::new(
            ContentBuilder::Frame { content: content.into() },
            properties
        )
    }

    pub fn from_expr(
        expr: eval::Expression<Overnight>,
        err: &mut eval::Error
    ) -> Result<Self, Failed> {
        match expr.value {
            eval::ExprVal::Custom(val) => Ok(val),
            eval::ExprVal::Text(val) => {
                Ok(LayoutBuilder::span(val, Default::default()))
            }
            _ => {
                err.add(expr.pos, "expected layout or string");
                return Err(Failed)
            }
        }
    }

    pub fn rebase_properties(&mut self, base: &PropertiesBuilder) {
        self.properties.rebase(base)
    }

    pub fn into_feature(
        self, position: Position, on_path: bool, base: Properties
    ) -> Feature {
        Feature::Label(self.into_label(position, on_path, base))
    }

    pub fn into_label(
        self, position: Position, on_path: bool, base: Properties
    ) -> Label<Overnight> {
        Label::new(position, on_path, self.into_layout(&base))
    }

    pub fn into_layout(
        self, base: &Properties
    ) -> Layout<Overnight> {
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
                Layout::span(Span::Text(TextSpan::new(content, properties)))
            }
            ContentBuilder::Hrule => {
                Layout::span(
                    Span::Hrule(HruleSpan::new(properties.class))
                )
            }
            ContentBuilder::Vrule => {
                Layout::span(
                    Span::Vrule(VruleSpan::new(properties.class))
                )
            }
            ContentBuilder::Frame { content } => {
                let content = content.into_layout(&properties);
                Layout::span(
                    Span::Frame(FrameSpan::new(properties.class, content))
                )
            }
        }
    }
}

impl From<LayoutBuilder> for eval::ExprVal<Overnight> {
    fn from(src: LayoutBuilder) -> Self {
        eval::ExprVal::custom(src)
    }
}


//------------ StackBuilder --------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct StackBuilder {
    items: Vec<LayoutBuilder>,
}

impl StackBuilder {
    pub fn from_args(
        args: impl Iterator<Item = eval::Expression<Overnight>>,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let mut res = Self::default();
        for expr in args {
            res.items.push(LayoutBuilder::from_expr(expr, err)?);
        }
        Ok(res)
    }
}


impl From<Vec<LayoutBuilder>> for StackBuilder {
    fn from(items: Vec<LayoutBuilder>) -> Self {
        StackBuilder { items }
    }
}


impl IntoIterator for StackBuilder {
    type Item = <Vec<LayoutBuilder> as IntoIterator>::Item;
    type IntoIter = <Vec<LayoutBuilder> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}


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
        arg: eval::Expression<Overnight>,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.into_symbol_set(err)?;
        let res = Self::from_symbols(&mut symbols);
        symbols.check_exhausted(err)?;
        Ok(res)
    }

    pub fn from_symbols(symbols: &mut eval::SymbolSet) -> Self {
        let res = PropertiesBuilder {
            font: FontFaceBuilder::from_symbols(symbols),
            size: FontSize::from_symbols(symbols),
            class: Class::from_symbols(symbols),
        };
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

#[derive(Clone, Debug)]
pub struct Properties {
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
        arg: eval::Expression<Overnight>,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.into_symbol_set(err)?;
        let res = PropertiesBuilder::from_symbols(&mut symbols);
        symbols.check_exhausted(err)?;
        Ok(Self::default().update(&res))
    }

    pub fn update(&self, update: &PropertiesBuilder) -> Self {
        Properties {
            face: update.font.apply(self.face),
            size: update.size.unwrap_or(self.size),
            class: self.class.update(&update.class),
        }
    }

    pub fn apply_font(&self, style: &Style, canvas: &Canvas) {
        canvas.apply_font(self.face, self.size.size() * style.canvas_bp());
    }

    pub fn apply_color(&self, style: &Style, canvas: &Canvas) {
        style.label_color(&self.class).apply(canvas);
    }
}

impl Default for Properties {
    fn default() -> Self {
        Properties {
            face: FontFace::with_family(FontFamily::Inter),
            size: FontSize::default(),
            class: Class::default()
        }
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


//------------ Span ----------------------------------------------------------

/// The various types of spans we support.
pub enum Span {
    Text(TextSpan),
    Hrule(HruleSpan),
    Vrule(VruleSpan),
    Frame(FrameSpan),
}

impl crate::render::label::Span<Overnight> for Span {
    fn extent(&self, style: &Style, canvas: &Canvas) -> (Rect, usize) {
        match self {
            Span::Text(span) => span.extent(style, canvas),
            Span::Hrule(span) => span.extent(style, canvas),
            Span::Vrule(span) => span.extent(style, canvas),
            Span::Frame(span) => span.extent(style, canvas),
        }
    }

    fn render(
        &self, style: &Style, canvas: &Canvas, depth: usize, point: Point,
        extent: Rect, outer: Rect,
    ) {
        match self {
            Span::Text(span) => {
                span.render(style, canvas, depth, point, extent, outer)
            }
            Span::Hrule(span) => {
                span.render(style, canvas, depth, point, extent, outer)
            }
            Span::Vrule(span) => {
                span.render(style, canvas, depth, point, extent, outer)
            }
            Span::Frame(span) => {
                span.render(style, canvas, depth, point, extent, outer)
            }
        }
    }
}


//------------ TextSpan ------------------------------------------------------

/// The rendering rule for a span of text.
pub struct TextSpan {
    text: String,
    properties: Properties,
}

impl TextSpan {
    fn new(text: String, properties: Properties) -> Self {
        TextSpan { text, properties }
    }

    fn extent(&self, style: &Style, canvas: &Canvas) -> (Rect, usize) {
        self.properties.apply_font(style, canvas);

        // We take the width from the text extents and the height from the
        // font extents. This assumes that the text is one line exactly.
        let text = canvas.text_extents(&self.text).unwrap();
        let font = canvas.font_extents().unwrap();
 
        // The font height may be bigger than ascent plus descent so we correct
        // the descent for this.
        let top = -font.ascent();
        let bottom = top + font.height();

        // For the width, we use the text’s x_advance. This should consider the
        // intended width instead of the inked width.
        let left = 0.;
        let right = text.x_advance();

        (Rect::new(left, top, right, bottom), 2)
    }

    fn render(
        &self, style: &Style, canvas: &Canvas, depth: usize, point: Point,
        _extent: Rect, _outer: Rect,
    ) {
        match depth {
            1 =>  {
                let cap = canvas.line_cap();
                let join = canvas.line_join();
                self.properties.apply_font(style, canvas);
                Color::WHITE.apply(canvas);
                canvas.set_line_width(self.properties.size.size());
                canvas.set_line_cap(cairo::LineCap::Butt);
                canvas.set_line_join(cairo::LineJoin::Bevel);
                canvas.move_to(point.x, point.y);
                canvas.text_path(&self.text);
                canvas.stroke().unwrap();
                canvas.set_line_join(join);
                canvas.set_line_cap(cap);
            }
            0 => {
                self.properties.apply_font(style, canvas);
                self.properties.apply_color(style, canvas);
                canvas.move_to(point.x, point.y);
                canvas.show_text(&self.text).unwrap();
            }
            _ => { }
        }
    }
}


//------------ HruleSpan -----------------------------------------------------

/// The rendering rule for a horizontal bar
pub struct HruleSpan {
    class: Class,
}

impl HruleSpan {
    fn new(class: Class) -> Self {
        HruleSpan { class }
    }

    fn resolved_width(&self, style: &Style) -> f64 {
        style.dimensions().guide_width
    }

    fn extent(&self, style: &Style, _: &Canvas) -> (Rect, usize) {
        let height = self.resolved_width(style) / 2.;
        (Rect::new(0., -height, 0., height), 1)
    }

    fn render(
        &self, style: &Style, canvas: &Canvas, depth: usize, point: Point,
        _extent: Rect, outer: Rect,
    ) {
        if depth == 0 {
            canvas.set_line_width(self.resolved_width(style));
            canvas.move_to(point.x + outer.x0, point.y);
            canvas.line_to(point.x + outer.x1, point.y);
            style.label_color(&self.class).apply(canvas);
            canvas.stroke().unwrap();
        }
    }
}


//------------ VruleSpan -----------------------------------------------------

/// The rendering rule for a horizontal bar
pub struct VruleSpan {
    class: Class,
}

impl VruleSpan {
    fn new(class: Class) -> Self {
        VruleSpan { class }
    }

    fn resolved_width(&self, style: &Style) -> f64 {
        style.dimensions().guide_width
    }

    fn extent(&self, style: &Style, _: &Canvas) -> (Rect, usize) {
        let width = self.resolved_width(style) / 2.;
        (Rect::new(-width, 0., width, 0.), 1)
    }

    fn render(
        &self, style: &Style, canvas: &Canvas, depth: usize, point: Point,
        _extent: Rect, outer: Rect,
    ) {
        if depth == 0 {
            canvas.set_line_width(self.resolved_width(style));
            canvas.move_to(point.x, point.y + outer.y0);
            canvas.line_to(point.x, point.y + outer.y1);
            style.label_color(&self.class).apply(canvas);
            canvas.stroke().unwrap();
        }
    }
}


//------------ FrameSpan -----------------------------------------------------

/// The rendering rule for a frame around more label.
pub struct FrameSpan {
    content: Box<Layout<Overnight>>,
    class: Class,
}

impl FrameSpan {
    fn new(
        class: Class, content: Layout<Overnight>,
    ) -> Self {
        FrameSpan { content: content.into(), class }
    }

    fn resolved_width(&self, style: &Style) -> f64 {
        style.dimensions().guide_width
    }

    fn extent(&self, style: &Style, canvas: &Canvas) -> (Rect, usize) {
        let width = self.resolved_width(style);
        let (mut extent, depth) = self.content.extent(style, canvas);
        extent.x0 -= width;
        extent.y0 -= width;
        extent.x1 += width;
        extent.y1 += width;
        (extent, depth)
    }

    fn render(
        &self, style: &Style, canvas: &Canvas, depth: usize, point: Point,
        extent: Rect, outer: Rect,
    ) {
        if depth == 0 {
            let width = self.resolved_width(style);
            let half = width / 2.;
            canvas.set_line_width(width);
            canvas.move_to(
                point.x + extent.x0 + half, point.y + outer.y0 + half
            );
            canvas.line_to(
                point.x + extent.x1 - half, point.y + outer.y0 + half
            );
            canvas.line_to(
                point.x + extent.x1 - half, point.y + outer.y1 - half
            );
            canvas.line_to(
                point.x + extent.x0 + half, point.y + outer.y1 - half
            );
            canvas.close_path();
            Color::WHITE.apply(canvas);
            canvas.fill_preserve().unwrap();
            style.label_color(&self.class).apply(canvas);
            canvas.stroke().unwrap();
        }
        self.content.render(style, canvas, depth, point, extent, outer)
    }
}

