//! Making and rendering label features.

use femtomap::{layout, world};
use femtomap::import::eval::{EvalErrors, Failed, SymbolSet};
use femtomap::layout::{Align, Base, Margins, ShapedBlock, ShapedLayout};
use femtomap::path::Position;
use femtomap::render::{
    Canvas, Color, Font, FontBuilder, FontFamily, FontFeatures, FontStretch,
    FontStyle, FontWeight, LineCap, LineJoin, LineWidth, Matrix,
    Operator, TextDecoration, Sketch,
};
use crate::railway::import::eval;
use crate::railway::import::eval::{Custom, Expression, Scope};
use crate::railway::class::Railway;
use crate::railway::style::Style;
use super::{AnyShape, Category, Group, Feature, Shape, Stage, StageSet};


//------------ Configuration -------------------------------------------------

const SANS_FAMILY: FontFamily = FontFamily::from_static("FiraGO");
const ROMAN_FAMILY: FontFamily = FontFamily::from_static(
    "Source Serif 4 SmText"
);

const SANS_FEATURES: FontFeatures = FontFeatures::from_static("pnum");
const ROMAN_FEATURES: FontFeatures = FontFeatures::from_static("");

const LINE_HEIGHT: f64 = 0.9;


//------------ Label ---------------------------------------------------------

/// The feature for the label.
pub struct Label {
    /// The position the label is attached to.
    position: Position,

    /// Is the position’s base direction along the path?
    ///
    /// If this is `false`, the base direction is to the right.
    on_path: bool,

    /// The block to render
    block: Layout,
}

impl Label {
    pub fn new(
        mut block: Layout,
        position: Position,
        on_path: bool,
        mut base: BlockProperties,
    ) -> Self {
        base.update(&BlockProperties::base());
        block.update_properties(&base, |me, parent| me.update(parent));

        Self {
            position, on_path, block
        }
    }
}

impl Feature for Label {
    fn storage_bounds(&self) -> world::Rect {
        self.position.storage_bounds()
    }

    fn group(&self) -> Group {
        Group::with_category(Category::Label)
    }

    fn shape(
        &self, style: &Style, canvas: &Canvas
    ) -> AnyShape {
        let (point, angle) = self.position.resolve_label(style, self.on_path);
        let matrix = Matrix::identity().translate(point).rotate(angle);
        let layout = self.block.shape(Default::default(), style, canvas);
        AnyShape::from(LabelShape { matrix, layout })
    }
}


//------------ LabelShape -----------------------------------------------------

struct LabelShape<'a> {
    matrix: Matrix,
    layout: ShapedLayout<'a, BlockProperties>,
}

const LABEL_STAGES: StageSet = StageSet::from_slice(&[
    Stage::MarkerBase, Stage::MarkerCasing, Stage::MarkerBase,
]);

impl<'a> Shape<'a> for LabelShape<'a> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        self.layout.render(style, &stage, canvas.sketch().apply(self.matrix))
    }

    fn stages(&self) -> StageSet {
        LABEL_STAGES
    }
}
    


//------------ Layout ---------------------------------------------------------

pub type Layout = layout::Layout<BlockProperties>;

pub fn block_from_expr(
    expr: eval::Expression,
    err: &mut EvalErrors
) -> Result<Layout, Failed> {
    match expr.value {
        eval::Value::Custom(Custom::Layout(val)) => Ok(val),
        eval::Value::Text(val) => {
            Ok(Layout::span(val.into(), Default::default()))
        }
        _ => {
            err.add(expr.pos, "expected block layout or string");
            return Err(Failed)
        }
    }
}


//------------ Block --------------------------------------------------------

pub type Block = layout::Block<BlockProperties>;

pub fn layout_from_expr(
    expr: eval::Expression,
    err: &mut EvalErrors
) -> Result<Block, Failed> {
    match expr.value {
        eval::Value::Custom(Custom::Layout(val)) => Ok(val.into()),
        eval::Value::Custom(Custom::Block(val)) => Ok(val),
        eval::Value::Text(val) => {
            Ok(Block::span(val.into(), Default::default()))
        }
        _ => {
            err.add(expr.pos, "expected layout or string");
            return Err(Failed)
        }
    }
}


//------------ Creating Blocks ----------------------------------------------

