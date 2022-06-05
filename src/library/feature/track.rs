//! Rendering of track.
//!
//! Track is a contour feature with complex rendering rules depending on the
//! class, style, and detail level.

use crate::canvas::Canvas;
use crate::features::contour::RenderContour;
use crate::features::path::Path;
use crate::import::eval;
use crate::import::Failed;
use crate::import::eval::{Expression, SymbolSet};
use super::super::class::{Category, Class, Gauge, GaugeGroup};


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
        arg: Expression,
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
    class: TrackClass
}

impl TrackContour {
    pub fn new(class: TrackClass) -> Self {
        TrackContour { class }
    }
}

impl RenderContour for TrackContour {
    fn render(&self, canvas: &Canvas, path: &Path) {
        match canvas.style().detail() {
            0 => self.render_detail_0(canvas, path),
            1 => self.render_detail_1(canvas, path),
            2 => self.render_detail_2(canvas, path),
            3 => self.render_detail_3(canvas, path),
            _ => self.render_detail_full(canvas, path),
        }
    }
}

impl TrackContour {
    fn render_detail_0(&self, canvas: &Canvas, path: &Path) {
        canvas.set_line_width(canvas.style().dimensions().line_width * 0.7);
        canvas.style().track_color(&self.class.class).apply(canvas);
        path.apply(canvas);
        canvas.stroke();
    }

    fn render_detail_1(&self, canvas: &Canvas, path: &Path) {
        if self.class.double {
            canvas.set_line_width(
                canvas.style().dimensions().line_width * 1.2
            );
        }
        else {
            canvas.set_line_width(
                canvas.style().dimensions().line_width * 0.7
            );
        }
        canvas.style().track_color(&self.class.class).apply(canvas);
        path.apply(canvas);
        canvas.stroke();
    }

    fn render_detail_2(&self, canvas: &Canvas, path: &Path) {
        let units = canvas.style().dimensions();
        if self.class.class.category().is_main() {
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
        canvas.style().track_color(&self.class.class).apply(canvas);
        path.apply(canvas);
        canvas.stroke();
        canvas.set_dash(&[], 0.);
    }

    fn render_detail_3(&self, canvas: &Canvas, path: &Path) {
        self.render_detail_full(canvas, path)
    }

    fn render_detail_full(&self, canvas: &Canvas, path: &Path) {
        if self.class.double {
            self.render_full_double(canvas, path);
        }
        else {
            self.render_full_single(canvas, path);
        }
    }

    fn render_full_single(&self, canvas: &Canvas, path: &Path) {
        self.render_full_electric(true, canvas, path);
        if self.class.has_property() {
            self.render_full_property(true, canvas, path);
        }
        self.render_full_base(None, canvas, path);
    }

    fn render_full_double(&self, canvas: &Canvas, path: &Path) {
        self.render_full_electric(false, canvas, path);
        if self.class.has_property() {
            self.render_full_property(false, canvas, path);
        }
        let offset = canvas.style().dimensions().dt * 0.5;
        self.render_full_base(Some(offset), canvas, path);
        self.render_full_base(Some(-offset), canvas, path);
    }

    fn render_full_electric(
        &self, single: bool, canvas: &Canvas, path: &Path,
    ) {
        if self.class.station {
            return
        }
        let cat_color = canvas.style().cat_color(&self.class.class);
        let rail_color = canvas.style().rail_color(&self.class.class);
        if cat_color.is_none() && rail_color.is_none() {
            return;
        }
        let seg = canvas.style().dimensions().seg;

        if single {
            path.apply_offset(
                self.class.maybe_flip(
                    0.5 * canvas.style().dimensions().mark(self.class.tight)
                ),
                canvas
            );
            canvas.set_line_width(
                canvas.style().dimensions().mark(self.class.tight)
            );
        }
        else {
            path.apply(canvas);
            canvas.set_line_width(
                canvas.style().dimensions().dt
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
                canvas.stroke();
            }
            else {
                // There also is rail. Which means we have to draw 0.3seg in
                // the center of the first half of each double seg.
                canvas.set_dash(
                    &[0.3 * seg, 1.7 * seg],
                    0.45 * seg
                );
                canvas.stroke_preserve();
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
            canvas.stroke();
        }

    }

    fn render_full_property(
        &self, single: bool, canvas: &Canvas, path: &Path,
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
        let seg = canvas.style().dimensions().seg;

        canvas.style().track_color(&self.class.class).apply(canvas);

        if category > 0 {
            let width = canvas.style().dimensions().mark(self.class.tight);
            let offset = self.class.maybe_flip(if single {
                -0.5 * width
            }
            else {
                -0.5 * width - 0.5 * canvas.style().dimensions().dt
            });
            path.apply_offset(offset, canvas);
            let stroke = if self.class.class.category().is_main() {
                canvas.style().dimensions().line_width
            } else {
                canvas.style().dimensions().other_width
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
            canvas.stroke();
        }

        if gauge > 0 {
            let mark = canvas.style().dimensions().mark(self.class.tight);
            let width = 0.5 * mark;
            let mut offset = mark - 0.5 * width;
            if !single {
                offset += 0.5 * canvas.style().dimensions().dt;
            }
            let offset = self.class.maybe_flip(offset);
            path.apply_offset(offset, canvas);
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
            canvas.stroke();
            canvas.set_line_cap(cairo::LineCap::Butt);
        }
    }

    fn render_full_base(
        &self,
        offset: Option<f64>,
        canvas: &Canvas, path: &Path
    ) {
        if let Some(offset) = offset {
            path.apply_offset(offset, canvas);
        }
        else {
            path.apply(canvas);
        }
        canvas.set_line_width(
            if self.class.class.category().is_main() {
                canvas.style().dimensions().line_width
            }
            else {
                canvas.style().dimensions().other_width
            }
        );
        canvas.style().track_color(&self.class.class).apply(canvas);
        if self.class.combined {
            let seg = canvas.style().dimensions().seg;
            canvas.set_dash(&[0.5 * seg, 0.5 * seg], 0.25 * seg);
        }
        else if self.class.class.status().is_project() {
            let seg = canvas.style().dimensions().seg;
            canvas.set_dash(&[0.7 * seg, 0.3 * seg], 0.7 * seg);
        }
        else {
            canvas.set_dash(&[], 0.);
        }
        if let Some(offset) = offset {
            path.apply_offset(offset, canvas);
        }
        else {
            path.apply(canvas);
        }
        canvas.stroke();
    }
}


//------------ TrackCasing ---------------------------------------------------

/// The markings attached to a track.
pub struct TrackCasing {
    class: TrackClass,
}

impl TrackCasing {
    pub fn new(class: TrackClass) -> Self {
        TrackCasing { class }
    }
}

impl RenderContour for TrackCasing {
    fn render(&self, canvas: &Canvas, path: &Path) {
        canvas.set_source_rgba(1., 1., 1., 0.7);
        canvas.set_line_width(self.line_width(canvas));
        path.apply(canvas);
        canvas.stroke();
    }
}

impl TrackCasing {
    fn line_width(&self, canvas: &Canvas) -> f64 {
        if self.class.double {
            2.2 * canvas.style().dimensions().dt
        }
        else {
            1.2 * canvas.style().dimensions().dt
        }
    }
}

