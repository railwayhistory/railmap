//! Rendering of track.
//!
//! Track is a contour feature with complex rendering rules depending on the
//! class, style, and detail level.
//!
//! # Track classes
//!
//! All line classes.
//!
//! Placement within multiple parallel tracks:
//!
//! * `:leftsame` if the track has another track of the same line on its left.
//! * `:leftother` if the track has another track of a different line on its
//!   left.
//! * `:rightsame` if the track has another track of the same line of its
//!   right.
//! * `:rightother` if the track has another track of a different line on its
//!   right.
//!
//! * In detail levels 3 and up, `:double` indicates two tracks of the same
//!   line offset half a dt to the left and right. The `:leftsame` and
//!   `:leftother` symbols apply to the left track only, while
//!   `:rightsame` and `:rightother` apply to the right track.
//!
//! * In detail levels 0 to 2, `:double` marks a track as double track.
//!
//! * `:tight` is a deprecated shortcut for `:leftother:rightother`.
//!
//! Placement within a sequence of segments that whose markings should look
//! consecutive:
//!
//! * `:start` if the segment is at the start of the sequence, i.e., extra
//!   spacing can be added at its beginning.
//! * `:end` if the segment is at the end of the sequence, i.e., extra
//!   spacing can be added at its end.
//! * `:inner` if the segment is in the middle and markings need to be
//!   “justified.”

#![allow(unused_imports)]
use std::f64::consts::{FRAC_PI_2, PI};
use femtomap::world;
use femtomap::import::eval::{EvalErrors, Failed, SymbolSet};
use femtomap::path::Trace;
use femtomap::render::{
    Canvas, Color, DashPattern, Group, LineCap, LineWidth, Outline, Sketch,
};
use kurbo::{PathEl, Vec2};
use crate::railway::import::eval::{Expression, Scope};
use crate::railway::class::{GaugeGroup, Railway, Pax};
use crate::railway::style::Style;
use super::{AnyShape, Category, Feature, Shape, Stage, StageSet};


//------------ Constants -----------------------------------------------------

/// How many no-pax-dash strokes go into a seg?
///
/// Works best with an odd number.
const NO_PAX_DASH_RATIO: f64 = 9.;

/// Half of NO_PAX_DASH_RATIO rounded down.
const NO_PAX_DASH_HALF: f64 = 4.;

/// Which portion of the no-pax-dash stroke should be on?
const NO_PAX_DASH_ON: f64 = 0.7;


//------------ TrackClass ----------------------------------------------------

/// The properties of the track.
#[derive(Clone, Debug)]
pub struct TrackClass {
    /// The feature class.
    class: Railway,

    /// The setup of the track and neighboring tracks.
    setup: Setup,
}

impl TrackClass {
    pub fn from_arg(
        arg: Expression,
        scope: &Scope,
        err: &mut EvalErrors,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.eval(err)?;
        let class = Self::from_symbols(&mut symbols, scope);
        symbols.check_exhausted(err)?;
        Ok(class)
    }

    pub fn from_symbols(symbols: &mut SymbolSet, scope: &Scope) -> Self {
        let _ = symbols.take("tight"); // XXX Deprecated.
        let _ = symbols.take("flip"); // XXX Deprecated.
        let _ = symbols.take("combined"); // XXX Deprecated.
        let _ = symbols.take("leftsame"); // XXX Deprecated.
        let _ = symbols.take("leftother"); // XXX Deprecated.
        let _ = symbols.take("rightsame"); // XXX Deprecated.
        let _ = symbols.take("rightother"); // XXX Deprecated.
        TrackClass {
            class: Railway::from_symbols(symbols, scope),
            setup: Setup::from_symbols(symbols),
        }
    }

    pub fn class(&self) -> &Railway {
        &self.class
    }

    pub fn double(&self) -> bool {
        self.class.double()
    }
}


//------------ Direction -----------------------------------------------------

/// The direction of a track of a multi-track line.
///
/// Despite being named “direction,” this is independent of whether trains
/// usually use the left or right track. Instead, we define it through the
/// left and right tracks of a double track line. If the track would be used
/// by traffic usually using the left track on double track lines, it is a
/// down track, otherwise an up track.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    /// An up track of a multi-track line.
    ///
    /// On a double track line, up is the right track in direction of
    /// increasing chainage.
    Up,

    /// A down track of a multi-track line.
    ///
    /// On a double track line, down is the left track in direction of
    /// increasing chainage.
    Down,

    /// A track both up and down.
    ///
    /// This should be used for the middle tracks of trippled lines.
    Updown,
}

impl Direction {
    fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
        if symbols.take("up") {
            Some(Self::Up)
        }
        else if symbols.take("dn") {
            Some(Self::Down)
        }
        else if symbols.take("ud") {
            Some(Self::Updown)
        }
        else {
            None
        }
    }
}


//------------ Neighbor ------------------------------------------------------

/// Describes the purpose of a neighboring track.
///
/// This is used for drawing track and line decorations and describes the
/// purpose of a track running parallel to this track.
#[derive(Clone, Copy, Debug, Default)]
pub enum Neighbor {
    /// There is no neighbor at all.
    #[default]
    None,

    /// The neighboring track is part of the same line for the same direction.
    Same(Direction),

    /// The neighboring track is part of a different line.
    Other,
}

impl Neighbor {
    fn left_from_symbols(symbols: &mut SymbolSet) -> Self {
        if symbols.take("lup") { // left up
            Self::Same(Direction::Up)
        }
        else if symbols.take("ldn") { // left down
            Self::Same(Direction::Down)
        }
        else if symbols.take("lud") { // left updown
            Self::Same(Direction::Updown)
        }
        else if symbols.take("lor") { // left other
            Self::Other
        }
        else if symbols.take("lno") { // left none
            Self::None
        }
        else {
            Self::None
        }
    }

