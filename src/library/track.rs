//! Rendering of track.
//!
//! This is kind of the heart of the whole enterprise, which is why it is a
//! little complex and has its own module.
//!
//! How a track is rendered is selected via the symbol set passed to the
//! track procedure. The following symbols currently have meaning:
//!
//! *  `:double`: renders doubled track. The two tracks will be 1dt apart.
//!    Electrification markings are applied between the tracks. Category 
//!    markings are applied on the right-hand side of the right track
//!    unless `:flip` is provided.
//!
//! *  `:closed`: The track is not in use anymore but is still present.
//!
//! *  `:removed`: The track has been removed. If both `:closed` and
//!    `:removed` are given, the track is considered removed.
//!
//! *  `:gone`: The track has been removed a long time ago.
//!
//! *  `:project`: The track is planned or under construction. When combined
//!    with `closed`, `removed`, or `gone`, the project was abandonned at some
//!    point in the past.
//!
//! *  `:first`, `:second`, `:third`: Marks the track as one of a railway line
//!    of the given classification. The particular meaning of these classes
//!    depends on the jurisdiction. In general, `:first` is for main lines,
//!    `:second` for branch lines. `:third` is used in some jurisdictions for
//!    railway lines with simplified rules.
//!
//! *  `:tram`: Marks the track as tram track.
//!
//! *  `:private`: An track for a railway that is not concessioned for public
//!    use. Typically industrial railways or industrial spurs.
//!
//! *  `:station`: Track that is part of a station. If no classification is
//!    given, `:station` is default.
//!
//! *  `:none`: Don’t render the track itself, only the electrification and
//!    category markings.
//!
//! *  `:narrow`, `:narrower`, `:minimum`, `standard`, `:broad`: The gauge of
//!    the track. Track marked with `:narrow` is between 800 mm and standard
//!    gauge, `:narrower` is between 500 and 800 mm, `:mininum` below that,
//!    `:standard` is for standard gaube, and `:broad` above standard
//!    gauge. The dominant gauge of a country is not marked and carries no
//!    gauge symbol.
//!
//! *  `:cat`, `:excat`: The track is or was electrified with a catenary
//!    system, respectively.
//!
//! *  `:rail`, `:exrail`: The track is or was electrified with a third rail
//!    system, respectively.
//!
//! *  `:flip`: Flips the sides electrification and category markings are
//!    applied to. Normally, category markings are on the right-hand side,
//!    electrification markings are on the left-hand side.
//!
//! *  `:tight`: The track runs in close proximity to other features. This
//!    causes electrification and category markings to be of reduced size.
//!
use crate::canvas::Canvas;
use crate::import::eval::SymbolSet;
use crate::features::contour::RenderContour;
use crate::features::path::Path;
use super::class::Class;
use super::colors::{Style};


//------------ Units ---------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct Units {
    /// The width of a line tracks.
    line_width: f64,

    /// The width of a station, private, or tram track.
    other_width: f64,

    guide_width: f64,

    /// The length of a segment of markings.
    seg: f64,

    /// The distance between two parallel tracks.
    dt: f64,

    /// The height of category markings.
    mark: f64,

    /// The height of tight single-track markings.
    tight_mark: f64,
}

impl Units {
    fn new(canvas: &Canvas) -> Self {
        Units {
            line_width: if canvas.detail() <= 2 {
                1.0 * canvas.canvas_bp()
            } else {
                0.8 * canvas.canvas_bp()
            },
            other_width:    0.5 * canvas.canvas_bp(),
            guide_width:    0.3 * canvas.canvas_bp(),
            seg:            5.0 * super::units::DT * canvas.canvas_bp(),
            dt:             super::units::DT * canvas.canvas_bp(),
            mark: if canvas.detail() < 4 {
                0.6 * super::units::DT * canvas.canvas_bp()
            }
            else {
                0.8 * super::units::DT * canvas.canvas_bp()
            },
            tight_mark:     0.4 * super::units::DT * canvas.canvas_bp(),
        }
    }
}


//------------ TrackContour --------------------------------------------------

/// The rendering rule for a track contour.
pub struct TrackContour {
    /// Is this actually casing?
    casing: bool,

    /// Is this double tracks?
    double: bool,

    /// The feature class.
    class: Class,

    /// The style to use for markings.
    style: &'static Style,

    /// The category of the track.
    category: Category,

    /// Is this station track?
    station: bool,

    /// Is this a project?
    project: bool,

