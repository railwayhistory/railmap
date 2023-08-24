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
//! * `:rightsame`if the track has another track of the same line of its
//!   right.
//! * `:rightother` if the track has another track of a different line on its
//!   right.
//!
//! * In detail levels 3 and up, `:double` is a shortcut for two tracks offset
//!   half a dt to the left and right and with `:rightsame` and `:leftsame`.
//! * In detail levels 0 to 3, `:double` marks a track as double track.
//! * `:tight` is a deprecated shortcut for `:leftother:rightother`.
//!
//! Placement within a sequence of segments that whose markings should look
//! consecutive:
//!
//! * `:start` if the segment is at the start of the sequence, i.e., extra
//!   spacing can be added at its beginning.
//! * `:end` if the segement is at the end of the sequence, i.e., extra
//!   spacing can be added at its end.
//! * `:inner` if the segment is in the middle and markings need to be
//!   “justified.”

use std::f64::consts::FRAC_PI_2;
use kurbo::{BezPath, PathEl, Rect, Vec2};
use femtomap::path::Trace;
use femtomap::render::{
    Canvas, Color, DashPattern, Group, LineCap, LineWidth, Outline, Sketch,
};
use crate::import::eval;
use crate::import::Failed;
use crate::import::eval::{Expression, SymbolSet};
use crate::theme::Style as _;
use super::super::class::{Category, Class, Gauge/*, GaugeGroup*/, Pax, Status};
use super::super::style::{Palette, Style};
use super::super::theme::Railwayhistory;
use super::{Shape, Stage};


//------------ TrackClass ----------------------------------------------------

/// The properties of the track.
#[derive(Clone, Debug)]
pub struct TrackClass {
    /// The feature class.
    class: Class,

    /// The gauge of the track.
    gauge: Gauge,

    /// Is this double tracked?
    double: bool,

    /// Should this track be combined with the underlying track?
    combined: bool,

    /// Is this station track?
    station: bool,

    /// Are markings flipped?
    flip: bool,

    /// Should markings be smaller?
    tight: bool,
}

impl TrackClass {
    pub fn from_arg(
        arg: Expression<Railwayhistory>,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.into_symbol_set(err)?;
        let class = Self::from_symbols(&mut symbols);
        symbols.check_exhausted(err)?;
        Ok(class)
    }

    pub fn from_symbols(symbols: &mut SymbolSet) -> Self {
        TrackClass {
            class: Class::from_symbols(symbols),
            gauge: Gauge::from_symbols(symbols),
            double: symbols.take("double"),
            combined: symbols.take("combined"),
            station: symbols.take("station"),
            flip: symbols.take("flip"),
            tight: symbols.take("tight"),
        }
    }

    pub fn class(&self) -> &Class {
        &self.class
    }

    /// Returns whether the track needs property markings.
    fn has_property(&self) -> bool {
        matches!(self.class.category(), Category::Second | Category::Third)
        || self.gauge.group().is_some()
    }