    fn right_from_symbols(symbols: &mut SymbolSet) -> Self {
        if symbols.take("rup") { // right up
            Self::Same(Direction::Up)
        }
        else if symbols.take("rdn") { // right down
            Self::Same(Direction::Down)
        }
        else if symbols.take("rud") { // right updown
            Self::Same(Direction::Updown)
        }
        else if symbols.take("ror") { // right other
            Self::Other
        }
        else if symbols.take("rno") { // right none
            Self::None
        }
        else {
            Self::None
        }
    }
}


//------------ Setup ---------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct Setup {
    /// The direction of the track for a multitrack line.
    ///
    /// If this is `None`, the line is considered single track.
    direction: Option<Direction>,

    /// What is to our left?
    left: Neighbor,

    /// What is to our left?
    right: Neighbor,
}

impl Setup {
    fn from_symbols(symbols: &mut SymbolSet) -> Self {
        Self {
            direction: Direction::from_symbols(symbols),
            left: Neighbor::left_from_symbols(symbols),
            right: Neighbor::right_from_symbols(symbols),
        }
    }

    fn double_left(self) -> Self {
        Self {
            direction: Some(Direction::Down),
            left: self.left,
            right: Neighbor::Same(Direction::Up),
        }
    }

    fn double_right(self) -> Self {
        Self {
            direction: Some(Direction::Up),
            left: Neighbor::Same(Direction::Down),
            right: self.right,
        }
    }
}



//------------ TrackContour --------------------------------------------------

/// The contour of the actual track.
pub struct TrackContour {
    class: TrackClass,
    casing: bool,
    trace: Trace,
}

impl TrackContour {
    pub fn new(class: TrackClass, casing: bool, trace: Trace) -> Self {
        TrackContour { class, casing, trace }
    }
}

impl Feature for TrackContour {
    fn storage_bounds(&self) -> world::Rect {
        self.trace.storage_bounds()
    }

    fn group(&self) -> super::Group {
        super::Group::with_railway(Category::Track, &self.class.class)
    }

    fn shape(
        &self, style: &Style, _canvas: &Canvas
    ) -> AnyShape<'_> {
        /*
        if style.detail() <= 1 {
            AnyShape::from(
                ContourShape::new(&self.class, self.trace.outline(style))
            )
        }
        else
        */
        if style.detail() <= 2 {
            return AnyShape::from(ContourShape2::new(self, style));
        }
        else if style.detail() == 3 {
            return AnyShape::from(ContourShape3::new(self, style));
        }
        else {
            return ContourShape4::new(self, style)
        }
    }
}


/*
//------------ ContourShape --------------------------------------------------

struct ContourShape<'a> {
    class: &'a TrackClass,
    trace: Outline,
}

impl<'a> ContourShape<'a> {
    fn new(
        class: &'a TrackClass,
        trace: Outline,
    ) -> Self {
        Self { class, trace }
    }
}


impl<'a> Shape<'a> for ContourShape<'a> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        if !matches!(stage, Stage::Base) {
            return
        }
        let mut canvas = canvas.sketch().into_group();
        self.apply_line_width(style, &mut canvas);
        self.apply_line_color(style, &mut canvas);
        canvas.apply(&self.trace);
        canvas.stroke()
     }

    fn stages(&self) -> StageSet {
        StageSet::from(Stage::Base)
    }
}

impl<'a> ContourShape<'a> {
    fn apply_line_width(&self, style: &Style, canvas: &mut Group) {
        let line_width = if style.detail() < 1 {
            style.measures().main_track()
        }
        else if self.class.double() {
            style.measures().class_double(&self.class.class) * 1.4
        }
        else {
            style.measures().class_track(&self.class.class)
        };
        canvas.apply_line_width(line_width);
    }

    fn apply_line_color(&self, style: &Style, canvas: &mut Group) {
        canvas.apply(
            if self.class.class.is_open_no_pax() {
                style.track_color(&self.class.class).lighten(0.7)
            }
            else {
                style.track_color(&self.class.class)
            }
        );
    }
}
*/


//------------ ContourShape2 -------------------------------------------------

struct ContourShape2 {
    open: bool,
    color: Color,
    width: f64,
    casing_width: Option<f64>,
    dash: Option<DashPattern<2>>,
    outline: Outline,
}

impl ContourShape2 {
    fn new(
        contour: &TrackContour,
        style: &Style,
    ) -> Self {
        let outline = contour.trace.outline(style);
        let dash = Self::project_dash(
            &contour.class, &outline, style
        ).or_else(|| {
            Self::pax_dash(&contour.class, &outline, style)
        });
        let width = if contour.class.double() {
            style.measures().class_double(&contour.class.class)
        }
        else {
            style.measures().class_track(&contour.class.class)
        };

        Self {
            open: contour.class.class.status().is_open(),
            color: style.track_color(&contour.class.class),
            width,
            casing_width: contour.casing.then(|| {
                width + 2. * style.measures().class_skip(&contour.class.class)
            }),
            dash,
            outline
        }
    }

    fn project_dash(
        class: &TrackClass, outline: &Outline, style: &Style
    ) -> Option<DashPattern<2>> {
        if !class.class.status().is_project() {
            return None
        }

        calc_seg(outline, style.measures().seg()).map(|seg| {
            DashPattern::new([0.7 * seg, 0.3 * seg], 0.35 * seg)
        })
    }