    /// Should this track be combined with the underlying track?
    combined: bool,

    /// The status of catenary electrification.
    cat: Status,

    /// The status of third rail electrification.
    rail: Status,

    /// The gauge of track.
    gauge: Gauge,

    /// Are markings flipped?
    flip: bool,

    /// Should markings be smaller?
    tight: bool,
}

impl TrackContour {
    pub fn new(
        style: &'static Style, casing: bool, symbols: &SymbolSet
    ) -> Self {
        TrackContour {
            casing,
            double: symbols.contains("double"),
            class: Class::from_symbols(&symbols),
            style,
            category: Category::from_symbols(&symbols),
            station: symbols.contains("station") || symbols.contains("guide"),
            project: symbols.contains("project"),
            combined: symbols.contains("combined"),
            cat: if symbols.contains("cat") { Status::Active }
                 else if symbols.contains("excat") { Status:: Ex }
                 else { Status::Never },
            rail: if symbols.contains("rail") { Status::Active }
                  else if symbols.contains("exrail") { Status:: Ex }
                  else { Status::Never },
            gauge: if symbols.contains("minimum") { Gauge::Minimum }
                   else if symbols.contains("narrower") { Gauge::Narrower }
                   else if symbols.contains("narrow") { Gauge::Narrow }
                   else if symbols.contains("broad") { Gauge::Broad }
                   else { Gauge::Standard },
            flip: symbols.contains("flip"),
            tight: symbols.contains("tight"),
        }
    }
}

impl RenderContour for TrackContour {
    fn render(&self, canvas: &Canvas, path: &Path) {
        let units = Units::new(canvas);
        if self.casing {
            self.render_casing(canvas, units, path);
        }
        else if canvas.detail() == 0 {
            self.render_detail_0(canvas, units, path);
        }
        else if canvas.detail() == 1 {
            self.render_detail_1(canvas, units, path);
        }
        else if canvas.detail() == 2 {
            self.render_detail_2(canvas, units, path);
        }
        else {
            self.render_detail_full(canvas, units, path);
        }
    }
}

impl TrackContour {
    fn render_casing(&self, canvas: &Canvas, units: Units, path: &Path) {
        //canvas.set_operator(cairo::Operator::Clear);
        canvas.set_source_rgba(1., 1., 1., 0.7);
        if self.double {
            canvas.set_line_width(2.2 * units.dt);
        }
        else {
            canvas.set_line_width(1.2 * units.dt);
        }
        path.apply(canvas);
        canvas.stroke();
        //canvas.set_operator(cairo::Operator::Over);
    }

    fn render_detail_1(&self, canvas: &Canvas, units: Units, path: &Path) {
        if self.style.name == "red" {
            self.render_glow(canvas, 2.5 * units.other_width, path);
        }
        if self.double {
            canvas.set_line_width(units.line_width * 1.2);
        }
        else {
            canvas.set_line_width(units.line_width * 0.7);
        }
        self.class.standard_color().apply(canvas);
        path.apply(canvas);
        canvas.stroke();
    }

    fn render_detail_0(&self, canvas: &Canvas, units: Units, path: &Path) {
        if self.style.name == "red" {
            self.render_glow(canvas, 2.5 * units.other_width, path);
        }
        canvas.set_line_width(units.line_width * 0.7);
        self.class.standard_color().apply(canvas);
        path.apply(canvas);
        canvas.stroke();
    }

    fn render_detail_2(&self, canvas: &Canvas, units: Units, path: &Path) {
        if self.style.name == "red" {
            self.render_glow(canvas, 2.5 * units.other_width, path);
        }
        if self.category.is_main_line() {
            if self.double {
                canvas.set_line_width(2.0 * units.line_width);
            }
            else {
                canvas.set_line_width(units.line_width);
            }
        }
        else {
            canvas.set_line_width(units.other_width);
        }
        if self.combined {
            canvas.set_dash(
                &[0.5 * units.seg, 0.5 * units.seg],
                0.25 * units.seg
            );
        }
        else if self.project {
            canvas.set_dash(
                &[0.7 * units.seg, 0.3 * units.seg],
                0.15 * units.seg
            );
        }
        self.class.standard_color().apply(canvas);
        path.apply(canvas);
        canvas.stroke();
        canvas.set_dash(&[], 0.);
    }

    fn render_detail_full(&self, canvas: &Canvas, units: Units, path: &Path) {
        if self.double {
            self.render_double(canvas, units, path);
        }
        else {
            self.render_single(canvas, units, path);
        }
    }

