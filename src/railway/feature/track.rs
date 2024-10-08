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
use super::{AnyShape, Category, Feature, Shape, Stage};


//------------ Constants -----------------------------------------------------

/// Multiplier over line_width for double track in detail 2.
pub const D2_DOUBLE_MULTIPLIER: f64 = 1.6;


//------------ TrackClass ----------------------------------------------------

/// The properties of the track.
#[derive(Clone, Debug)]
pub struct TrackClass {
    /// The feature class.
    class: Railway,

    /// What is to our left?
    left: Neighbor,

    /// What is to our left?
    right: Neighbor,

    /// Should this track be combined with the underlying track?
    combined: bool,
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
        let tight = symbols.take("tight");
        let _ = symbols.take("flip"); // XXX Deprecated.
        TrackClass {
            class: Railway::from_symbols(symbols, scope),
            left: Neighbor::left_from_symbols(symbols, tight),
            right: Neighbor::right_from_symbols(symbols, tight),
            combined: symbols.take("combined"),
        }
    }

    pub fn class(&self) -> &Railway {
        &self.class
    }

    pub fn double(&self) -> bool {
        self.class.double()
    }
}


//------------ Neighbor ------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
pub enum Neighbor {
    /// There is no neighbor at all.
    #[default]
    None,

    /// The neighboring track is part of the same line.
    Same,

    /// The neighboring track is part of a different line.
    Other,
}

impl Neighbor {
    fn left_from_symbols(symbols: &mut SymbolSet, tight: bool) -> Self {
        if tight {
            Self::Other
        }
        else if symbols.take("leftsame") {
            Self::Same
        }
        else if symbols.take("leftother") {
            Self::Other
        }
        else {
            Self::None
        }
    }