    fn pax_dash(
        class: &TrackClass, outline: &Outline, style: &Style
    ) -> Option<DashPattern<2>> {
        // For historical reasons, an missing explicit pax defaults to no pax
        // for open lines and full pax for closed ones.
        if class.class.status().is_open() {
            if class.class.pax().is_full() {
                return None
            }
        }
        else {
            if class.class.opt_pax().unwrap_or(Pax::Full).is_full() {
                return None
            }
        }

        if matches!(class.class.pax(), Pax::None) {
            calc_seg(outline, style.measures().seg() * 0.125).map(|dist| {
                DashPattern::new([dist * 0.7, dist * 0.3], dist * 0.35)
            })
        }
        else {
            calc_seg(outline, style.measures().seg() * 0.25).map(|dist| {
                DashPattern::new([dist * 0.8, dist * 0.2], dist * 0.4)
            })
        }
    }
}

impl<'a> Shape<'a> for ContourShape2 {
    fn render(&self, stage: Stage, _style: &Style, canvas: &mut Canvas) {
        match stage {
            Stage::Casing => {
                if let Some(width) = self.casing_width {
                    canvas.sketch().apply(
                        Color::rgba(1., 1., 1., 0.8)
                    ).apply(
                        LineWidth(width)
                    ).apply(
                        &self.outline
                    ).stroke();
                }
            }
            Stage::AbandonedBase => {
                if !self.open {
                    canvas.sketch()
                        .apply(
                            if self.dash.is_some() {
                                Color::rgba(1., 1., 1., 0.8)
                            }
                            else {
                                self.color
                            }
                        )
                        .apply(LineWidth(self.width))
                        .apply(&self.outline)
                        .stroke()
                }
            }
            Stage::AbandonedMarking => {
                if !self.open {
                    if let Some(dash) = self.dash {
                       canvas.sketch() 
                            .apply(self.color)
                            .apply(LineWidth(self.width))
                            .apply(dash)
                            .apply(&self.outline)
                            .stroke()
                    }
                }
            }
            Stage::LimitedBase => {
                if self.open && self.dash.is_some() {
                   canvas.sketch() 
                        .apply(Color::rgba(1., 1., 1., 0.8))
                        .apply(LineWidth(self.width))
                        .apply(&self.outline)
                        .stroke()
                }
            }
            Stage::LimitedMarking => {
                if self.open {
                    if let Some(dash) = self.dash {
                       canvas.sketch() 
                            .apply(self.color)
                            .apply(LineWidth(self.width))
                            .apply(dash)
                            .apply(&self.outline)
                            .stroke()
                    }
                }
            }
            Stage::Base => {
                if self.open && self.dash.is_none() {
                   canvas.sketch() 
                        .apply(self.color)
                        .apply(LineWidth(self.width))
                        .apply(&self.outline)
                        .stroke()
                }
            }
            _ => { }
        }
    }

    fn stages(&self) -> StageSet {
        let res = if self.casing_width.is_some() {
            StageSet::from(Stage::Casing)
        }
        else {
            StageSet::empty()
        };
        if !self.open {
            if self.dash.is_some() {
                res.add(Stage::AbandonedBase).add(Stage::AbandonedMarking)
            }
            else {
                res.add(Stage::AbandonedBase)
            }
        }
        else if self.dash.is_some() {
            res.add(Stage::LimitedBase).add(Stage::LimitedMarking)
        }
        else {
            res.add(Stage::Base)
        }
    }
}


//------------ ContourShape3 -------------------------------------------------

struct ContourShape3 {
    open: bool,
    color: Color,
    width: f64,
    casing_width: Option<f64>,
    dash: Option<DashPattern<2>>,
    electric: Option<ElectricDecor>,
    outline: Outline,
}

impl ContourShape3 {
    fn new(
        contour: &TrackContour,
        style: &Style,
    ) -> Self {
        let outline = contour.trace.outline(style);
        let dash = Self::project_dash(
            &contour.class, &outline, style
        ).or_else(|| {
            Self::pax_dash(&contour.class, &outline, style)
        });
        let width = if contour.class.double() {
            style.measures().class_double(&contour.class.class)
        }
        else {
            style.measures().class_track(&contour.class.class)
        };
        let electric = ElectricDecor::new(
            &contour.class, contour.class.setup, width, &outline, style
        );

        Self {
            open: contour.class.class.status().is_open(),
            color: style.track_color(&contour.class.class),
            width,
            casing_width: contour.casing.then(|| {
                width + 2. * style.measures().class_skip(&contour.class.class)
            }),
            dash,
            electric,
            outline
        }
    }

    fn project_dash(
        class: &TrackClass, outline: &Outline, style: &Style
    ) -> Option<DashPattern<2>> {
        if !class.class.status().is_project() {
            return None
        }

        calc_seg(outline, style.measures().seg()).map(|seg| {
            DashPattern::new([0.7 * seg, 0.3 * seg], 0.35 * seg)
        })
    }

    fn pax_dash(
        class: &TrackClass, outline: &Outline, style: &Style
    ) -> Option<DashPattern<2>> {
        // For historical reasons, an missing explicit pax defaults to no pax
        // for open lines and full pax for closed ones.
        if class.class.status().is_open() {
            if class.class.pax().is_full() {
                return None
            }
        }
        else {
            if class.class.opt_pax().unwrap_or(Pax::Full).is_full() {
                return None
            }
        }

        if matches!(class.class.pax(), Pax::None) {
            calc_seg(outline, style.measures().seg() * 0.125).map(|dist| {
                DashPattern::new([dist * 0.7, dist * 0.3], dist * 0.35)
            })
        }
        else {
            calc_seg(outline, style.measures().seg() * 0.25).map(|dist| {
                DashPattern::new([dist * 0.8, dist * 0.2], dist * 0.4)
            })
        }
    }
}

