//! Rendering of track.
//!
//! Track is a contour feature with complex rendering rules depending on the
//! class, style, and detail level.

use kurbo::Rect;
use crate::import::eval;
use crate::import::Failed;
use crate::import::eval::{Expression, SymbolSet};
use crate::render::canvas::Canvas;
use crate::render::path::Trace;
use crate::theme::Style as _;
use super::super::class::{Category, Class, Gauge, GaugeGroup};
use super::super::style::Style;
use super::super::theme::Railwayhistory;


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
        let mut symbols = arg.into_symbol_set(err)?.0;
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
    trace: Trace,
}

impl TrackContour {
    pub fn new(class: TrackClass, trace: Trace) -> Self {
        TrackContour { class, trace }
    }

    pub fn storage_bounds(&self) -> Rect {
        self.trace.storage_bounds()
    }

    pub fn render(&self, style: &Style, canvas: &Canvas) {
        match style.detail() {
            0 => self.render_detail_0(style, canvas),
            1 => self.render_detail_1(style, canvas),
            2 => self.render_detail_2(style, canvas),
            3 => self.render_detail_3(style, canvas),
            _ => self.render_detail_full(style, canvas),
        }
    }

    fn render_detail_0(&self, style: &Style, canvas: &Canvas) {
        canvas.set_line_width(style.dimensions().line_width * 0.7);
        style.track_color(&self.class.class).apply(canvas);
        self.trace.apply(canvas, style);
        canvas.stroke().unwrap();
    }

    fn render_detail_1(&self, style: &Style, canvas: &Canvas) {
        if self.class.double {
            canvas.set_line_width(
                style.dimensions().line_width * 1.2
            );
        }
        else {
            canvas.set_line_width(
                style.dimensions().line_width * 0.7
            );
        }
        style.track_color(&self.class.class).apply(canvas);
        self.trace.apply(canvas, style);
        canvas.stroke().unwrap();
    }

    fn render_detail_2(&self, style: &Style, canvas: &Canvas) {
        let units = style.dimensions();
        if self.class.class.category().is_main()
            || self.class.class.category().is_tram()
        {
            if self.class.double {
                canvas.set_line_width(2.0 * units.line_width);
            }
            else {
                canvas.set_line_width(units.line_width);
            }
        }
        else {
            canvas.set_line_width(units.other_width);
        }
        if self.class.combined {
            canvas.set_dash(
                &[0.5 * units.seg, 0.5 * units.seg],
                0.25 * units.seg
            );
        }
        else if self.class.class.status().is_project() {
            canvas.set_dash(
                &[0.7 * units.seg, 0.3 * units.seg],
                0.15 * units.seg
            );
        }
        style.track_color(&self.class.class).apply(canvas);
        self.trace.apply(canvas, style);
        canvas.stroke().unwrap();
        canvas.set_dash(&[], 0.);
    }

    fn render_detail_3(&self, style: &Style, canvas: &Canvas) {
        self.render_detail_full(style, canvas)
    }

    fn render_detail_full(&self, style: &Style, canvas: &Canvas) {
        if self.class.double {
            self.render_full_double(style, canvas);
        }
        else {
            self.render_full_single(style, canvas);
        }
    }

    fn render_full_single(&self, style: &Style, canvas: &Canvas) {
        self.render_full_electric(true, style, canvas);
        if self.class.has_property() {
            self.render_full_property(true, style, canvas);
        }
        self.render_full_base(None, style, canvas);
    }

    fn render_full_double(&self, style: &Style, canvas: &Canvas) {
        self.render_full_electric(false, style, canvas);
        if self.class.has_property() {
            self.render_full_property(false, style, canvas);
        }
        let offset = style.dimensions().dt * 0.5;
        self.render_full_base(Some(offset), style, canvas);
        self.render_full_base(Some(-offset), style, canvas);
    }

    fn render_full_electric(
        &self, single: bool, style: &Style, canvas: &Canvas
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

        if single {
            self.trace.apply_offset(
                self.class.maybe_flip(
                    0.5 * style.dimensions().mark(self.class.tight)
                ),
                canvas,
                style,
            );
            canvas.set_line_width(
                style.dimensions().mark(self.class.tight)
            );
        }
        else {
            self.trace.apply(canvas, style);
            canvas.set_line_width(
                style.dimensions().dt
            );
        }

        if let Some(cat_color) = cat_color {
            cat_color.apply(canvas);
            if rail_color.is_none() {
                // We only have cat. This means we have to draw 0.3seg in
                // the center of each seg.
                canvas.set_dash(
                    &[0.3 * seg, 0.7 * seg],
                    0.45 * seg
                );
                canvas.stroke().unwrap();
            }
            else {
                // There also is rail. Which means we have to draw 0.3seg in
                // the center of the first half of each double seg.
                canvas.set_dash(
                    &[0.3 * seg, 1.7 * seg],
                    0.45 * seg
                );
                canvas.stroke_preserve().unwrap();
            }
        }

        if let Some(rail_color) = rail_color {
            rail_color.apply(canvas);
            if cat_color.is_none() {
                // We only have third rail. This means we have to draw a
                // 0.3seg made from one 0.05seg and one 0.2seg segment in
                // the center of each seg.
                canvas.set_dash(
                    &[0.05 * seg, 0.05 * seg, 0.2 * seg, 0.7 * seg],
                    0.45 * seg
                );
            }
            else {
                // We have both cat and rail. This means our bit goes in
                // the center of the second half of the double seg.
                canvas.set_dash(
                    &[0.05 * seg, 0.05 * seg, 0.2 * seg, 1.7 * seg],
                    1.45 * seg
                );
            }
            canvas.stroke().unwrap();
        }

    }

    fn render_full_property(
        &self, single: bool, style: &Style, canvas: &Canvas
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

    fn render_full_base(
        &self,
        offset: Option<f64>,
        style: &Style, canvas: &Canvas
    ) {
        if let Some(offset) = offset {
            self.trace.apply_offset(offset, canvas, style);
        }
        else {
            self.trace.apply(canvas, style);
        }
        canvas.set_line_width(
            if self.class.class.category().is_main() {
                style.dimensions().line_width
            }
            else {
                style.dimensions().other_width
            }
        );
        style.track_color(&self.class.class).apply(canvas);
        if self.class.combined {
            let seg = style.dimensions().seg;
            canvas.set_dash(&[0.5 * seg, 0.5 * seg], 0.25 * seg);
        }
        else if self.class.class.status().is_project() {
            let seg = style.dimensions().seg;
            canvas.set_dash(&[0.7 * seg, 0.3 * seg], 0.7 * seg);
        }
        else {
            canvas.set_dash(&[], 0.);
        }
        if let Some(offset) = offset {
            self.trace.apply_offset(offset, canvas, style);
        }
        else {
            self.trace.apply(canvas, style);
        }
        canvas.stroke().unwrap();
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

    pub fn render(&self, style: &Style, canvas: &Canvas) {
        canvas.set_source_rgba(1., 1., 1., 0.7);
        canvas.set_line_width(self.line_width(style));
        self.trace.apply(canvas, style);
        canvas.stroke().unwrap();
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

