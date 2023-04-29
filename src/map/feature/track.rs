//! Rendering of track.
//!
//! Track is a contour feature with complex rendering rules depending on the
//! class, style, and detail level.

use kurbo::{BezPath, Rect};
use femtomap::path::Trace;
use femtomap::render::{Canvas, Color, DashPattern, Group, LineWidth, Outline};
use crate::import::eval;
use crate::import::Failed;
use crate::import::eval::{Expression, SymbolSet};
use crate::theme::Style as _;
use super::super::class::{Category, Class, Gauge/*, GaugeGroup*/};
use super::super::style::Style;
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
        || !self.gauge.main_group().is_standard()
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
        if style.detail() == 2 {
            return Box::new(ContourShape2::new(self, style));
        }

        let yes = !self.class.station;

        if self.class.double && style.detail() >= 3 {
            let left = self.trace.outline_offset(
                -0.5 * style.dimensions().dt,
                style
            );
            let right = self.trace.outline_offset(
                0.5 * style.dimensions().dt,
                style
            );
            if self.class.flip {
                Box::new([
                    ContourShape::new(
                        &self.class, self.casing, false, yes, left
                    ),
                    ContourShape::new(
                        &self.class, self.casing, yes, true, right
                    ),
                ])
            }
            else {
                Box::new([
                    ContourShape::new(
                        &self.class, self.casing, yes, false, left
                    ),
                    ContourShape::new(
                        &self.class, self.casing, false, yes, right
                    ),
                ])
            }
        }
        else {
            let trace = self.trace.outline(style);
            Box::new(
                ContourShape::new(&self.class, self.casing, yes, yes, trace)
            )
        }
    }
}


//------------ ContourShape --------------------------------------------------

struct ContourShape<'a> {
    class: &'a TrackClass,
    casing: bool,
    left: bool,
    right: bool,
    trace: Outline,
}

impl<'a> ContourShape<'a> {
    fn new(
        class: &'a TrackClass,
        casing: bool,
        left: bool,
        right: bool,
        trace: Outline,
    ) -> Self {
        Self { class, casing, left, right, trace }
    }
}


impl<'a> Shape for ContourShape<'a> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        let canvas = canvas.sketch().into_group();
        match stage {
            Stage::Casing => {
                if self.casing {
                    self.render_casing(style, canvas);
                }
            }
            Stage::Base => {
                match style.detail() {
                    0 => self.render_detail_0(style, canvas),
                    1 => self.render_detail_1(style, canvas),
                    2 => self.render_detail_2(style, canvas),
                    _ => self.render_detail_full(style, canvas),
                }
            }
            Stage::Inside => {
                if self.class.class().surface().is_tunnel()
                    && style.detail() >= 3
                {
                    //self.render_tunnel_full(style, canvas);
                }
            }
            _ => { }
        }
     }
}

impl<'a> ContourShape<'a> {

    fn render_casing(&self, style: &Style, mut canvas: Group) {
        canvas.apply(Color::rgba(1., 1., 1., 0.7));
        canvas.apply_line_width(self.casing_width(style));
        canvas.apply(&self.trace);
        canvas.stroke();
    }

    fn render_detail_0(&self, style: &Style, mut canvas: Group) {
        canvas.apply_line_width(style.dimensions().line_width);
        canvas.apply(style.track_color(&self.class.class));
        canvas.apply(&self.trace);
        canvas.stroke()
    }

    fn render_detail_1(&self, style: &Style, mut canvas: Group) {
        if self.class.double {
            canvas.apply_line_width(
                style.dimensions().line_width * 1.4
            );
        }
        else {
            canvas.apply_line_width(
                style.dimensions().line_width * 1.0
            );
        }
        canvas.apply(style.track_color(&self.class.class));
        canvas.apply(&self.trace);
        canvas.stroke()
    }