impl<'a> Shape<'a> for ContourShape3 {
    fn render(&self, stage: Stage, _style: &Style, canvas: &mut Canvas) {
        match stage {
            Stage::Casing => {
                if let Some(width) = self.casing_width {
                    canvas.sketch().apply(
                        Color::rgba(1., 1., 1., 0.8)
                    ).apply(
                        LineWidth(width)
                    ).apply(
                        &self.outline
                    ).stroke();
                }
            }
            Stage::AbandonedBase => {
                if !self.open {
                    canvas.sketch()
                        .apply(
                            if self.dash.is_some() {
                                Color::rgba(1., 1., 1., 0.8)
                            }
                            else {
                                self.color
                            }
                        )
                        .apply(LineWidth(self.width))
                        .apply(&self.outline)
                        .stroke()
                }
            }
            Stage::AbandonedMarking => {
                if !self.open {
                    let mut canvas = canvas.sketch();
                    if let Some(dash) = self.dash {
                       canvas
                            .apply(self.color)
                            .apply(LineWidth(self.width))
                            .apply(dash)
                            .apply(&self.outline)
                            .stroke();
                    }
                    if let Some(electric) = self.electric {
                        electric.render(&self.outline, &mut canvas)
                    }
                }
            }
            Stage::LimitedBase => {
                if self.open && self.dash.is_some() {
                   canvas.sketch() 
                        .apply(Color::rgba(1., 1., 1., 0.8))
                        .apply(LineWidth(self.width))
                        .apply(&self.outline)
                        .stroke();
                }
            }
            Stage::LimitedMarking => {
                if self.open {
                    let mut canvas = canvas.sketch();
                    if let Some(dash) = self.dash {
                       canvas
                            .apply(self.color)
                            .apply(LineWidth(self.width))
                            .apply(dash)
                            .apply(&self.outline)
                            .stroke();
                    }
                    if let Some(electric) = self.electric {
                        electric.render(&self.outline, &mut canvas)
                    }
                }
            }
            Stage::Base => {
                if self.open && self.dash.is_none() {
                    let mut canvas = canvas.sketch();
                    canvas
                        .apply(self.color)
                        .apply(LineWidth(self.width))
                        .apply(&self.outline)
                        .stroke();
                    if let Some(electric) = self.electric {
                        electric.render(&self.outline, &mut canvas)
                    }
                }
            }
            _ => { }
        }
    }

    fn stages(&self) -> StageSet {
        let res = if self.casing_width.is_some() {
            StageSet::from(Stage::Casing)
        }
        else {
            StageSet::empty()
        };
        if !self.open {
            if self.dash.is_some() {
                res.add(Stage::AbandonedBase).add(Stage::AbandonedMarking)
            }
            else {
                res.add(Stage::AbandonedBase)
            }
        }
        else if self.dash.is_some() {
            res.add(Stage::LimitedBase).add(Stage::LimitedMarking)
        }
        else {
            res.add(Stage::Base)
        }
    }
}


//------------ ContourShape4 -------------------------------------------------

struct ContourShape4 {
    open: bool,
    color: Color,
    width: f64,
    casing_width: Option<f64>,
    dash: Option<(f64, f64)>, // on - off

    electric: Option<ElectricDecor>,

    outline: Outline,
}

impl ContourShape4 {
    fn new<'a>(contour: &'a TrackContour, style: &Style) -> AnyShape<'a> {
        let open = contour.class.class.status().is_open();
        let color = style.track_color(&contour.class.class);
        let width = style.measures().class_track(&contour.class.class);
        let casing_width = contour.casing.then(|| {
            width + 2. * style.measures().class_skip(&contour.class.class)
        });

        if contour.class.double() {
            let off = style.measures().class_offset(&contour.class.class) * 0.5;
            let left = contour.trace.outline_offset(off, style);
            let dash = Self::pax_dash(&contour.class, &left, style);

            let left_electric = ElectricDecor::new(
                &contour.class, contour.class.setup.double_left(),
                width, &left, style
            );
            let right_electric = ElectricDecor::new(
                &contour.class, contour.class.setup.double_right(),
                width, &left, style
            );

            let left_shape = Self {
                open, color, width, casing_width, dash,
                electric: left_electric,
                outline: left,
            };
            let right_shape = Self {
                open, color, width, casing_width, dash,
                electric: right_electric,
                outline: contour.trace.outline_offset(-off, style),
            };
            AnyShape::from((left_shape, right_shape))
        }
        else {
            let outline = contour.trace.outline(style);
            AnyShape::from(
                Self {
                    open, color, width, casing_width,
                    dash: Self::pax_dash(&contour.class, &outline, style),
                    electric: ElectricDecor::new(
                        &contour.class, contour.class.setup, width, &outline,
                        style
                    ),
                    outline
                }
            )
        }
    }

    fn pax_dash(
        class: &TrackClass, outline: &Outline, style: &Style
    ) -> Option<(f64, f64)> {
        // For historical reasons, an missing explicit pax defaults to no pax
        // for open lines and full pax for closed ones.
        if class.class.status().is_open() {
            if class.class.pax().is_full() {
                return None
            }
        }
        else {
            if class.class.opt_pax().unwrap_or(Pax::Full).is_full() {
                return None
            }
        }

        // If we have enough space, we want to operate with full segs so the
        // electrification and gauge markings are properly centered. Onyl if
        // that doesn’t work will be fall back to full “dash groups” since we
        // won’t have those markings.

        let seg = calc_seg(outline, style.measures().seg());

        if matches!(class.class.pax(), Pax::None) {
            let dist = match seg {
                Some(seg) => seg / NO_PAX_DASH_RATIO,
                None => {
                    calc_seg(
                        outline, style.measures().seg() / NO_PAX_DASH_RATIO,
                    )?
                }
            };
            Some((dist * NO_PAX_DASH_ON, dist * (1. - NO_PAX_DASH_ON)))
        }
        else {
            // XXX Fix this ...
            calc_seg(outline, style.measures().seg() * 0.25).map(|dist| {
                (dist * 0.8, dist * 0.2)
            })
        }
    }
}

