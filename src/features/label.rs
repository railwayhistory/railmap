use kurbo::{Point, Rect};
use crate::canvas::{Canvas, FontFace};
use crate::import::eval::SymbolSet;
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
    pub fn new(
        position: Position, on_path: bool, layout: Layout
    ) -> Self {
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

        let extent = self.layout.extent(canvas);
        self.layout.render_background(Point::default(), extent, canvas);
        self.layout.render(Point::default(),extent, extent, canvas);
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

impl Layout {
    fn new(content: Content) -> Self {
        Layout { content }
    }

    fn render_background(&self, point: Point, extent: Rect, canvas: &Canvas) {
        match self.content {
            Content::Vbox(ref v)
                => v.render_background(point, extent, canvas),
            Content::Hbox(ref v)
                => v.render_background(point, extent, canvas),
            Content::Span(ref v)
                => v.render_background(point, extent, canvas),
            Content::Hbar(_) => { }
        }
    }

    fn render(
        &self, point: Point, extent: Rect, outer: Rect, canvas: &Canvas
    ) {
        match self.content {
            Content::Vbox(ref v)
                => v.render(point, extent, canvas),
            Content::Hbox(ref v)
                => v.render(point, extent, canvas),
            Content::Span(ref v)
                => v.render(point, canvas),
            Content::Hbar(ref v)
                => v.render(point, outer, canvas),
        }
    }

    /// The extent of the layout.
    ///
    /// The values are given relative to the layout’s reference point.
    fn extent(&self, canvas: &Canvas) -> Rect {
        match self.content {
            Content::Vbox(ref v) => v.extent(canvas),
            Content::Hbox(ref v) => v.extent(canvas),
            Content::Span(ref v) => v.extent(canvas),
            Content::Hbar(ref v) => v.extent(canvas),
        }
    }
}


//------------ Content -------------------------------------------------------

#[derive(Clone, Debug)]
enum Content {
    Vbox(Vbox),
    Hbox(Hbox),
    Span(Span),
    Hbar(Hbar),
}


//------------ Vbox ----------------------------------------------------------

#[derive(Clone, Debug)]
struct Vbox {
    halign: Align,
    valign: Align,
    lines: Stack,
}

impl Vbox {
    fn render_background(&self, point: Point, extent: Rect, canvas: &Canvas) {
        self.render_op(point, extent, canvas, |layout, point, extent| {
            layout.render_background(point, extent, canvas)
        })
    }

    fn render(&self, point: Point, extent: Rect, canvas: &Canvas) {
        let outer = extent;
        self.render_op(point, extent, canvas, |layout, point, extent| {
            layout.render(point, extent, outer, canvas)
        })
    }

    fn render_op<F: Fn(&Layout, Point, Rect)>(
        &self, mut point: Point, extent: Rect, canvas: &Canvas, op: F
    ) {
        point.y += extent.y0;
        for layout in &self.lines {
            let extent = layout.extent(canvas);
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

    fn extent(&self, canvas: &Canvas) -> Rect {
        let mut res = Rect::default();
        let mut top = None;
        for layout in &self.lines {
            let extent = layout.extent(canvas);
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

/// A sequence of layouts stacked horizontally.
#[derive(Clone, Debug)]
struct Hbox {
    halign: Align,
    valign: Align,
    spans: Vec<Layout>,
}

impl Hbox {
    fn render_background(&self, point: Point, extent: Rect, canvas: &Canvas) {
        self.render_op(point, extent, canvas, |layout, point, extent| {
            layout.render_background(point, extent, canvas)
        })
    }

    fn render(&self, point: Point, extent: Rect, canvas: &Canvas) {
        let outer = extent;
        self.render_op(point, extent, canvas, |layout, point, extent| {
            layout.render(point, extent, outer, canvas)
        });
    }

    fn render_op<F: Fn(&Layout, Point, Rect)>(
        &self, mut point: Point, extent: Rect, canvas: &Canvas, op: F
    ) {
        point.x += extent.x0;
        for layout in &self.spans {
            let extent = layout.extent(canvas);
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

    fn extent(&self, canvas: &Canvas) -> Rect {
        let mut res = Rect::default();
        let mut left = None;
        for layout in &self.spans {
            let extent = layout.extent(canvas);
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
    content: String,
    properties: Properties,
}

impl Span {
    fn render_background(&self, point: Point, extent: Rect, canvas: &Canvas) {
        if matches!(self.properties.background, Background::Transparent) {
            return
        }
        let extent = extent + point.to_vec2();
        let clearance = self.properties.font.clearance(canvas);
        canvas.move_to(extent.x0 - clearance.x, extent.y0 - clearance.y);
        canvas.line_to(extent.x0 - clearance.x, extent.y1 + clearance.y);
        canvas.line_to(extent.x1 + clearance.x, extent.y1 + clearance.y);
        canvas.line_to(extent.x1 + clearance.x, extent.y0 - clearance.y);
        canvas.close_path();
        match self.properties.background {
            Background::Transparent => {
                // We should have returned already.
                unreachable!()
            }
            Background::Clear => {
                canvas.set_operator(cairo::Operator::Clear);
                canvas.fill();
                canvas.set_operator(cairo::Operator::Over);
            }
            Background::Fill(color) => {
                color.apply(canvas);
                canvas.fill();
            }
        }
    }

    fn render(&self, point: Point, canvas: &Canvas) {
        self.properties.font.apply(canvas);
        canvas.move_to(point.x, point.y);
        canvas.show_text(&self.content);
    }

    fn extent(&self, canvas: &Canvas) -> Rect {
        self.properties.font.apply(canvas);

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


//------------ Hbar ----------------------------------------------------------

#[derive(Clone, Debug)]
struct Hbar {
    width: f64,
    color: Color,
}

impl Hbar {
    fn render(&self, point: Point, outer: Rect, canvas: &Canvas) {
        canvas.set_line_width(self.width * canvas.canvas_bp());
        canvas.move_to(point.x + outer.x0, point.y);
        canvas.line_to(point.x + outer.x1, point.y);
        canvas.stroke()
    }

    fn extent(&self, canvas: &Canvas) -> Rect {
        let height = self.width * canvas.canvas_bp() * 0.5;
        Rect::new(0., -height, 0., height)
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
    pub fn h_from_symbols(symbols: &SymbolSet) -> Option<Align> {
        if symbols.contains("left") {
            Some(Align::Start)
        }
        else if symbols.contains("center") {
            Some(Align::Center)
        }
        else if symbols.contains("sep") {
            Some(Align::Ref)
        }
        else if symbols.contains("right") {
            Some(Align::End)
        }
        else {
            None
        }
    }

    pub fn v_from_symbols(symbols: &SymbolSet) -> Option<Align> {
        if symbols.contains("top") {
            Some(Align::Start)
        }
        else if symbols.contains("middle") {
            Some(Align::Center)
        }
        else if symbols.contains("base") {
            Some(Align::Ref)
        }
        else if symbols.contains("bottom") {
            Some(Align::End)
        }
        else {
            None
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

    fn clearance(&self, canvas: &Canvas) -> Point {
        Point::new(canvas.canvas_bp(), canvas.canvas_bp())
    }
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


//------------ Stack ---------------------------------------------------------

pub type Stack = Vec<Layout>;


//============ Building Layouts ==============================================


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
    Hbar {
        width: f64,
    }
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

    pub fn hbar(
        width: f64
    ) -> Self {
        LayoutBuilder::new(
            ContentBuilder::Hbar { width },
            Default::default()
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
                Layout::new(Content::Vbox(Vbox {
                    halign, valign,
                    lines: lines.into_iter().map(|line| {
                        line.into_layout(&properties)
                    }).collect(),
                }))
            }
            ContentBuilder::Hbox { halign, valign, spans } => {
                Layout::new(Content::Hbox(Hbox {
                    halign, valign,
                    spans: spans.into_iter().map(|line| {
                        line.into_layout(&properties)
                    }).collect(),
                }))
            }
            ContentBuilder::Span { content } => {
                Layout::new(Content::Span(Span {
                    content,
                    properties
                }))
            }
            ContentBuilder::Hbar { width } => {
                Layout::new(Content::Hbar(Hbar {
                    width,
                    color: properties.font.color
                }))
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