    fn render_single(&self, canvas: &Canvas, units: Units, path: &Path) {
        // Category and electrification markings
        // 
        // These go first so they get overpainted with the line.
        if !self.station && (self.has_category(canvas) || self.has_electric()) {
            if self.flip {
                path.apply_offset(-0.5 * self.mark(units), canvas);
            }
            else {
                path.apply_offset(0.5 * self.mark(units), canvas);
            }

            canvas.set_line_width(self.mark(units));
            if self.has_category(canvas) {
                self.apply_category_properties(canvas, units);
                if self.has_electric() {
                    canvas.stroke_preserve()
                }
                else {
                    canvas.stroke()
                }
            }

            if self.has_electric() {
                self.apply_electric_properties(canvas, units);
                canvas.stroke()
            }
        }

        // Base track
        if self.category.has_base() {
            self.apply_base_properties(canvas, units);
            path.apply(canvas);
            if self.combined {
                canvas.set_dash(
                    &[0.5 * units.seg, 0.5 * units.seg],
                    0.25 * units.seg
                );
                canvas.stroke();
                canvas.set_dash(&[], 0.);
            }
            else {
                canvas.stroke();
            }
        }


        // Project.
        if !self.station && self.project {
            if self.flip {
                path.apply_offset(-0.5 * self.mark(units), canvas);
            }
            else {
                path.apply_offset(0.5 * self.mark(units), canvas);
            }
            canvas.set_dash(
                &[0.3 * units.seg, 0.7 * units.seg],
                1.15 * units.seg
            );
            canvas.set_line_width(
                self.mark(units) + 2. * self.line_width(units)
            );
            canvas.set_operator(cairo::Operator::Clear);
            canvas.stroke();
            canvas.set_operator(cairo::Operator::Over);
        }

        /*
        // Debug helper: marks out segments in red.
        if self.flip {
            path.apply_offset(-0.5 * self.mark(units), canvas);
        }
        else {
            path.apply_offset(0.5 * self.mark(units), canvas);
        }
        canvas.set_line_width(1.5 * self.line_width(units));
        crate::features::color::Color::RED.apply(canvas);
        canvas.set_dash(
            &[0.04 * units.seg, 0.96 * units.seg],
            0.0
        );
        canvas.stroke();
        */
     }

    fn render_double(&self, canvas: &Canvas, units: Units, path: &Path) {
        // Category and electrification markings
        // 
        // These go first so they get overpainted with the line.
        if !self.station && (self.has_category(canvas) || self.has_electric()) {
            if self.has_category(canvas) {
                if self.flip {
                    path.apply_offset(
                        -0.5 * self.mark(units) - 0.5 * units.dt,
                        canvas
                    );
                }
                else {
                    path.apply_offset(
                        0.5 * self.mark(units) + 0.5 * units.dt,
                        canvas
                    );
                }
                canvas.set_line_width(self.mark(units));
                self.apply_category_properties(canvas, units);
                canvas.stroke()
            }

            if self.has_electric() {
                self.apply_electric_properties(canvas, units);
                path.apply(canvas);
                canvas.set_line_width(units.dt);
                canvas.stroke()
            }
        }

        // Base tracks
        if self.category.has_base() {
            self.apply_base_properties(canvas, units);
            if self.combined {
                canvas.set_dash(
                    &[0.5 * units.seg, 0.5 * units.seg],
                    0.25 * units.seg
                );
            }
            else if self.project {
                canvas.set_dash(
                    &[0.7 * units.seg, 0.3 * units.seg],
                    0.7 * units.seg
                );
            }
            path.apply_offset(0.5 * units.dt, canvas);
            path.apply_offset(-0.5 * units.dt, canvas);
            canvas.stroke();
        }
    }

    fn render_glow(&self, canvas: &Canvas, width: f64, path: &Path) {
        canvas.set_line_cap(cairo::LineCap::Round);
        canvas.set_line_width(3.8 * width);
        self.class.standard_color().apply(canvas);
        path.apply(canvas);
        canvas.stroke();
        canvas.set_line_cap(cairo::LineCap::Butt);
    }
}