impl<'a> Shape<'a> for ContourShape4 {
    fn render(&self, stage: Stage, _style: &Style, canvas: &mut Canvas) {
        match stage {
            Stage::Casing => {
                if let Some(width) = self.casing_width {
                    self.render_casing(width, &mut canvas.sketch());
                }
            }
            Stage::AbandonedBase => {
                if !self.open {
                    self.render_base(&mut canvas.sketch());
                }
            }
            Stage::AbandonedMarking => {
                if !self.open {
                    let mut canvas = canvas.sketch();
                    if let Some(dash) = self.dash {
                        self.render_dashed_track(dash, &mut canvas)
                    }
                    if let Some(electric) = self.electric {
                        electric.render(&self.outline, &mut canvas)
                    }
                }
            }
            Stage::LimitedBase => {
                if self.open && self.dash.is_some() {
                    self.render_base(&mut canvas.sketch());
                }
            }
            Stage::LimitedMarking => {
                if self.open {
                    let mut canvas = canvas.sketch();
                    if let Some(dash) = self.dash {
                        self.render_dashed_track(dash, &mut canvas)
                    }
                    if let Some(electric) = self.electric {
                        electric.render(&self.outline, &mut canvas)
                    }
                }
            }
            Stage::Base => {
                if self.open && !self.dash.is_some() {
                    let mut canvas = canvas.sketch();
                    self.render_base(&mut canvas);
                    if let Some(electric) = self.electric {
                        electric.render(&self.outline, &mut canvas)
                    }
                }
            }
            _ => { }
        }
    }

    fn stages(&self) -> StageSet {
        let res = if self.casing_width.is_some() {
            StageSet::from(Stage::Casing)
        }
        else {
            StageSet::empty()
        };
        if !self.open {
            if self.dash.is_some() {
                res.add(Stage::AbandonedBase).add(Stage::AbandonedMarking)
            }
            else {
                res.add(Stage::AbandonedBase)
            }
        }
        else if self.dash.is_some() {
            res.add(Stage::LimitedBase).add(Stage::LimitedMarking)
        }
        else {
            res.add(Stage::Base)
        }
    }
}

impl ContourShape4 {
    fn render_casing(&self, width: f64, canvas: &mut Sketch) {
        canvas.apply(
            Color::rgba(1., 1., 1., 0.8)
        ).apply(
            LineWidth(width)
        ).apply(
            &self.outline
        ).stroke();
    }

    fn render_base(&self, canvas: &mut Sketch) {
        canvas
            .apply(
                if self.dash.is_some() {
                    Color::rgba(1., 1., 1., 0.8)
                }
                else {
                    self.color
                }
            )
            .apply(LineWidth(self.width))
            .apply(&self.outline)
            .stroke()
    }

    fn render_dashed_track(
        &self, (on, off): (f64, f64), canvas: &mut Sketch
    ) {
       canvas.apply(self.color).apply(LineWidth(self.width));

        let mut positions = self.outline.positions();
        if positions.advance(0.5 * off).is_none() {
            return
        }
        loop {
            match positions.advance_sub(on) {
                Some(sub) => {
                    canvas.apply(&sub).stroke()
                }
                None => return
            }
            if positions.advance(off).is_none() {
                return
            }
        }
    }
}


//------------ ElectricDecore ------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct ElectricDecor {
    seg: f64,
    width: f64,
    cat: Option<CatDecor>,
    rail: Option<RailDecor>,
    dl: f64,
    dr: f64,
}

#[derive(Clone, Copy, Debug)]
struct CatDecor {
    color: Color,
    skip: f64,
}

#[derive(Clone, Copy, Debug)]
struct RailDecor {
    color: Color,
    skip: f64,
    offset: f64,
}

impl ElectricDecor {
    fn new(
        class: &TrackClass,
        setup: Setup,
        width: f64,
        outline: &Outline,
        style: &Style,
    ) -> Option<Self> {
        if class.class.station() || !class.class.category().is_railway() {
            return None
        }

        let dist = calc_seg(
            outline, style.measures().seg()
        )? / NO_PAX_DASH_RATIO;

        let (cat, rail) = match (
            style.cat_color(&class.class), style.rail_color(&class.class)
        ) {
            (Some(cat_color), None) => {
                (
                    Some(CatDecor {
                        color: cat_color,
                        skip: dist * (NO_PAX_DASH_HALF + 0.5)
                    }),
                    None
                )
            }
            (None, Some(rail_color)) => {
                (
                    None,
                    Some(RailDecor {
                        color: rail_color,
                        skip: dist * (NO_PAX_DASH_HALF - 0.5),
                        offset: dist,
                    })
                )
            }
            (Some(cat_color), Some(rail_color)) => {
                (
                    Some(CatDecor {
                        color: cat_color,
                        skip: dist * (NO_PAX_DASH_HALF * 0.5 + 0.5)
                    }),
                    Some(RailDecor {
                        color: rail_color,
                        skip: dist * (NO_PAX_DASH_HALF * 1.5 + 0.5),
                        offset: dist,
                    })
                )
            }
            (None, None) => return None
        };

        let (dl, dr) = Self::dl_dr(
            setup, width,
            style.measures().class_skip(&class.class),
            style.measures().class_track(&class.class),
        );

        Some(Self {
            seg: dist * NO_PAX_DASH_RATIO,
            width: dist * NO_PAX_DASH_ON * 0.9,
            cat, rail, dl, dr
        })
    }

    fn dl(setup: Setup, width: f64, skip: f64, track: f64) -> f64 {
        if matches!(setup.direction, Some(Direction::Updown)) {
            return 0.
        }
        match setup.left {
            Neighbor::None => -0.5 * width - track,
            Neighbor::Same(Direction::Updown) => 0.,
            Neighbor::Same(dir) if Some(dir) == setup.direction => {
                0.
            }
            Neighbor::Other => -0.5 * width - 0.2 * skip,
            _ => -0.5 * width - 0.5 * skip,
        }
    }