    fn render_detail_2(&self, style: &Style, mut canvas: Group) {
        let units = style.dimensions();
        if self.class.class.category().is_main()
            || self.class.class.category().is_tram()
        {
            if self.class.double {
                canvas.apply_line_width(2.0 * units.line_width);
            }
            else {
                canvas.apply_line_width(units.line_width);
            }
        }
        else {
            canvas.apply_line_width(units.other_width);
        }
        if self.class.combined {
            canvas.apply(DashPattern::new(
                [0.5 * units.seg, 0.5 * units.seg],
                0.25 * units.seg
            ))
        }
        else if self.class.class.status().is_project() {
            canvas.apply(DashPattern::new(
                [0.7 * units.seg, 0.3 * units.seg],
                0.15 * units.seg
            ))
        }
        canvas.apply(style.track_color(&self.class.class));
        canvas.apply(&self.trace);
        canvas.stroke()
    }

    fn render_detail_full(&self, style: &Style, mut canvas: Group) {
        self.render_full_electric(style, &mut canvas);
        /*
        if self.class.has_property() {
            self.render_full_property(true, style, canvas);
        }
        */
        self.render_full_base(style, &mut canvas);
    }

    fn render_full_electric(&self, style: &Style, canvas: &mut Group) {
        if !self.left {
            return
        }
        let cat_color = style.cat_color(&self.class.class);
        let rail_color = style.rail_color(&self.class.class);
        if cat_color.is_none() && rail_color.is_none() {
            return;
        }

        
    }