/// # Construct Track Markings
///
/// We currently use Cairo’s dash pattern feature to construct the markings.
/// Up to three dashed lines are drawn off to one side of the actual line:
/// the electrification marking, the category marking, and the project
/// marking. They have to be drawn individually because their colour and
/// drawing operator may differ. Specifically, electrification could have been
/// removed on an active line and the project marking needs to use the clear
/// operator.
///
/// Each pattern repeats after `Units::seg` which we shall refer to as a
/// *seg* from now on. The various markings need to be arranged evenly within
/// a *seg* which means the dash pattern of each marking depends on the
/// presence of the other markings.
///
/// The project marking removes 0.15 *seg* at the beginning and end of each
/// *seg.* In other words, it only leaves 0.7 *seg* at the centre of a
/// *seg* to apply the other markings to.
///
/// The electrification marking is 0.3 *seg* long. For sole overhead line
/// electrification, it is fully filled. For sole third rail electrification,
/// it has a gap 0.05 *seg* wide ending 0.05 *seg* before the end of the
/// marking. If there are both overhead and third rail electrification, the
/// marking has an equal gap at its start, too.
///
/// The category marking consists of up to four strokes one line width wide
/// and one line width apart. One stroke is for secondary standard gauge
/// railways, two strokes is for tertiary standard gauge railways, three is
/// for narrow gauge railways and four strokes for even more narrow gauge
/// railways.
///
/// If there is only electrification marking or only category marking present,
/// the marking is applied in the centre of each *seg.*
///
/// If both these markings are present but there is no project marking, the
/// category marking is centered around a point 0.25 *seg* into each *seg*
/// and the electrification marking starts 0.6 *seg* into each seg.
///
/// If all three markings are present, the category marking is centered around
/// a point 0.325 *seg* into each *seg* and the electrification marking starts
/// 0.5 *seg* into each *seg.* These values may have to be shifted a bit to
/// make it all look good.
impl TrackContour {
    fn apply_base_properties(&self, canvas: &Canvas, units: Units) {
        canvas.set_dash(&[], 0.);
        canvas.set_line_width(self.line_width(units));
        /*
        if self.project {
            // Projected lines have an 0.7seg on/0.3 seg off pattern
            canvas.set_dash(
                &[0.7 * units.seg, 0.3 * units.seg],
                0.85 * units.seg
            );
        }
        */
        self.class.standard_color().apply(canvas);
    }

    /// Configures the canvas for drawing 

    /// Configures the canvas for drawing the category markings.
    ///
    /// Returns whether category markings need to be draw at all.
    fn apply_category_properties(
        &self, canvas: &Canvas, units: Units
    ) {
        let strokes = self.category_strokes(canvas);
        let seg = units.seg;
        let w = units.line_width;
        let center = match (self.project, self.has_electric()) {
            (_, false) => 0.5 * seg,
            (false, true) => 0.25 * seg,
            (true, true) => 0.325 * seg
        };
        match strokes {
            1 => {
                canvas.set_dash(
                    &[w, seg - w],
                    (seg - center) + 0.5 * w
                );
            }
            2 => {
                canvas.set_dash(
                    &[w, w, w, seg - 3. * w],
                    (seg - center) + 1.5 * w
                );
            }
            3 => {
                canvas.set_dash(
                    &[w, w, w, w, w, seg - 5. * w],
                    (seg - center) + 2.5 * w
                );
            }
            4 => {
                canvas.set_dash(
                    &[w, w, w, w, w, w, w, seg - 7. * w],
                    (seg - center) + 3.5 * w
                );
            }
            _ => unreachable!()
        }
        self.class.standard_color().apply(canvas);
    }

    /// Configures the canvas for drawing the electrification markings.
    fn apply_electric_properties(&self, canvas: &Canvas, units: Units) {
        use Status::*;

        let seg = units.seg;
        let start = match (self.project, self.has_category(canvas)) {
            (_, false) => 0.65 * seg,
            (false, true) => 0.4 * seg,
            (true, true) => 0.5 * seg
        };

        match (self.cat, self.rail) {
            (Active, Never) | (Active, Ex) => {
                canvas.set_dash(
                    &[0.3 * seg, 0.7 * seg],
                    start
                );
                self.class.standard_color().apply(canvas);
            }
            (Never, Active) | (Ex, Active) => {
                canvas.set_dash(
                    &[0.05 * seg, 0.05 * seg, 0.2 * seg, 0.7 * seg],
                    start
                );
                self.class.standard_color().apply(canvas);
            }
            (Active, Active) => {
                canvas.set_dash(
                    &[0.05 * seg, 0.05 * seg, 0.1 * seg,
                      0.05 * seg, 0.05 * seg,  0.7 * seg],
                    start
                );
                self.class.standard_color().apply(canvas);
            }
            (Ex, Never) => {
                canvas.set_dash(
                    &[0.3 * seg, 0.7 * seg],
                    start
                );
                self.class.removed_color().apply(canvas);
            }
            (Never, Ex) => {
                canvas.set_dash(
                    &[0.05 * seg, 0.05 * seg, 0.2 * seg, 0.7 * seg],
                    start
                );
                self.class.removed_color().apply(canvas);
            }
            (Ex, Ex) => {
                canvas.set_dash(
                    &[0.05 * seg, 0.05 * seg, 0.1 * seg,
                      0.05 * seg, 0.05 * seg,  0.7 * seg],
                    start
                );
                self.class.removed_color().apply(canvas);
            }
            (Never, Never) => unreachable!()
        }
    }


