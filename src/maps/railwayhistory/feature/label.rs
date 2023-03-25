//! Making and rendering label features.

use kurbo::{Point, Rect};
use crate::render::canvas::{
    Canvas, FontFamily, FontFace, FontSlant, FontStretch, FontWeight,
};
use crate::render::color::Color;
use crate::render::label::{Align, Label, Layout};
use crate::render::path::{Distance, Position};
use crate::import::eval;
use crate::import::Failed;
use crate::theme::Style as _;
use super::super::class::{Class, Status};
use super::super::style::Style;
use super::super::theme::Railwayhistory;


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
    BadgeFrame {
        content: Box<LayoutBuilder>,
    },
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
        width: Distance, properties: PropertiesBuilder
    ) -> Self {
        LayoutBuilder::new(
            ContentBuilder::Hrule { width },
            properties
        )
    }

    pub fn badge_frame(
        properties: PropertiesBuilder,
        content: LayoutBuilder,
    ) -> Self {
        LayoutBuilder::new(
            ContentBuilder::BadgeFrame { content: content.into() },
            properties
        )
    }

    pub fn from_expr(
        expr: eval::Expression<Railwayhistory>,
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
        self,
        label_properties: LabelProperties,
        position: Position,
        on_path: bool,
        base: Properties,
    ) -> super::Feature {
        super::Feature::Label(
            self.into_label(label_properties, position, on_path, base)
        )
    }

    pub fn into_label(
        self,
        label_properties: LabelProperties,
        position: Position,
        on_path: bool,
        base: Properties,
    ) -> Feature {
        Feature::new(
            label_properties, position, on_path, self.into_layout(&base)
        )
    }

    pub fn into_layout(
        self, base: &Properties
    ) -> Layout<Railwayhistory> {
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
            ContentBuilder::Hrule { width } => {
                Layout::span(
                    Span::Hrule(HruleSpan::new(width, properties.class))
                )
            }
            ContentBuilder::BadgeFrame { content } => {
                Layout::span(
                    Span::BadgeFrame(BadgeFrame::new(
                        content.into_layout(&properties)
                    ))
                )
            }
        }
    }
}

impl From<LayoutBuilder> for eval::ExprVal<Railwayhistory> {
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
        args: impl Iterator<Item = eval::Expression<Railwayhistory>>,
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

    packed: Option<bool>,
}

impl PropertiesBuilder {
    pub fn from_arg(
        arg: eval::Expression<Railwayhistory>,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.into_symbol_set(err)?;
        let res = Self::from_symbols(&mut symbols);
        symbols.check_exhausted(err)?;
        Ok(res)
    }