    /*
    fn render_full_electric(
        &self, single: bool, style: &Style, mut canvas: Group
    ) {
        if self.class.station {
            return
        }
        let cat_color = style.cat_color(&self.class.class);
        let rail_color = style.rail_color(&self.class.class);
        if cat_color.is_none() && rail_color.is_none() {
            return;
        }
        let seg = style.dimensions().seg;
        let stroke = if self.class.class.category().is_main() {
            style.dimensions().line_width
        } else {
            style.dimensions().other_width
        };

        if single {
            self.trace.apply_offset(
                self.class.maybe_flip(
                    0.5 * style.dimensions().mark(self.class.tight)
                ),
                &mut canvas,
                style,
            );
            canvas.apply_line_width(
                style.dimensions().mark(self.class.tight)
            );
        }
        else {
            self.trace.apply(&mut canvas, style);
            canvas.apply_line_width(
                style.dimensions().dt
            );
        }

        if let Some(cat_color) = cat_color {
            canvas.apply(cat_color);
            if rail_color.is_none() {
                // We only have cat. This means one stroke in the center of
                // the seg.
                canvas.apply(DashPattern::new(
                    [stroke, seg - stroke],
                    0.5 * (seg - stroke)
                ));
                /*
                // We only have cat. This means we have to draw 0.3seg in
                // the center of each seg.
                canvas.set_dash(
                    &[0.3 * seg, 0.7 * seg],
                    0.45 * seg
                );
                */
                canvas.stroke()
            }
            else {
                // There also is rail. This means one stroke one third into
                // the seg.
                canvas.apply(DashPattern::new(
                    [stroke, seg - stroke],
                    (1./3.) * (seg - stroke)
                ));
                /*
                // There also is rail. Which means we have to draw 0.3seg in
                // the center of the first half of each double seg.
                canvas.set_dash(
                    &[0.3 * seg, 1.7 * seg],
                    0.45 * seg
                );
                */
                canvas.stroke();
            }
        }

        if let Some(rail_color) = rail_color {
            canvas.apply(rail_color);
            if cat_color.is_none() {
                // We only have third rail. This means we have two strokes
                // around the center of the seg. The strokes are 1 stroke wide
                // and 1.5 strokes apart.
                canvas.apply(DashPattern::new(
                    [
                        stroke, stroke, stroke,
                        seg - 3. * stroke
                    ],
                    0.5 * (seg - 3. * stroke)
                ));
                /*
                // We only have third rail. This means we have to draw a
                // 0.3seg made from one 0.05seg and one 0.2seg segment in
                // the center of each seg.
                canvas.set_dash(
                    &[0.05 * seg, 0.05 * seg, 0.2 * seg, 0.7 * seg],
                    0.45 * seg
                );
                */
            }
            else {
                // We have both cat and rail. This means our two strokes
                // around around the second third of the seg.
                canvas.apply(DashPattern::new(
                    [
                        stroke, stroke, stroke,
                        seg - 3. * stroke
                    ],
                    (2./3.) * (seg - 3. * stroke)
                ));
                /*
                // We have both cat and rail. This means our bit goes in
                // the center of the second half of the double seg.
                canvas.set_dash(
                    &[0.05 * seg, 0.05 * seg, 0.2 * seg, 1.7 * seg],
                    1.45 * seg
                );
                */
            }
            canvas.stroke()
        }

    }

    fn render_full_property(
        &self, _single: bool, _style: &Style, _canvas: &Canvas
    ) {
        if self.class.station {
            return
        }

        let category = match self.class.class.category() {
            Category::Second => 1,
            Category::Third => 2,
            _ => 0
        };
        let gauge = match self.class.gauge.main_group() {
            GaugeGroup::Minimum | GaugeGroup::Narrower => 2,
            GaugeGroup::Narrow => 1,
            GaugeGroup::Standard => 0,
            GaugeGroup::Broad => 3,
        };
        let seg = style.dimensions().seg;

        style.track_color(&self.class.class).apply(canvas);

        if category > 0 {
            let width = style.dimensions().mark(self.class.tight);
            let offset = self.class.maybe_flip(if single {
                -0.5 * width
            }
            else {
                -0.5 * width - 0.5 * style.dimensions().dt
            });
            self.trace.apply_offset(offset, canvas, style);
            let stroke = if self.class.class.category().is_main() {
                style.dimensions().line_width
            } else {
                style.dimensions().other_width
            };

            if gauge == 0 {
                // We don’t have gauge markings. So the category strokes go
                // around the center of a seg. They are 1 stroke wide and 2
                // strokes apart.
                match category {
                    1 => {
                        canvas.set_dash(
                            &[stroke, seg - stroke],
                            0.5 * (seg - stroke)
                        );
                    }
                    2 => {
                        canvas.set_dash(
                            &[
                                stroke, 2. * stroke, stroke,
                                seg - 4. * stroke
                            ],
                            0.5 * (seg - 4. * stroke)
                        );
                    }
                    _ => unreachable!()
                }
            }
            else {
                // We have gauge markings, so the category markings go into
                // the first half of a double seg.
                match category {
                    1 => {
                        canvas.set_dash(
                            &[stroke, 2. * seg - stroke],
                            0.5 * (seg - stroke)
                        );
                    }
                    2 => {
                        canvas.set_dash(
                            &[
                                stroke, 2. * stroke, stroke,
                                2. * seg - 4. * stroke
                            ],
                            0.5 * (seg - 4. * stroke)
                        );
                    }
                    _ => unreachable!()
                }
            }
            canvas.set_line_width(width);
            canvas.stroke().unwrap();
        }

        if gauge > 0 {
            let mark = style.dimensions().mark(self.class.tight);
            let width = 0.5 * mark;
            let mut offset = mark - 0.5 * width;
            if !single {
                offset += 0.5 * style.dimensions().dt;
            }
            let offset = self.class.maybe_flip(offset);
            self.trace.apply_offset(offset, canvas, style);
            let epsilon = 0.1 * width;
            let apart = 2. * width;

            if category == 0 {
                // We don’t have category markings, so the gauge markings go
                // in the center of each seg.
                // We make the dots by drawing on dashes epsilon long and set
                // the line cap to round. 
                match gauge {
                    1 => {
                        canvas.set_dash(
                            &[epsilon, seg - epsilon],
                            0.5 * (seg - epsilon)
                        );
                    }
                    2 => {
                        canvas.set_dash(
                            &[
                                epsilon, apart, epsilon,
                                seg - (2. * epsilon + apart)
                            ],
                            0.5 * (seg - (2. * epsilon + apart))
                        );
                    }
                    3 => {
                        canvas.set_dash(
                            &[
                                epsilon, apart, epsilon, apart, epsilon,
                                seg - (3. * epsilon + 2. * apart)
                            ],
                            0.5 * (seg - (3. * epsilon + 2. * apart))
                        );
                    }
                    _ => unreachable!()
                }
            }

            canvas.set_line_cap(cairo::LineCap::Round);
            canvas.set_line_width(width);
            canvas.stroke().unwrap();
            canvas.set_line_cap(cairo::LineCap::Butt);
        }
    }
    */

    fn render_full_base(&self, style: &Style, canvas: &mut Group) {
        canvas.apply(&self.trace);
        canvas.apply_line_width(
            if self.class.class.category().is_main() {
                style.dimensions().line_width
            }
            else {
                style.dimensions().other_width
            }
        );
        canvas.apply(style.track_color(&self.class.class));
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
        canvas.stroke()
    }

    /*
    fn render_tunnel_full(&self, style: &Style, mut canvas: Group) {
        if self.class.double {
            let offset = style.dimensions().dt * 0.5;
            self.render_tunnel_base(Some(offset), style, &mut canvas);
            self.render_tunnel_base(Some(-offset), style, &mut canvas);
        }
        else {
            self.render_tunnel_base(None, style, &mut canvas);
        }
    }

    fn render_tunnel_base(
        &self,
        offset: Option<f64>,
        style: &Style, canvas: &mut Group
    ) {
        if let Some(offset) = offset {
            self.trace.apply_offset(offset, canvas, style);
        }
        else {
            self.trace.apply(canvas, style);
        }
        canvas.apply_line_width(
            0.5 * if self.class.class.category().is_main() {
                style.dimensions().line_width
            }
            else {
                style.dimensions().other_width
            }
        );
        canvas.apply(Color::WHITE);
        canvas.stroke();
    }
    */

    fn casing_width(&self, style: &Style) -> f64 {
        1.2 * style.dimensions().dt
    }
}