    fn right_from_symbols(symbols: &mut SymbolSet, tight: bool) -> Self {
        if tight {
            Self::Other
        }
        else if symbols.take("rightsame") {
            Self::Same
        }
        else if symbols.take("rightother") {
            Self::Other
        }
        else {
            Self::None
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
    ) -> AnyShape {
        if style.detail() <= 1 {
            AnyShape::from(
                ContourShape::new(&self.class, self.trace.outline(style))
            )
        }
        else if style.detail() == 2 {
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
}

impl<'a> ContourShape<'a> {
    fn apply_line_width(&self, style: &Style, canvas: &mut Group) {
        let line_width = if style.detail() < 1 {
            style.measures().main_width()
        }
        else if self.class.double() {
            style.measures().line_width(&self.class.class) * 1.4
        }
        else {
            style.measures().line_width(&self.class.class)
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


//------------ ContourShape2 -------------------------------------------------

struct ContourShape2<'a> {
    class: &'a TrackClass,
    casing: bool,
    color: Color,
    width: f64,
    base_dash: Option<DashPattern<2>>,
    trace: Outline,
    has_inside: bool,

    /// Should we render the base during `Stage::InsideBase`?
    ///
    /// This is necessary so that non-open lines don’t draw over the inside
    /// of open lines.
    use_inside_base: bool,
}

impl<'a> ContourShape2<'a> {
    fn new(
        contour: &'a TrackContour,
        style: &Style,
    ) -> Self {
        let has_inside = Self::will_have_inside(&contour.class, style);
        Self {
            class: &contour.class,
            casing: contour.casing,
            color: style.track_color(&contour.class.class),
            width: if contour.class.double() {
                style.measures().double_width()
            }
            else {
                style.measures().line_width(&contour.class.class)
            },
            base_dash: if contour.class.combined {
                Some(DashPattern::new(
                    [
                        0.5 * style.measures().seg(),
                        0.5 * style.measures().seg()
                    ],
                    0.75 * style.measures().seg()
                ))
            }
            else if contour.class.class.status().is_project() {
                Some(DashPattern::new(
                    [
                        0.7 * style.measures().seg(),
                        0.3 * style.measures().seg(),
                    ],
                    0.85 * style.measures().seg()
                ))
            }
            else {
                None
            },
            trace: contour.trace.outline(style),
            has_inside,
            use_inside_base: has_inside || !contour.class.class.is_open(),
        }
    }
}

impl<'a> Shape<'a> for ContourShape2<'a> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        match stage {
            Stage::Casing => {
                if self.casing {
                    canvas.sketch().apply(
                        Color::rgba(1., 1., 1., 0.8)
                    ).apply(
                        LineWidth(1.6 * self.width)
                    ).apply(
                        &self.trace
                    ).stroke();
                }
            }
            Stage::InsideBase => {
                if self.use_inside_base {
                    self.render_base(style, canvas);
                }
            }
            Stage::Inside => {
                if self.has_inside {
                    self.render_inside(style, canvas);
                }
            }
            Stage::Base => {
                if !self.use_inside_base {
                    self.render_base(style, canvas);
                }
            }
            _ => { }
        }
    }
}

impl<'a> ContourShape2<'a> {
    fn render_base(&self, _style: &Style, canvas: &mut Canvas) {
        let mut canvas = canvas.sketch();
        canvas.apply(self.color);
        canvas.apply(LineWidth(self.width));
        if let Some(dash) = self.base_dash {
            canvas.apply(dash);
        }
        canvas.apply(&self.trace);
        canvas.stroke();
    }

    fn will_have_inside(class: &TrackClass, style: &Style) -> bool {
        if !class.class.is_open() {
            return false
        }
        match (style.pax_only(), class.class.pax()) {
            (false, Pax::None) => true,
            (_, Pax::Heritage | Pax::Seasonal) => true,
            _ => false,
        }
    }

    fn render_inside(&self, style: &Style, canvas: &mut Canvas) {
        if !self.class.class.is_open() {
            return
        }

        let seg = style.measures().seg();
        let mut canvas = canvas.sketch();

        match (style.pax_only(), self.class.class.pax()) {
            (false, Pax::None) => { }
            (_, Pax::Heritage | Pax::Seasonal) => {
                if self.class.class.station() {
                    return
                }
                canvas.apply(DashPattern::new(
                    [0.25 * seg, 0.25 * seg],
                    0.375 * seg
                ));
            }
            _ => return
        };
        canvas.apply(LineWidth(self.width * 0.4));
        canvas.apply(Color::WHITE);
        canvas.apply(&self.trace);
        canvas.stroke();
    }
}


//------------ ContourShape3 -------------------------------------------------

struct ContourShape3<'a> {
    class: &'a TrackClass,
    casing: bool,
    color: Color,
    width: f64,
    inside_width: Option<f64>,
    base_dash: Option<DashPattern<2>>,
    trace: Outline,

    /// Should we render the base during `Stage::InsideBase`?
    ///
    /// This is necessary so that non-open lines don’t draw over the inside
    /// of open lines.
    use_inside_base: bool,
}

impl<'a> ContourShape3<'a> {
    fn new(
        contour: &'a TrackContour,
        style: &Style,
    ) -> Self {
        let has_inside = Self::will_have_inside(&contour.class, style);
        Self {
            class: &contour.class,
            casing: contour.casing,
            color: style.track_color(&contour.class.class),
            width: if contour.class.double() {
                style.measures().double_width()
            }
            else {
                style.measures().line_width(&contour.class.class)
            },
            inside_width: has_inside.then(|| {
                if contour.class.double() {
                    style.measures().double_inside()
                }
                else {
                    style.measures().line_inside(&contour.class.class)
                }
            }),
            base_dash: if contour.class.combined {
                Some(DashPattern::new(
                    [
                        0.5 * style.measures().seg(),
                        0.5 * style.measures().seg()
                    ],
                    0.75 * style.measures().seg()
                ))
            }
            else if contour.class.class.status().is_project() {
                Some(DashPattern::new(
                    [
                        0.7 * style.measures().seg(),
                        0.3 * style.measures().seg(),
                    ],
                    0.85 * style.measures().seg()
                ))
            }
            else {
                None
            },
            trace: contour.trace.outline(style),
            use_inside_base: has_inside || !contour.class.class.is_open(),
        }
    }
}

impl<'a> Shape<'a> for ContourShape3<'a> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        match stage {
            Stage::Casing => {
                if self.casing {
                    canvas.sketch().apply(
                        Color::rgba(1., 1., 1., 0.8)
                    ).apply(
                        LineWidth(1.6 * self.width)
                    ).apply(
                        &self.trace
                    ).stroke();
                }
            }
            Stage::InsideBase => {
                if self.use_inside_base {
                    self.render_base(style, canvas);
                }
            }
            Stage::Inside => {
                if let Some(width) = self.inside_width {
                    self.render_inside(style, canvas, width);
                }
            }
            Stage::Base => {
                if !self.use_inside_base {
                    self.render_base(style, canvas);
                }
            }
            _ => { }
        }
    }
}