    pub fn from_symbols(symbols: &mut eval::SymbolSet) -> Self {
        let mut res = PropertiesBuilder {
            font: FontFaceBuilder::from_symbols(symbols),
            size: FontSize::from_symbols(symbols),
            class: Class::from_symbols(symbols),
            packed: None,
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

    pub fn packed() -> Self {
        PropertiesBuilder {
            packed: Some(true),
            .. Default::default()
        }
    }

    pub fn class(&self) -> &Class {
        &self.class
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


//------------ LabelProperties -----------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct LabelProperties {
    linenum: bool,
}

impl LabelProperties {
    pub fn from_arg(
        linenum: bool,
        arg: eval::Expression<Railwayhistory>,
        err: &mut eval::Error,
    ) -> Result<(Self, Properties), Failed> {
        let mut symbols = arg.into_symbol_set(err)?;
        let mut properties = PropertiesBuilder::from_symbols(&mut symbols);
        let label_properties = LabelProperties {
            linenum: symbols.take("linenum") || linenum,
        };
        symbols.check_exhausted(err)?;
        if label_properties.linenum {
            properties.size = Some(FontSize::Badge)
        }
        Ok((label_properties, Properties::default().update(&properties)))
    }

    pub fn default_pair(linenum: bool) -> (Self, Properties) {
        (
            LabelProperties { linenum },
            if linenum {
                Properties::with_size(FontSize::Badge)
            }
            else {
                Properties::default()
            }
        )
    }
}


//------------ Properties ----------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Properties {
    face: FontFace,
    size: FontSize,
    class: Class,
    packed: bool,
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
        arg: eval::Expression<Railwayhistory>,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.into_symbol_set(err)?;
        let res = Self::from_symbols(&mut symbols);
        symbols.check_exhausted(err)?;
        Ok(res)
    }

    pub fn from_symbols(symbols: &mut eval::SymbolSet) -> Self {
        Self::default().update(&PropertiesBuilder::from_symbols(symbols))
    }

    pub fn update(&self, update: &PropertiesBuilder) -> Self {
        Properties {
            face: update.font.apply(self.face),
            size: update.size.unwrap_or(self.size),
            class: self.class.update(&update.class),
            packed: update.packed.unwrap_or(self.packed),
        }
    }

    pub fn class(&self) -> &Class {
        &self.class
    }

    pub fn apply_font(&self, style: &Style, canvas: &Canvas) {
        canvas.apply_font(
            self.face, self.size.size(style) * style.canvas_bp()
        );
    }

    pub fn apply_color(&self, style: &Style, canvas: &Canvas) {
        style.label_color(&self.class).apply(canvas);
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
    pub fn size(self, style: &Style) -> f64 {
        use self::FontSize::*;

        if style.detail() >= 3 {
            match self {
                Xsmall => 5.,
                Small => 6.5,
                Medium => 7.,
                Large => 9.,
                Xlarge => 11.,
                Badge => 5.4,
            }
        }
        else {
            match self {
                Xsmall => 5.,
                Small => 6.,
                Medium => 7.,
                Large => 9.,
                Xlarge => 11.,
                Badge => 5.4,
            }
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


//------------ Feature -------------------------------------------------------

/// The feature for the label.
pub struct Feature {
    properties: LabelProperties,
    label: Label<Railwayhistory>,
}

impl Feature {
    pub fn new(
        properties: LabelProperties,
        position: Position, on_path: bool, layout: Layout<Railwayhistory>,
    ) -> Self {
        Feature {
            properties,
            label: Label::new(position, on_path, layout)
        }
    }

    pub fn storage_bounds(&self) -> Rect {
        self.label.storage_bounds()
    }

    pub fn render(&self, style: &Style, canvas: &Canvas) {
        if !self.properties.linenum || style.include_line_labels() {
            self.label.render(style, canvas)
        }
    }
}



//------------ Span ----------------------------------------------------------

/// The various types of spans we support.
pub enum Span {
    Text(TextSpan),
    Hrule(HruleSpan),
    BadgeFrame(BadgeFrame),
}

impl crate::render::label::Span<Railwayhistory> for Span {
    fn extent(&self, style: &Style, canvas: &Canvas) -> (Rect, usize) {
        match self {
            Span::Text(span) => span.extent(style, canvas),
            Span::Hrule(span) => span.extent(style, canvas),
            Span::BadgeFrame(span) => span.extent(style, canvas),
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
            Span::BadgeFrame(span) => {
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
 
        // If we are packed, we only consider the inked area, otherwise
        // use the font’s extents.
        let (top, bottom) = if self.properties.packed {
            (text.y_bearing(), text.y_bearing() + text.height())
        }
        else {
            // The font height may be bigger than ascent plus descent so
            // we correct descent for this.
            (-font.ascent(), -font.ascent() + font.height())
        };

        // For the width, we use the text’s x_advance. This should consider the
        // intended width instead of the inked width.
        let left = 0.;
        let right = text.x_advance();

        (
            Rect::new(left, top, right, bottom),
            if self.properties.packed { 1 }  else { 2 }
        )
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
                canvas.set_line_width(self.properties.size.size(style));
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
    width: Distance,
    class: Class,
}

impl HruleSpan {
    fn new(width: Distance, class: Class) -> Self {
        HruleSpan { width, class }
    }

    fn resolved_width(&self, style: &Style) -> f64 {
        self.width.resolve(Default::default(), style)
    }

    fn extent(&self, style: &Style, _canvas: &Canvas) -> (Rect, usize) {
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


//------------ BadgeFrame ---------------------------------------------------

/// The rendering rule for a frame around more label.
pub struct BadgeFrame {
    content: Box<Layout<Railwayhistory>>,
}

impl BadgeFrame {
    fn new(
        content: Layout<Railwayhistory>,
    ) -> Self {
        BadgeFrame { content: content.into() }
    }

    fn extent(&self, style: &Style, canvas: &Canvas) -> (Rect, usize) {
        let (mut extent, depth) = self.content.extent(style, canvas);
        let xmargin = style.dimensions().dt * 0.2;
        let ymargin = style.dimensions().dt * 0.1;
        extent.x0 -= xmargin;
        extent.x1 += xmargin;
        extent.y0 -= ymargin;
        extent.y1 += ymargin;
        (extent, depth)
    }

    fn render(
        &self, style: &Style, canvas: &Canvas, depth: usize, mut point: Point,
        extent: Rect, outer: Rect,
    ) {
        let xmargin = style.dimensions().dt * 0.2;
        let ymargin = style.dimensions().dt * 0.1;
        if depth == 0 {
            canvas.move_to(
                point.x + extent.x0 - xmargin, point.y + outer.y0 - ymargin,
            );
            canvas.line_to(
                point.x + extent.x1 + xmargin, point.y + outer.y0 - ymargin,
            );
            canvas.line_to(
                point.x + extent.x1 + xmargin, point.y + outer.y1 + ymargin,
            );
            canvas.line_to(
                point.x + extent.x0 - xmargin, point.y + outer.y1 + ymargin
            );
            canvas.close_path();
            canvas.set_operator(cairo::Operator::Clear);
            canvas.fill().unwrap();
            canvas.set_operator(cairo::Operator::Over);
        }
        point.x += xmargin;
        point.y += ymargin;
        self.content.render(style, canvas, depth, point, extent, outer)
    }
}