    fn dr(setup: Setup, width: f64, skip: f64, track: f64) -> f64 {
        if matches!(setup.direction, Some(Direction::Updown)) {
            return 0.
        }
        match setup.right {
            Neighbor::None => 0.5 * width + track,
            Neighbor::Same(Direction::Updown) => 0.,
            Neighbor::Same(dir) if Some(dir) == setup.direction => {
                0.
            }
            Neighbor::Other => 0.5 * width + 0.2 * skip,
            _ => 0.5 * width + 0.5 * skip,
        }
    }

    fn dl_dr(setup: Setup, width: f64, skip: f64, track: f64) -> (f64, f64) {
        (
            Self::dl(setup, width, skip, track),
            Self::dr(setup, width, skip, track)
        )
    }

    fn render(&self, outline: &Outline, canvas: &mut Sketch) {
        if let Some(cat) = self.cat {
            self.render_cat(outline, cat, canvas);
        }
        if let Some(rail) = self.rail {
            self.render_rail(outline, rail, canvas);
        }
    }

    fn render_cat(
        &self, outline: &Outline, cat: CatDecor, canvas: &mut Sketch
    ) {
        canvas.apply(LineWidth(self.width * 0.5));
        canvas.apply(cat.color);
        outline.iter_positions(
            self.seg, Some(cat.skip)
        ).for_each(|(pos, dir)| {
            let dir = Vec2::from_angle(dir + FRAC_PI_2);
            canvas.apply([
                PathEl::MoveTo(pos + dir * self.dl),
                PathEl::LineTo(pos + dir * self.dr),
            ]);
            canvas.stroke()
        });
    }

    fn render_rail(
        &self, outline: &Outline, rail: RailDecor, canvas: &mut Sketch
    ) {
        canvas.apply(LineWidth(self.width * 0.5));
        canvas.apply(rail.color);
        let mut positions = outline.positions();
        let (mut p1, mut d1) = match positions.advance(rail.skip) {
            Some(some) => some,
            None => return,
        };

        loop {
            let (p2, d2) = match positions.advance(rail.offset) {
                Some(some) => some,
                None => return,
            };

            let dir1 = Vec2::from_angle(d1 + FRAC_PI_2);
            let dir2 = Vec2::from_angle(d2 + FRAC_PI_2);

            canvas.apply([
                PathEl::MoveTo(p1 + dir1 * self.dl),
                PathEl::LineTo(p1 + dir1 * self.dr),
                PathEl::MoveTo(p2 + dir2 * self.dl),
                PathEl::LineTo(p2 + dir2 * self.dr),
            ]);
            canvas.stroke();

            (p1, d1) = match positions.advance(self.seg - rail.offset) {
                Some(some) => some,
                None => return,
            };
        }
    }
}