impl<'a> ContourShape3<'a> {
    fn render_base(&self, _style: &Style, canvas: &mut Canvas) {
        let mut canvas = canvas.sketch();
        canvas.apply(self.color);
        canvas.apply(LineWidth(self.width));
        if let Some(dash) = self.base_dash {
            canvas.apply(dash);
        }
        canvas.apply(&self.trace);
        canvas.stroke();
    }

    fn will_have_inside(class: &TrackClass, style: &Style) -> bool {
        if !class.class.is_open() || !class.class.category().is_main() {
            return false
        }
        match (style.pax_only(), class.class.pax()) {
            (false, Pax::None) => true,
            (_, Pax::Heritage | Pax::Seasonal) => true,
            _ => false,
        }
    }

    fn render_inside(
        &self, style: &Style, canvas: &mut Canvas, width: f64
    ) {
        let seg = style.measures().seg();
        let mut canvas = canvas.sketch();

        match (style.pax_only(), self.class.class.pax()) {
            (false, Pax::None) => { }
            (_, Pax::Heritage | Pax::Seasonal) => {
                if self.class.class.station() {
                    return
                }
                canvas.apply(DashPattern::new(
                    [0.25 * seg, 0.25 * seg],
                    0.375 * seg
                ));
            }
            _ => return
        };
        canvas.apply(LineWidth(width));
        canvas.apply(Color::WHITE);
        canvas.apply(&self.trace);
        canvas.stroke();
    }
}


//------------ ContourShape4 -------------------------------------------------

struct ContourShape4<'a> {
    class: &'a TrackClass,

    #[allow(dead_code)]
    left: Neighbor,

    right: Neighbor,
    casing: bool,
    track: Outline,
    seg: Option<f64>,
    has_inside: bool,

    /// Should we render the base during `Stage::InsideBase`?
    ///
    /// This is necessary so that non-open lines don’t draw over the inside
    /// of open lines.
    use_inside_base: bool,
}

impl<'a> ContourShape4<'a> {
    fn new(contour: &'a TrackContour, style: &Style) -> AnyShape<'a> {
        let has_inside = Self::will_have_inside(&contour.class);
        let use_inside_base = has_inside || !contour.class.class.is_open();
        if contour.class.double() {
            let off = style.measures().dt() * 0.5;
            let left = contour.trace.outline_offset(off, style);
            let seg = Self::calc_seg(&contour.class, &left, style);

            let left_shape = Self {
                class: &contour.class,
                left: contour.class.left,
                right: Neighbor::Same,
                casing: contour.casing,
                track: left,
                seg,
                has_inside,
                use_inside_base,
            };
            let right_shape = Self {
                class: &contour.class,
                left: Neighbor::Same,
                right: contour.class.right,
                casing: contour.casing,
                track: contour.trace.outline_offset(-off, style),
                seg,
                has_inside,
                use_inside_base,
            };
            AnyShape::from(move |stage, style: &_, canvas: &mut _| {
                left_shape.render(stage, style, canvas);
                right_shape.render(stage, style, canvas);
            })
        }
        else {
            let track = contour.trace.outline(style);
            let seg = Self::calc_seg(&contour.class, &track, style);
            AnyShape::from(
                Self {
                    class: &contour.class,
                    left: contour.class.left,
                    right: contour.class.right,
                    casing: contour.casing,
                    track,
                    seg,
                    has_inside,
                    use_inside_base,
                }
            )
        }
    }

    fn calc_seg(
        _class: &TrackClass, outline: &Outline, style: &Style
    ) -> Option<f64> {
        /*
        if class.station {
            return None
        }
        */
        let len = outline.base_arclen();
        let base_seg = style.measures().seg();
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
}

impl<'a> Shape<'a> for ContourShape4<'a> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        match stage {
            Stage::Casing => {
                if self.casing {
                    self.render_casing(style, &mut canvas.sketch());
                }
            }
            Stage::InsideBase => {
                if self.use_inside_base {
                    self.render_base(style, &mut canvas.sketch());
                }
            }
            Stage::Inside => {
                if self.has_inside {
                    self.render_inside(style, &mut canvas.sketch());
                }
            }
            Stage::Base => {
                if !self.use_inside_base {
                    self.render_base(style, &mut canvas.sketch());
                }
            }
            _ => { }
        }
    }
}

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
    ) -> AnyShape {
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