/// Determines the horizontal alignment of a vbox.
pub fn halign_from_symbols(symbols: &mut SymbolSet) -> Option<Align> {
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

/// Determines the vertical base of an vbox.
pub fn vbase_from_symbols(symbols: &mut SymbolSet) -> Option<Base> {
    if symbols.take("top") {
        Some(Base::Start)
    }
    else if symbols.take("middle") {
        Some(Base::Center)
    }
    else if symbols.take("topbase") {
        Some(Base::FirstBase)
    }
    else if symbols.take("base") {
        Some(Base::LastBase)
    }
    else if symbols.take("bottom") {
        Some(Base::End)
    }
    else {
        None
    }
}

/// Determines the horizontal base of an hbox.
pub fn hbase_from_symbols(symbols: &mut SymbolSet) -> Option<Base> {
    if symbols.take("left") {
        Some(Base::Start)
    }
    else if symbols.take("center") {
        Some(Base::Center)
    }
    else if symbols.take("sep") {
        Some(Base::FirstAnchor)
    }
    else if symbols.take("lastsep") {
        Some(Base::LastAnchor)
    }
    else if symbols.take("right") {
        Some(Base::End)
    }
    else {
        None
    }
}

/// Determines the vertical alignment of a hbox.
pub fn valign_from_symbols(symbols: &mut SymbolSet) -> Option<Align> {
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

pub fn layouts_from_args<'a, I: IntoIterator<Item = Expression<'a>>>(
    args: I, err: &mut EvalErrors,
) -> Result<Vec<Block>, Failed> {
    let mut res = Vec::new();
    for expr in args {
        res.push(layout_from_expr(expr, err)?);
    }
    Ok(res)
}


//------------ BlockProperties ----------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct BlockProperties {
    /// The font to be used.
    font: FontBuilder,

    /// The relative size of the font.
    size: Option<FontSize>,

    /// Is this a packed layout?
    packed: Option<bool>,

    /// What kind of layout to we have?
    layout_type: BlockType,

    /// The class for the layout.
    class: Railway,
}

impl BlockProperties {
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

    pub fn with_class(class: Railway) -> Self {
        Self { class, .. Default::default() }
    }

    pub fn from_scope(scope: &Scope) -> Self {
        Self {
            class: Railway::from_scope(scope),
            .. Default::default()
        }
    }

    pub fn from_arg(
        arg: Expression,
        scope: &Scope,
        err: &mut EvalErrors,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.eval(err)?;
        let res = Self::from_symbols(&mut symbols, scope);
        symbols.check_exhausted(err)?;
        Ok(res)
    }

    pub fn from_arg_only(
        arg: Expression,
        err: &mut EvalErrors,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.eval(err)?;
        let res = Self::from_symbols_only(&mut symbols);
        symbols.check_exhausted(err)?;
        Ok(res)
    }

    pub fn from_symbols(
        symbols: &mut SymbolSet, scope: &Scope,
    ) -> Self {
        Self {
            font: Self::font_from_symbols(symbols),
            size: FontSize::from_symbols(symbols),
            packed: None,
            layout_type: BlockType::Normal,
            class: Railway::from_symbols(symbols, scope),
        }
    }

    pub fn from_symbols_only(
        symbols: &mut SymbolSet
    ) -> Self {
        Self {
            font: Self::font_from_symbols(symbols),
            size: FontSize::from_symbols(symbols),
            packed: None,
            layout_type: BlockType::Normal,
            class: Railway::from_symbols_only(symbols),
        }
    }

    fn font_from_symbols(symbols: &mut SymbolSet) -> FontBuilder {
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

        // Decoration
        if symbols.take("former") {
            res = res.decoration(TextDecoration::LineThrough)
        }

        res
    }

    pub fn class(&self) -> &Railway {
        &self.class
    }

    pub fn set_layout_type(&mut self, layout_type: BlockType) {
        self.layout_type = layout_type
    }

    pub fn set_packed(&mut self, packed: bool) {
        self.packed = Some(packed)
    }

    pub fn set_size(&mut self, size: FontSize) {
        self.size = Some(size)
    }

    pub fn update_size(&mut self, base: FontSize) {
        if self.size.is_none() {
            self.size = Some(base)
        }
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


impl layout::Properties for BlockProperties {
    type Style = Style;
    type Stage = Stage;
    type SpanText = Text;

    fn packed(&self, _style: &Self::Style) -> bool {
        matches!(self.packed, Some(true))
    }

    fn span_text<'a>(
        &self, text: &'a Self::SpanText, style: &Self::Style
    ) -> &'a str {
        if style.latin_text() {
            if let Some(text) = text.latin.as_ref() {
                return text
            }
        }
        &text.original
    }

    fn font(&self, style: &Self::Style) -> Font {
        self.font.clone().size(
            self.size().size(style)
        ).finalize()
    }

    fn frame(&self, style: &Self::Style) -> Option<Margins> {
        // XXX Make this font and size dependent.
        match self.layout_type {
            BlockType::Rule => {
                Some(Margins::equal(0.5 * style.measures().guide_width()))
            }
            BlockType::TextFrame => {
                Some(Margins::equal(style.measures().guide_width()))
            }
            _ => None,
        }
    }

    fn margins(&self, style: &Self::Style) -> Margins {
        match self.layout_type {
            BlockType::BadgeFrame => {
                Margins::vh(
                    style.measures().dt() * 0.1,
                    style.measures().dt() * 0.5,
                )
            }
            BlockType::Framed => {
                Margins::vh(
                    self.size().size(style) * 0.15,
                    self.size().size(style) * 0.2,
                )
            }
            BlockType::ReverseBadge => {
                Margins::vh(
                    self.size().size(style) * 0.1,
                    self.size().size(style) * 0.2,
                )
            }
            _ => Margins::default()
        }
    }

    fn render(
        &self,
        layout: &ShapedBlock<Self>,
        style: &Self::Style,
        stage: &Self::Stage,
        canvas: &mut Sketch,
    ) {
        match stage {
            Stage::MarkerCasing => {
                if !layout.is_span()
                    || matches!(self.layout_type, BlockType::ReverseBadge)
                {
                    return
                }
                canvas.apply(LineCap::Butt);
                canvas.apply(LineJoin::Bevel);
                canvas.apply(Color::WHITE);
                canvas.apply(LineWidth(self.size().size(style) * 0.3));
                layout.stroke_text(canvas);
            }
            Stage::MarkerBase => {
                match self.layout_type {
                    BlockType::BadgeFrame => {
                        let mut canvas = canvas.group();
                        canvas.apply(layout.outer());
                        canvas.apply(Operator::DestinationOut);
                        canvas.fill();
                    }
                    BlockType::TextFrame => {
                        canvas.apply(layout.outer());
                        canvas.apply(Color::WHITE);
                        canvas.fill();
                    }
                    BlockType::ReverseBadge => {
                        canvas.apply(layout.outer());
                        canvas.apply(style.label_color(&self.class));
                        canvas.fill();
                    }
                    _ => { }
                }
            }
            Stage::MarkerMarking => {
                if layout.is_span() {
                    if matches!(self.layout_type, BlockType::ReverseBadge) {
                        canvas.apply(Color::WHITE);
                    }
                    else {
                        canvas.apply(style.label_color(&self.class));
                    }
                    layout.fill_text(canvas);
                }
                if layout.has_frame() {
                    canvas.apply(style.label_color(&self.class));
                    layout.fill_frame(canvas);
                }
            }
            _ => { }
        }
    }
}


//------------ BlockType ----------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
pub enum BlockType {
    #[default]
    Normal,
    Rule,
    TextFrame,
    BadgeFrame,

    /// The layout lives inside a frame and needs to grow a bit of margin.
    Framed,

    /// A badge with the foreground and background colors reversed.
    ReverseBadge,
}


//------------ Text ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Text {
    /// The text in its original script.
    original: String,

    /// The text in latin script, if it isn’t originally latin.
    latin: Option<String>,
}