/*
impl<'a> ContourShape4<'a> {
    fn render_casing(&self, style: &Style, canvas: &mut Sketch) {
        self.render_track_casing(style, canvas);
        if !self.class.class.station() {
            self.render_electric_casing(style, canvas);
            //self.render_gauge(style, canvas);
        }
    }

    fn render_base(&self, style: &Style, canvas: &mut Sketch) {
        self.render_track(style, canvas);
        if !self.class.class.station() {
            self.render_electric(style, canvas);
            self.render_gauge(style, canvas);
        }
    }

    fn render_track(&self, style: &Style, canvas: &mut Sketch) {
        canvas.apply(style.track_color(&self.class.class));
        canvas.apply(LineWidth(self.line_width(style)));
        /*
        if self.class.combined {
            let seg = style.units().seg;
            canvas.apply(DashPattern::new(
                    [0.5 * seg, 0.5 * seg], 0.25 * seg
            ));
        }
        else if self.class.class.status().is_project() {
            let seg = style.units().seg;
            canvas.apply(DashPattern::new(
                [0.7 * seg, 0.3 * seg], 0.7 * seg
            ));
        }
        if !self.class.station && self.class.class.is_open_no_pax() {
            let seg = style.units().seg;
            let on = 0.9 * seg;
            let off = seg - on;
            let mut pos = self.track.positions();
            //pos.advance(0.5 * step + (self.track.arclen() % step) * 0.5);
            loop {
                let sub = match pos.advance_sub(on) {
                    Some(sub) => sub,
                    None => break,
                };
                canvas.apply(&sub);
                canvas.stroke();
                pos.advance(off);
            }
        }
        else {
            canvas.apply(&self.track);
            canvas.stroke();
        }
        */
        canvas.apply(&self.track);
        canvas.stroke();
        if !self.class.class.station() && self.class.class.is_open_no_pax() {
            let seg = match self.seg {
                Some(seg) => seg,
                None => return
            };
            let offset = self.line_width(style) * 0.35;

            canvas.apply(Color::WHITE);
            canvas.apply(LineWidth(self.line_width(style) * 0.71));
            let mut positions = self.track.positions();
            let sub = match positions.advance_sub(0.075 * seg) {
                Some(sub) => sub,
                None => return,
            };
            canvas.apply(&sub.offset(offset));
            canvas.stroke();
            loop {
                if positions.advance(0.85 * seg).is_none() {
                    return
                }
                let sub = match positions.advance_sub(0.15 * seg) {
                    Some(sub) => sub,
                    None => return,
                };
                canvas.apply(&sub.offset(offset));
                canvas.stroke();
            }
        }
    }

    fn render_track_casing(&self, style: &Style, canvas: &mut Sketch) {
        canvas.apply(style.casing_color());
        canvas.apply(LineWidth(
            1.2 * style.measures().dt()
        ));
        canvas.apply(&self.track);
        canvas.stroke();
    }

    /*
    fn render_tunnel(&self, style: &Style, canvas: &mut Sketch) {
        canvas.apply(Color::WHITE);
        canvas.apply(LineWidth(
            self.line_width(style) - style.units().guide_width
        ));
        canvas.apply(&self.track);
        canvas.stroke();
    }
    */
    fn render_electric(&self, style: &Style, canvas: &mut Sketch) {
        self.render_electric_common(style, canvas, false)
    }

    fn render_electric_casing(&self, style: &Style, canvas: &mut Sketch) {
        self.render_electric_common(style, canvas, true)
    }

    fn render_electric_common(
        &self, style: &Style, canvas: &mut Sketch, casing: bool
    ) {
        let seg = match self.seg {
            Some(seg) => seg,
            None => return
        };
        let cat_color = style.cat_color(&self.class.class);
        let rail_color = style.rail_color(&self.class.class);

        let dt = style.measures().dt();
        /*
        let dr = match self.right {
            Neighbor::None => 0.75 * dt,
            Neighbor::Same => 0.55 * dt, // Wee bit overlap.
            Neighbor::Other => 0.40 * dt,
        };
        let dl = match self.left {
            Neighbor::None => -0.75 * dt,
            Neighbor::Same => -0.55 * dt, // Wee bit overlap.
            Neighbor::Other => -0.40 * dt,
        };
        */
        let (dl, dr) = match self.right {
            Neighbor::None => (0., dt),
            _ => return,
        };

        match (cat_color, rail_color) {
            (Some(cat_color), None) => {
                if casing {
                    canvas.apply(LineWidth(
                        style.measures().line_width(&self.class.class) * 2.)
                    );
                    canvas.apply(LineCap::Round);
                    canvas.apply(style.casing_color());
                }
                else {
                    canvas.apply(LineWidth(
                        style.measures().line_width(&self.class.class)
                    ));
                    canvas.apply(cat_color);
                }
                self.track.iter_positions(
                    seg, Some(0.5 * seg)
                ).for_each(|(pos, dir)| {
                    let dir = Vec2::from_angle(dir + FRAC_PI_2);
                    canvas.apply([
                        PathEl::MoveTo(pos + dir * dl),
                        PathEl::LineTo(pos + dir * dr),
                    ]);
                    canvas.stroke()
                });
            }

            (None, Some(rail_color)) => {
                let mark_width = style.measures().line_width(
                    &self.class.class
                );
                let skip = mark_width * 3.;
                let seg = seg - skip;

                if casing {
                    canvas.apply(LineWidth(mark_width * 2.));
                    canvas.apply(LineCap::Round);
                    canvas.apply(style.casing_color());
                }
                else {
                    canvas.apply(LineWidth(mark_width));
                    canvas.apply(rail_color);
                }

                let mut positions = self.track.positions();
                let (mut p1, mut d1) = match positions.advance(0.5 * seg) {
                    Some(some) => some,
                    None => return,
                };

                loop {
                    let (p2, d2) = match positions.advance(skip) {
                        Some(some) => some,
                        None => return,
                    };

                    let dir1 = Vec2::from_angle(d1 + FRAC_PI_2);
                    let dir2 = Vec2::from_angle(d2 + FRAC_PI_2);

                    canvas.apply([
                        PathEl::MoveTo(p1 + dir1 * dl),
                        PathEl::LineTo(p1 + dir1 * dr),
                        PathEl::MoveTo(p2 + dir2 * dl),
                        PathEl::LineTo(p2 + dir2 * dr),
                    ]);
                    canvas.stroke();

                    (p1, d1) = match positions.advance(seg) {
                        Some(some) => some,
                        None => return,
                    };
                }
            }

            _ => { }
        }
    }

    fn render_gauge(&self, style: &Style, canvas: &mut Sketch) {
        if self.class.class.gauge().secondary().is_some() {
            self.render_multi_gauge(style, canvas)
        }
        else if !matches!(
            self.class.class.gauge_group(), GaugeGroup::Standard
        ) {
            self.render_gauge_group(style, canvas)
        }
    }

    fn render_multi_gauge(&self, style: &Style, canvas: &mut Sketch) {
        let seg = match self.seg {
            Some(seg) => seg,
            None => return
        };

        let radius = style.measures().dt() * 0.5;
        let halfwidth = style.measures().guide_width() * 0.5;
        let inner_radius = radius - halfwidth;
        let outer_radius = radius + halfwidth;

        self.track.iter_positions(
            seg, Some(0.5 * seg)
        ).for_each(|(pos, dir)| {
            canvas.apply(style.track_color(&self.class.class));
            canvas.apply(kurbo::Circle::new(pos, outer_radius));
            canvas.fill();
            canvas.apply(Color::WHITE);
            canvas.apply(kurbo::CircleSegment::new(
                pos, inner_radius, 0., dir + 0.5 * PI, PI,
            ));
            canvas.fill();
        })
    }

    fn render_gauge_group(&self, style: &Style, canvas: &mut Sketch) {
        use super::super::class::GaugeGroup::*;

        let group = self.class.class.gauge_group();
        if matches!(group, Standard) {
            return
        };

        let seg = match self.seg {
            Some(seg) => seg,
            None => return
        };
        let radius = style.measures().dt() * 0.5;
        let width = style.measures().guide_width();
        let radius_width = radius + 0.5 * width;

        match group {
            Narrower | Broader => {
                canvas.apply(style.track_color(&self.class.class));
            }
            Narrow | Broad => {
                canvas.apply(LineWidth(width));
            }
            Standard => { }
        }

        self.track.iter_positions(
            seg, Some(0.5 * seg)
        ).for_each(|(pos, dir)| {
            match group {
                Narrower => {
                    canvas.apply(kurbo::Circle::new(pos, radius_width));
                    canvas.fill();
                }
                Narrow => {
                    canvas.apply(kurbo::Circle::new(pos, radius));
                    canvas.apply(Color::WHITE);
                    canvas.fill();
                    canvas.apply(style.track_color(&self.class.class));
                    canvas.stroke();
                }
                Standard => { }
                Broad => {
                    let fwd = Vec2::from_angle(dir) * radius;
                    let side = Vec2::from_angle(dir + FRAC_PI_2) * radius;
                    canvas.apply([
                        PathEl::MoveTo(pos + fwd + side),
                        PathEl::LineTo(pos - fwd + side),
                        PathEl::LineTo(pos - fwd - side),
                        PathEl::LineTo(pos + fwd - side),
                        PathEl::ClosePath
                    ]);
                    canvas.apply(Color::WHITE);
                    canvas.fill();
                    canvas.apply(style.track_color(&self.class.class));
                    canvas.stroke();
                }
                Broader => {
                    let fwd = Vec2::from_angle(dir) * radius_width;
                    let side = Vec2::from_angle(dir + FRAC_PI_2) * radius_width;
                    canvas.apply([
                        PathEl::MoveTo(pos + fwd + side),
                        PathEl::LineTo(pos - fwd + side),
                        PathEl::LineTo(pos - fwd - side),
                        PathEl::LineTo(pos + fwd - side),
                        PathEl::ClosePath
                    ]);
                    canvas.fill();
                }
            }
        })
    }

    fn will_have_inside(_class: &TrackClass) -> bool {
        false
        /*
        if !class.class.is_open() {
            return false
        }
        matches!(class.class.pax(), Pax::None | Pax::Heritage)
        */
    }

    fn render_inside(&self, style: &Style, canvas: &mut Sketch) {
        let seg = match self.seg {
            Some(seg) => seg,
            None => return
        };

        if !self.class.class.is_open() {
            return;
        }

        canvas.apply(
            LineWidth(style.measures().line_inside(&self.class.class))
        );
        canvas.apply(Color::WHITE);
        let mut pos = self.track.positions();
        match self.class.class.pax() {
            Pax::None => {
                let sub = match pos.advance_sub(0.25 * seg) {
                    Some(sub) => sub,
                    None => return,
                };
                canvas.apply(&sub);
                canvas.stroke();
                let step = 0.5 * seg;
                loop {
                    pos.advance(step);
                    match pos.advance_sub(step) {
                        Some(sub) => {
                            canvas.apply(&sub);
                            canvas.stroke();
                        }
                        None => break,
                    }
                }
            }
            Pax::Heritage | Pax::Seasonal => {
                pos.advance(0.125 * seg);
                let step = 0.25 * seg;
                loop {
                    let sub = match pos.advance_sub(step) {
                        Some(sub) => sub,
                        None => break,
                    };
                    canvas.apply(&sub);
                    canvas.stroke();
                    pos.advance(step);
                }
            }
            _ => { }
        };
    }

    /*
    fn render_inside(&self, style: &Style, canvas: &mut Sketch) {
        if !self.class.class.is_open() {
            return;
        }

        match self.class.class.pax() {
            Pax::None => {
                if self.class.station {
                    canvas.apply(LineWidth(self.line_width(style) * 0.3));
                }
                else {
                    canvas.apply(LineWidth(self.line_width(style) * 0.4));
                }
                canvas.apply(Color::WHITE);
                canvas.apply(&self.track);
                canvas.stroke();
            }
            Pax::Heritage => {
                if self.class.station {
                    return
                }
                let step = 0.25 * style.units().seg;
                canvas.apply(LineWidth(self.line_width(style) * 0.5));
                canvas.apply(Color::WHITE);
                let mut pos = self.track.positions();
                pos.advance(0.5 * step + (self.track.arclen() % step) * 0.5);
                loop {
                    let sub = match pos.advance_sub(step) {
                        Some(sub) => sub,
                        None => break,
                    };
                    canvas.apply(&sub);
                    canvas.stroke();
                    pos.advance(step);
                }
            }
            _ => return
        };
    }
    */

    fn line_width(&self, style: &Style) -> f64 {
        style.measures().line_width(&self.class.class)
    }
}
*/