    /// Inverts a value if the track is flipped.
    fn maybe_flip(&self, value: f64) -> f64 {
        if self.flip {
            -value
        }
        else {
            value
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

    pub fn storage_bounds(&self) -> Rect {
        self.trace.storage_bounds()
    }

    pub fn shape(
        &self, style: &Style, _canvas: &Canvas
    ) -> Box<dyn Shape + '_> {
        if style.detail_step() == 2 {
            return Box::new(ContourShape2::new(self, style));
        }
        else if style.detail_step() >= 3 {
            return ContourShape4::new(self, style)
        }
        else {
            Box::new(
                ContourShape::new(&self.class, self.trace.outline(style))
            )
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


impl<'a> Shape for ContourShape<'a> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        if !matches!(stage, Stage::Base) || self.is_hidden(style) {
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
    fn is_hidden(&self, style: &Style) -> bool {
        false
    }

    fn apply_line_width(&self, style: &Style, canvas: &mut Group) {
        let line_width = if style.detail() < 1. {
            style.dimensions().line_width
        }
        else if self.class.double {
            style.dimensions().line_width * 1.4
        }
        else {
            style.dimensions().line_width
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
}

impl<'a> ContourShape2<'a> {
    fn new(
        contour: &'a TrackContour,
        style: &Style,
    ) -> Self {
        Self {
            class: &contour.class,
            casing: contour.casing,
            color: style.track_color(&contour.class.class),
            width: if contour.class.class.category().is_main() {
                if contour.class.double {
                    style.dimensions().line_width * 2.0
                }
                else {
                    style.dimensions().line_width * 1.4
                }
            }
            else {
                style.dimensions().other_width * 1.2
            },
            base_dash: if contour.class.combined {
                Some(DashPattern::new(
                    [
                        0.5 * style.dimensions().seg,
                        0.5 * style.dimensions().seg
                    ],
                    0.75 * style.dimensions().seg
                ))
            }
            else if contour.class.class.status().is_project() {
                Some(DashPattern::new(
                    [
                        0.7 * style.dimensions().seg,
                        0.3 * style.dimensions().seg
                    ],
                    0.85 * style.dimensions().seg
                ))
            }
            else {
                None
            },
            trace: contour.trace.outline(style),
            has_inside: Self::will_have_inside(&contour.class, style),
        }
    }
}

impl<'a> Shape for ContourShape2<'a> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        match stage {
            Stage::Casing => {
                if self.casing {
                    canvas.sketch().apply(
                        Color::rgba(1., 1., 1., 0.7)
                    ).apply(
                        LineWidth(1.5 * self.width)
                    ).apply(
                        &self.trace
                    ).stroke();
                }
            }
            Stage::InsideBase => {
                if self.has_inside {
                    self.render_base(style, canvas);
                }
            }
            Stage::Inside => {
                if self.has_inside {
                    self.render_inside(style, canvas);
                }
            }
            Stage::Base => {
                if !self.has_inside {
                    self.render_base(style, canvas);
                }
            }
            _ => { }
        }
    }
}

impl<'a> ContourShape2<'a> {
    fn render_base(&self, style: &Style, canvas: &mut Canvas) {
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
        match (style.palette(), class.class.pax()) {
            (Palette::El | Palette::Proof, Pax::None) => true,
            (_, Pax::Heritage) => true,
            _ => false,
        }
    }

    fn render_inside(&self, style: &Style, canvas: &mut Canvas) {
        if !self.class.class.is_open() {
            return
        }

        let seg = style.dimensions().seg;
        let mut canvas = canvas.sketch();

        match (style.palette(), self.class.class.pax()) {
            (Palette::El | Palette::Proof, Pax::None) => { }
            (_, Pax::Heritage) => {
                if self.class.station {
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

//------------ ContourShape4 -------------------------------------------------

struct ContourShape4<'a> {
    class: &'a TrackClass,
    casing: bool,
    track: Outline,
    seg: Option<f64>,
    has_inside: bool,
}

impl<'a> ContourShape4<'a> {
    fn new(contour: &'a TrackContour, style: &Style) -> Box<dyn Shape + 'a> {
        let has_inside = Self::will_have_inside(&contour.class);
        if contour.class.double {
            let off = style.dimensions().dt * 0.5;
            let left = contour.trace.outline_offset(-off, style);
            let seg = Self::calc_seg(&contour.class, &left, style);
            Box::new([
                Self {
                    class: &contour.class,
                    casing: contour.casing,
                    track: left,
                    seg,
                    has_inside,
                },
                Self {
                    class: &contour.class,
                    casing: contour.casing,
                    track: contour.trace.outline_offset(off, style),
                    seg,
                    has_inside,
                }
            ])
        }
        else {
            let track = contour.trace.outline(style);
            let seg = Self::calc_seg(&contour.class, &track, style);
            Box::new(
                Self {
                    class: &contour.class,
                    casing: contour.casing,
                    track,
                    seg,
                    has_inside,
                }
            )
        }
    }

    fn calc_seg(
        class: &TrackClass, outline: &Outline, style: &Style
    ) -> Option<f64> {
        if class.station {
            return None
        }
        let len = outline.base_arclen();
        let base_seg = style.dimensions().seg;
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

impl<'a> Shape for ContourShape4<'a> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        match stage {
            Stage::Casing => {
                if self.casing {
                    let mut canvas = canvas.sketch();
                    canvas.apply(Color::rgba(1., 1., 1., 0.7));
                    canvas.apply(LineWidth(
                        1.2 * style.dimensions().dt
                    ));
                    canvas.apply(&self.track);
                    canvas.stroke();
                }
            }
            Stage::InsideBase => {
                if self.has_inside {
                    self.render_base(style, &mut canvas.sketch());
                }
            }
            Stage::Inside => {
                if self.has_inside {
                    self.render_inside(style, &mut canvas.sketch());
                }
            }
            Stage::Base => {
                if !self.has_inside {
                    self.render_base(style, &mut canvas.sketch());
                }
            }
            _ => { }
        }
    }
}

impl<'a> ContourShape4<'a> {
    fn render_base(&self, style: &Style, canvas: &mut Sketch) {
        self.render_track(style, canvas);
        if !self.class.station {
            self.render_electric(style, canvas);
            self.render_gauge(style, canvas);
        }
    }

    fn render_track(&self, style: &Style, canvas: &mut Sketch) {
        canvas.apply(style.track_color(&self.class.class));
        canvas.apply(LineWidth(self.line_width(style)));
        /*
        if self.class.combined {
            let seg = style.dimensions().seg;
            canvas.apply(DashPattern::new(
                    [0.5 * seg, 0.5 * seg], 0.25 * seg
            ));
        }
        else if self.class.class.status().is_project() {
            let seg = style.dimensions().seg;
            canvas.apply(DashPattern::new(
                [0.7 * seg, 0.3 * seg], 0.7 * seg
            ));
        }
        if !self.class.station && self.class.class.is_open_no_pax() {
            let seg = style.dimensions().seg;
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
        */
            canvas.apply(&self.track);
            canvas.stroke();
        //}
    }

    fn render_tunnel(&self, style: &Style, canvas: &mut Sketch) {
        canvas.apply(Color::WHITE);
        canvas.apply(LineWidth(
            self.line_width(style) - style.dimensions().guide_width
        ));
        canvas.apply(&self.track);
        canvas.stroke();
    }

    fn render_electric(&self, style: &Style, canvas: &mut Sketch) {
        let seg = match self.seg {
            Some(seg) => seg,
            None => return
        };
        let cat_color = style.cat_color(&self.class.class);
        let rail_color = style.rail_color(&self.class.class);

        let dt = style.dimensions().dt;
        let dr = if self.class.tight { 0.5 * dt } else { 0.75 * dt };
        /*
        let dl = -0.5 * dt;
        */
        let dl = -dr;
        let (dl, dr) = if self.class.flip { (dr, dl) } else { (dl, dr) };

        match (cat_color, rail_color) {
            (Some(cat_color), None) => {

                canvas.apply(LineWidth(style.dimensions().mark_width));
                canvas.apply(cat_color);
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
                let skip = style.dimensions().mark_width * 3.;
                let seg = seg - skip;

                canvas.apply(LineWidth(style.dimensions().mark_width));
                canvas.apply(rail_color);

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
        use super::super::class::GaugeGroup::*;

        let seg = match self.seg {
            Some(seg) => seg,
            None => return
        };
        let group = match self.class.gauge.group() {
            Some(group) => group,
            None => return,
        };
        let radius = style.dimensions().dt * 0.5;
        let width = style.dimensions().guide_width;
        let radius_width = radius + 0.5 * width;

        match group {
            Narrower | Broader => {
                canvas.apply(style.track_color(&self.class.class));
            }
            Narrow | Broad => {
                canvas.apply(LineWidth(width));
            }
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

    fn will_have_inside(class: &TrackClass) -> bool {
        if !class.class.is_open() {
            return false
        }
        matches!(class.class.pax(), Pax::None | Pax::Heritage)
    }

    fn render_inside(&self, style: &Style, canvas: &mut Sketch) {
        let seg = match self.seg {
            Some(seg) => seg,
            None => return
        };

        if !self.class.class.is_open() {
            return;
        }

        canvas.apply(LineWidth(self.line_width(style) * 0.5));
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
            Pax::Heritage => {
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
                let step = 0.25 * style.dimensions().seg;
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
        if self.class.class.category().is_main() {
            style.dimensions().line_width
        }
        else {
            style.dimensions().other_width
        }
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

    pub fn storage_bounds(&self) -> Rect {
        self.trace.storage_bounds()
    }

    pub fn shape(
        &self, _style: &Style, _canvas: &Canvas
    ) -> Box<dyn Shape + '_> {
        Box::new(|style: &Style, canvas: &mut Canvas| {
            let mut canvas = canvas.sketch();
            canvas.apply(Color::rgba(1., 1., 1., 0.7));
            canvas.apply(LineWidth(self.line_width(style)));
            canvas.apply(self.trace.iter_outline(style));
            canvas.stroke();
        })
    }

    fn line_width(&self, style: &Style) -> f64 {
        if self.class.double {
            2.2 * style.dimensions().dt
        }
        else {
            1.2 * style.dimensions().dt
        }
    }
}

