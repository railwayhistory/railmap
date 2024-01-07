//! Making and rendering label features.

use femtomap::layout;
use femtomap::layout::{Align, Margins, ShapedLayout};
use femtomap::path::{MapDistance, Position};
use femtomap::render::{
    Canvas, Color, Font, FontBuilder, FontFamily, FontFeatures, FontStretch,
    FontStyle, FontWeight, Group, LineCap, LineJoin, LineWidth, Matrix,
    Operator, Sketch,
};
use kurbo::{Point, Rect, Vec2};
use crate::import::eval;
use crate::import::Failed;
use super::super::style::Style;
use super::super::class::Class;
use super::super::theme::Railwayhistory;
use super::{Shape, Stage};


//------------ Configuration -------------------------------------------------

const SANS_FAMILY: FontFamily = FontFamily::from_static("FiraGO");
const ROMAN_FAMILY: FontFamily = FontFamily::from_static(
    "Source Serif 4 SmText"
);

const SANS_FEATURES: FontFeatures = FontFeatures::from_static("pnum");
const ROMAN_FEATURES: FontFeatures = FontFeatures::from_static("");

const LINE_HEIGHT: f64 = 0.9;


//------------ Feature -------------------------------------------------------

/// The feature for the label.
pub struct Feature {
    /// The position the label is attached to.
    position: Position,

    /// Is the position’s base direction along the path?
    ///
    /// If this is `false`, the base direction is to the right.
    on_path: bool,

    /// Properties describing if and how the features should be rendered.
    properties: LabelProperties,

    /// The layout to render
    layout: layout::Block<LayoutProperties>,
}

impl Feature {
    pub fn new(
        layout: Layout,
        properties: LabelProperties,
        position: Position,
        on_path: bool,
        mut base: LayoutProperties,
    ) -> Self {
        base.update(&LayoutProperties::base());
        let mut layout = layout::Block::new(layout);
        layout.update_properties(&base, |me, parent| me.update(parent));

        Self {
            position, on_path, properties, layout
        }
    }

    pub fn storage_bounds(&self) -> Rect {
        self.position.storage_bounds()
    }

    pub fn shape(
        &self, style: &Style, canvas: &Canvas
    ) -> Option<Box<dyn Shape + '_>> {
        if self.properties.linenum && !style.include_line_labels() {
            return None
        }
        let (point, angle) = self.position.resolve_label(style, self.on_path);
        Some(Box::new(ShapedFeature {
            matrix: Matrix::identity().translate(point).rotate(angle),
            layout: self.layout.shape(style, canvas)
        }))
    }
}

impl From<Feature> for super::Feature {
    fn from(src: Feature) -> Self {
        super::Feature::Label(src)
    }
}


//------------ ShapedFeature -------------------------------------------------

struct ShapedFeature<'a> {
    matrix: Matrix,
    layout: ShapedLayout<'a, LayoutProperties>,
}

impl<'a> Shape for ShapedFeature<'a> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        self.layout.render(
            style, Default::default(), &stage,
            canvas.sketch().apply(self.matrix)
        )
    }
}


//------------ LabelProperties -----------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct LabelProperties {
    /// Is the label for a line number?
    linenum: bool,
}

impl LabelProperties {
    pub fn new_linenum() -> Self {
        Self { linenum: true }
    }

    pub fn from_arg(
        linenum: bool,
        arg: eval::Expression<Railwayhistory>,
        err: &mut eval::Error,
    ) -> Result<(Self, LayoutProperties), Failed> {
        let mut symbols = arg.into_symbol_set(err)?;
        let mut properties = LayoutProperties::from_symbols(&mut symbols);
        let label_properties = LabelProperties {
            linenum: symbols.take("linenum") || linenum,
        };
        symbols.check_exhausted(err)?;
        if label_properties.linenum {
            properties.size = Some(FontSize::Badge)
        }
        Ok((label_properties, properties))
    }

    pub fn default_pair(linenum: bool) -> (Self, LayoutProperties) {
        (
            LabelProperties { linenum },
            if linenum {
                LayoutProperties::with_size(FontSize::Badge)
            }
            else {
                LayoutProperties::default()
            }
        )
    }
}



//------------ Layout --------------------------------------------------------

pub type Layout = layout::Layout<LayoutProperties>;

impl From<Layout> for eval::ExprVal<Railwayhistory> {
    fn from(src: Layout) -> Self {
        eval::ExprVal::custom(src)
    }
}

pub fn layout_from_expr(
    expr: eval::Expression<Railwayhistory>,
    err: &mut eval::Error
) -> Result<Layout, Failed> {
    match expr.value {
        eval::ExprVal::Custom(val) => Ok(val),
        eval::ExprVal::Text(val) => {
            Ok(Layout::span(val, Default::default()))
        }
        _ => {
            err.add(expr.pos, "expected layout or string");
            return Err(Failed)
        }
    }
}