impl Text {
    pub fn with_latin(original: String, latin: String) -> Self {
        Self {
            original,
            latin: Some(latin)
        }
    }
}

impl From<String> for Text {
    fn from(original: String) -> Self {
        Self {
            original,
            latin: None,
        }
    }
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

        match self {
            Xsmall => style.measures().xsmall_font(),
            Small => style.measures().small_font(),
            Medium => style.measures().medium_font(),
            Large => style.measures().large_font(),
            Xlarge => style.measures().xlarge_font(),
            Badge => style.measures().badge_font(),
        }
    }

    pub fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
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
    pub fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
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

    pub fn from_legacy_symbols(symbols: &mut SymbolSet) -> Option<Self> {
        Self::from_symbols(symbols).or_else(|| {
            if symbols.take("left") {
                Some(Anchor::East)
            }
            else if symbols.take("right") {
                Some(Anchor::West)
            }
            else if symbols.take("top") {
                Some(Anchor::South)
            }
            else if symbols.take("bottom") {
                Some(Anchor::North)
            }
            else {
                None
            }
        })
    }

    /// Converts the anchor into horizontal and vertical align.
    pub fn into_hbox_aligns(self) -> (Base, Align) {
        match self {
            Anchor::North => (Base::Center, Align::Start),
            Anchor::NorthEast => (Base::End, Align::Start),
            Anchor::East => (Base::End, Align::Center),
            Anchor::SouthEast => (Base::End, Align::End),
            Anchor::South => (Base::Center, Align::End),
            Anchor::SouthWest => (Base::Start, Align::End),
            Anchor::West => (Base::Start, Align::Center),
            Anchor::NorthWest => (Base::Start, Align::Start),
        }
    }
}


//------------ TextAnchor ----------------------------------------------------

/// The compass direction where to anchor a label.
#[derive(Clone, Copy, Debug)]
pub struct TextAnchor {
    pub h: Base,
    pub v: Align,
}

impl TextAnchor {
    pub fn new(h: Base, v: Align) -> Self {
        Self { h, v }
    }

    pub fn from_pair((h, v): (Base, Align)) -> Self {
        Self::new(h, v)
    }

    pub fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
        if let Some(anchor) = Anchor::from_symbols(symbols) {
            Some(Self::from_pair(anchor.into_hbox_aligns()))
        }
        else if symbols.take("left") {
            Some(Self::new(Base::End, Align::Base))
        }
        else if symbols.take("right") {
            Some(Self::new(Base::Start, Align::Base))
        }
        else if symbols.take("top") {
            Some(Self::new(Base::Center, Align::End))
        }
        else if symbols.take("bottom") {
            Some(Self::new(Base::Center, Align::Start))
        }
        else {
            None
        }
    }
}