//------------ TrackCasing ---------------------------------------------------

/// The markings attached to a track.
pub struct TrackCasing {
    class: TrackClass,
    trace: Trace,
}

impl TrackCasing {
    pub fn new(class: TrackClass, trace: Trace) -> Self {
        TrackCasing { class, trace }
    }
}

impl Feature for TrackCasing {
    fn storage_bounds(&self) -> world::Rect {
        self.trace.storage_bounds()
    }

    fn group(&self) -> super::Group {
        super::Group::with_railway(Category::Track, &self.class.class)
    }

    fn shape(
        &self, style: &Style, _canvas: &Canvas
    ) -> AnyShape<'_> {
        let line_width = if self.class.double() {
            2.2 * style.measures().dt()
        }
        else {
            1.2 * style.measures().dt()
        };

        AnyShape::single_stage(move |style: &Style, canvas: &mut Canvas| {
            let mut canvas = canvas.sketch();
            canvas.apply(Color::rgba(1., 1., 1., 0.7));
            canvas.apply(LineWidth(line_width));
            canvas.apply(self.trace.iter_outline(style));
            canvas.stroke();
        })
    }
}


//------------ Helper Functions ----------------------------------------------

fn calc_seg(
    outline: &Outline, base_seg: f64
) -> Option<f64> {
    let len = outline.base_arclen();
    if len < base_seg {
        return None
    }
    let div = len / base_seg;
    let full = if div.fract() > 0.5 {
        div.trunc() + 1.
    }
    else {
        div.trunc()
    };
    Some(len / full)
}