//------------ Creating Layouts ----------------------------------------------

pub fn halign_from_symbols(symbols: &mut eval::SymbolSet) -> Option<Align> {
    if symbols.take("left") {
        Some(Align::Start)
    }
    else if symbols.take("center") {
        Some(Align::Center)
    }
    else if symbols.take("sep") {
        Some(Align::Base)
    }
    else if symbols.take("right") {
        Some(Align::End)
    }
    else {
        None
    }
}

pub fn valign_from_symbols(symbols: &mut eval::SymbolSet) -> Option<Align> {
    if symbols.take("top") {
        Some(Align::Start)
    }
    else if symbols.take("middle") {
        Some(Align::Center)
    }
    else if symbols.take("base") {
        Some(Align::Base)
    }
    else if symbols.take("bottom") {
        Some(Align::End)
    }
    else {
        None
    }
}

pub fn layouts_from_args(
    args: impl Iterator<Item = eval::Expression<Railwayhistory>>,
    err: &mut eval::Error,
) -> Result<Vec<Layout>, Failed> {
    let mut res = Vec::new();
    for expr in args {
        res.push(layout_from_expr(expr, err)?);
    }
    Ok(res)
}


//------------ LayoutProperties ----------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct LayoutProperties {
    /// The font to be used.
    font: FontBuilder,

    /// The relative size of the font.
    size: Option<FontSize>,

    /// Is this a packed layout?
    packed: Option<bool>,

    /// What kind of layout to we have?
    layout_type: LayoutType,

    /// The class for the layout.
    class: Class,
}

impl LayoutProperties {
    fn base() -> Self {
        Self {
            font: FontBuilder::new()
                .family(SANS_FAMILY)
                .features(SANS_FEATURES)
                .line_height(LINE_HEIGHT),
            .. Default::default()
        }
    }

    pub fn with_size(size: FontSize) -> Self {
        Self { size: Some(size), .. Default::default() }
    }

    pub fn with_class(class: Class) -> Self {
        Self { class, .. Default::default() }
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
        Self {
            font: Self::font_from_symbols(symbols),
            size: FontSize::from_symbols(symbols),
            packed: None,
            layout_type: LayoutType::Normal,
            class: Class::from_symbols(symbols),
        }
    }

    fn font_from_symbols(symbols: &mut eval::SymbolSet) -> FontBuilder {
        let mut res = FontBuilder::default();

        // Family
        //
        if symbols.take("sans") {
            res = res.family(SANS_FAMILY).features(SANS_FEATURES);
        }
        else if symbols.take("roman") {
            res = res.family(ROMAN_FAMILY).features(ROMAN_FEATURES);
        }

        // Stretch
        //
        if symbols.take("condensed") {
            res = res.stretch(FontStretch::Condensed);
        }

        // Style
        //
        if symbols.take("italic")
            || symbols.take("designation")
            || symbols.take("former")
        {
            res = res.style(FontStyle::Italic);
        }
        else if symbols.take("upright") {
            res = res.style(FontStyle::Normal);
        }

        // Variant
        //

        // Weight
        if symbols.take("bold") {
            res = res.weight(FontWeight::Bold)
        }
        else if symbols.take("regular") {
            res = res.weight(FontWeight::Normal)
        }
        else if symbols.take("light") {
            res = res.weight(FontWeight::Light)
        }

        res
    }

    pub fn class(&self) -> &Class {
        &self.class
    }

    pub fn set_layout_type(&mut self, layout_type: LayoutType) {
        self.layout_type = layout_type
    }

    pub fn set_packed(&mut self, packed: bool) {
        self.packed = Some(packed)
    }

    pub fn set_size(&mut self, size: FontSize) {
        self.size = Some(size)
    }

    pub fn update(&mut self, base: &Self) {
        self.font.update(&base.font);
        if self.size.is_none() {
            self.size = base.size
        }
        if self.packed.is_none() {
            self.packed = base.packed
        }
        self.class.update(&base.class)
    }

    fn size(&self) -> FontSize {
        self.size.unwrap_or_default()
    }
}


impl layout::Properties for LayoutProperties {
    type Style = Style;
    type Stage = Stage;

    fn font(&self, style: &Self::Style) -> Font {
        self.font.clone().size(
            self.size().size(style)
        ).finalize()
    }

    fn packed(&self, _style: &Self::Style) -> bool {
        matches!(self.packed, Some(true))
    }

    fn frame(&self, style: &Self::Style) -> Option<Margins> {
        // XXX Make this font and size dependent.
        match self.layout_type {
            LayoutType::Rule => {
                Some(Margins::equal(0.5 * style.dimensions().guide_width))
            }
            LayoutType::TextFrame => {
                Some(Margins::equal(style.dimensions().guide_width))
            }
            _ => None,
        }
    }