    fn line_width(&self, units: Units) -> f64 {
        if self.category.is_main_line() { units.line_width }
        else if self.category.is_guide() { units.guide_width }
        else { units.other_width }
    }

    fn has_electric(&self) -> bool {
        self.cat.present() || self.rail.present()
    }

    fn has_category(&self, canvas: &Canvas) -> bool {
        self.category_strokes(canvas) != 0
    }

    fn category_strokes(&self, canvas: &Canvas) -> usize {
        if canvas.detail() >= 4 {
            self.category_strokes_full()
        }
        else {
            self.category_strokes_light()
        }
    }

    fn category_strokes_full(&self) -> usize {
        match (self.category, self.gauge) {
            (_, Gauge::Narrower) => 2,
            (_, Gauge::Narrow) => 2,
            (Category::Third, Gauge::Standard) => 0,
            (Category::Second, Gauge::Standard) => 1,
            _ => 0
        }
    }

    fn category_strokes_light(&self) -> usize {
        match self.category {
            Category::Second => 1,
            _ => 0
        }
    }

    fn mark(&self, units: Units) -> f64 {
        if self.tight {
            units.tight_mark
        }
        else {
            units.mark
        }
    }
}


//------------ TrackShade ---------------------------------------------------

/// The rendering rule for track shading.
pub struct TrackShading {
    /// The feature class.
    class: Class,

    /// Is this double tracks?
    double: bool,
}

impl TrackShading {
    pub fn new(
        symbols: &SymbolSet
    ) -> Self {
        TrackShading {
            class: Class::from_symbols(&symbols),
            double: symbols.contains("double"),
        }
    }
}

impl RenderContour for TrackShading {
    fn render(&self, canvas: &Canvas, path: &Path) {
        let units = Units::new(canvas);
        if self.double {
            canvas.set_line_width(2.6 * units.dt);
        }
        else {
            canvas.set_line_width(1.6 * units.dt);
        }
        self.class.shade_color().lighten(0.2).apply(canvas);
        path.apply(canvas);
        canvas.stroke();
    }
}
        

//------------ Category -----------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum Category {
    None,
    First,
    Second,
    Third,
    Tram,
    Private,
    Station,
    Guide,
}

impl Category {
    fn from_symbols(symbols: &SymbolSet) -> Self {
        if symbols.contains("none") { Category::None }
        else if symbols.contains("first") { Category::First }
        else if symbols.contains("second") { Category::Second }
        else if symbols.contains("third") { Category::Third }
        else if symbols.contains("tram") { Category::Tram }
        else if symbols.contains("private") { Category::Private }
        else if symbols.contains("guide") { Category::Guide }
        else { Category::Station }
    }

    /*
    fn is_line(self) -> bool {
        match self {
            Category::First | Category::Second | Category::Third => true,
            _ => false
        }
    }
    */

    fn is_main_line(self) -> bool {
        match self {
            Category::First | Category::Second => true,
            _ => false
        }
    }

    fn is_guide(self) -> bool {
        match self {
            Category::Guide => true,
            _ => false
        }
    }

    fn has_base(self) -> bool {
        match self {
            Category::None => false,
            _ => true
        }
    }
}


//------------ Status --------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum Status {
    Active, 
    Ex,
    Never,
}

impl Status {
    fn present(self) -> bool {
        match self {
            Status::Active => true,
            Status::Ex => true,
            Status::Never => false,
        }
    }

    /*
    fn is_ex(self) -> bool {
        match self {
            Status::Ex => true,
            _ => false,
        }
    }
    */
}


//------------ Gauge ---------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum Gauge {
    Minimum,
    Narrower,
    Narrow,
    Standard,
    Broad,
}