//------------ ContourShape2 -------------------------------------------------

struct ContourShape2 {
    casing: bool,
    color: Color,
    width: f64,
    dash: Option<DashPattern<2>>,
    trace: Outline,
}

impl ContourShape2 {
    fn new(
        contour: &TrackContour,
        style: &Style,
    ) -> Self {
        Self {
            casing: contour.casing,
            color: style.track_color(&contour.class.class),
            width: if contour.class.class.category().is_main() {
                if contour.class.double {
                    style.dimensions().line_width * 2.0
                }
                else {
                    style.dimensions().line_width
                }
            }
            else {
                style.dimensions().other_width
            },
            dash: if contour.class.combined {
                Some(DashPattern::new(
                    [
                        0.5 * style.dimensions().seg,
                        0.5 * style.dimensions().seg
                    ],
                    0.25 * style.dimensions().seg
                ))
            }
            else if contour.class.class.status().is_project() {
                Some(DashPattern::new(
                    [
                        0.7 * style.dimensions().seg,
                        0.3 * style.dimensions().seg
                    ],
                    0.15 * style.dimensions().seg
                ))
            }
            else {
                None
            },
            trace: contour.trace.outline(style)
        }
    }
}

impl Shape for ContourShape2 {
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
            Stage::Base => {
                let mut canvas = canvas.sketch();
                canvas.apply(self.color);
                canvas.apply(LineWidth(self.width));
                if let Some(dash) = self.dash {
                    canvas.apply(dash);
                }
                canvas.apply(&self.trace);
                canvas.stroke();
            }
            _ => { }
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