    fn margins(&self, style: &Self::Style) -> Margins {
        match self.layout_type {
            LayoutType::BadgeFrame => {
                Margins::vh(
                    style.dimensions().dt * 0.1,
                    style.dimensions().dt * 0.5,
                )
            }
            LayoutType::Framed => {
                Margins::vh(
                    self.size().size(style) * 0.15,
                    self.size().size(style) * 0.2,
                )
            }
            _ => Margins::default()
        }
    }

    fn render(
        &self,
        layout: &ShapedLayout<Self>,
        style: &Self::Style,
        base: Vec2,
        stage: &Self::Stage,
        canvas: &mut Sketch,
    ) {
        match stage {
            Stage::Back => {
                match self.layout_type {
                    LayoutType::BadgeFrame => {
                        let mut canvas = canvas.group();
                        canvas.apply(layout.outer(base));
                        canvas.apply(Operator::DestinationOut);
                        canvas.fill();
                    }
                    LayoutType::TextFrame => {
                        canvas.apply(layout.outer(base));
                        canvas.apply(Color::WHITE);
                        canvas.fill();
                    }
                    _ => { }
                }
            }
            Stage::Casing => {
                if !layout.is_span() {
                    return
                }
                canvas.apply(LineCap::Butt);
                canvas.apply(LineJoin::Bevel);
                canvas.apply(Color::WHITE);
                canvas.apply(LineWidth(self.size().size(style) * 0.3));
                layout.stroke_text(base, canvas);
            }
            Stage::Base => {
                if layout.is_span() {
                    canvas.apply(style.label_color(&self.class));
                    layout.fill_text(base, canvas);
                }
                if layout.has_frame() {
                    canvas.apply(style.label_color(&self.class));
                    layout.fill_frame(base, canvas);
                }
            }
            /*
            Stage::Inside => {
                // Draw boxes around boxes for debugging.
                let mut outer = layout.outer(base);
                canvas.apply(outer);
                canvas.apply(Color::RED);
                canvas.apply_line_width(0.6);
                canvas.stroke();
                outer.y0 = 0.;
                outer.y1 = 0.;
                canvas.apply(outer);
                canvas.apply(Color::BLUE);
                canvas.stroke();
            }
            */
            _ => { }
        }
    }
}


//------------ LayoutType ----------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
pub enum LayoutType {
    #[default]
    Normal,
    Rule,
    TextFrame,
    BadgeFrame,

    /// The layout lives inside a frame and needs to grow a bit of margin.
    Framed,
}



//------------ FontSize ------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
pub enum FontSize {
    Xsmall,
    Small,
    #[default]
    Medium,
    Large,
    Xlarge,
    Badge,
}

impl FontSize {
    pub fn size(self, style: &Style) -> f64 {
        use self::FontSize::*;

        let base = match self {
            Xsmall => 5.,
            Small => 6.,
            Medium => 7.,
            Large => 9.,
            Xlarge => 11.,
            Badge => 5.4,
        };
        /*
        let base = if style.detail() >= 3.0 {
            match self {
                Xsmall => 5.,
                Small => 6.,
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
        };
        */
        base * style.canvas_bp()
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


//------------ Anchor --------------------------------------------------------

/// The compass direction where to anchor a label.
#[derive(Clone, Copy, Debug)]
pub enum Anchor {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}


impl Anchor {
    pub fn from_symbols(symbols: &mut eval::SymbolSet) -> Option<Self> {
        if symbols.take("n") {
            Some(Anchor::North)
        }
        else if symbols.take("ne") {
            Some(Anchor::NorthEast)
        }
        else if symbols.take("e") {
            Some(Anchor::East)
        }
        else if symbols.take("se") {
            Some(Anchor::SouthEast)
        }
        else if symbols.take("s") {
            Some(Anchor::South)
        }
        else if symbols.take("sw") {
            Some(Anchor::SouthWest)
        }
        else if symbols.take("w") {
            Some(Anchor::West)
        }
        else if symbols.take("nw") {
            Some(Anchor::NorthWest)
        }
        else {
            None
        }
    }

    /// Converts the anchor into horizontal and vertical align.
    pub fn into_aligns(self) -> (Align, Align) {
        use self::Align::*;

        match self {
            Anchor::North => (Center, Start),
            Anchor::NorthEast => (End, Start),
            Anchor::East => (End, Center),
            Anchor::SouthEast => (End, End),
            Anchor::South => (Center, End),
            Anchor::SouthWest => (Start, End),
            Anchor::West => (Start, Center),
            Anchor::NorthWest => (Start, Start),
        }
    }
}

