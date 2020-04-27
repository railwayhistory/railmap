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
//! *  `:flip`: Flips the sides electrification and category markings are
//!    applied to. Normally, category markings are on the right-hand side,
//!    electrification markings are on the left-hand side.
//!
//! *  `:closed`: The track is not in use anymore but is still present.
//!
//! *  `:removed`: The track has been removed. If both `:closed` and
//!    `:removed` are given, the track is considered removed.
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
use crate::canvas::Canvas;
use crate::import::eval::SymbolSet;
use crate::features::contour::RenderContour;
use crate::features::path::Path;
use super::colors::Palette;


//------------ Units ---------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct Units {
    /// The width of a line tracks.
    line_width: f64,

    /// The width of a station, private, or tram track.
    other_width: f64,

    /// The length of a segment of markings.
    seg: f64,

    /// The distance between two parallel tracks.
    dt: f64,

    /// The height of single-track electrification markings.
    elmark: f64,

    /// The height of category markings.
    mark: f64,
}

impl Units {
    fn new(canvas: &Canvas) -> Self {
        Units {
            line_width:     0.6 * canvas.canvas_bp(),
            other_width:    0.4 * canvas.canvas_bp(),
            seg:            5.0 * super::units::DT * canvas.canvas_bp(),
            dt:             super::units::DT * canvas.canvas_bp(),
            elmark:         0.5 * super::units::DT * canvas.canvas_bp(),
            mark:           0.5 * super::units::DT * canvas.canvas_bp(),
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

    /// Are markings flipped?
    flip: bool,

    /// The palette to use for rendering
    palette: Palette,

    /// The category of the track.
    category: Category,

    /// Is this station track?
    station: bool,

    /// The status of catenary electrification.
    cat: Status,

    /// The status of third rail electrification.
    rail: Status,
}

impl TrackContour {
    pub fn new(casing: bool, symbols: SymbolSet) -> Self {
        TrackContour {
            casing,
            double: symbols.contains("double"),
            flip: symbols.contains("flip"),
            palette: Palette::from_symbols(&symbols),
            category: Category::from_symbols(&symbols),
            station: symbols.contains("station"),
            cat: if symbols.contains("cat") { Status::Active }
                 else if symbols.contains("excat") { Status:: Ex }
                 else { Status::Never },
            rail: if symbols.contains("rail") { Status::Active }
                  else if symbols.contains("exrail") { Status:: Ex }
                  else { Status::Never },
        }
    }
}

impl RenderContour for TrackContour {
    fn render(&self, canvas: &Canvas, path: &Path) {
        let units = Units::new(canvas);
        if self.casing {
            self.render_casing(canvas, units, path);
        }
        else if self.double {
            self.render_double(canvas, units, path);
        }
        else {
            self.render_single(canvas, units, path);
        }
    }
}

impl TrackContour {
    fn render_casing(&self, canvas: &Canvas, units: Units, path: &Path) {
        canvas.set_operator(cairo::Operator::Clear);
        if self.double {
            canvas.set_line_width(2.4 * units.dt);
        }
        else {
            canvas.set_line_width(1.4 * units.dt);
        }
        path.apply(canvas);
        canvas.stroke();
        canvas.set_operator(cairo::Operator::Over);
    }

    fn render_single(&self, canvas: &Canvas, units: Units, path: &Path) {
        // Catenary Electrification
        if self.cat.present() {
            self.apply_cat_properties(canvas, units);
            canvas.set_line_width(units.elmark);
            if self.flip {
                path.apply_offset(-0.5 * units.elmark, canvas);
            }
            else {
                path.apply_offset(0.5 * units.elmark, canvas);
            }
            canvas.stroke();
        }

        // Third-rail Electrification
        if self.rail.present() {
            self.apply_rail_properties(canvas, units);
            canvas.set_line_width(units.elmark);
            if self.flip {
                path.apply_offset(-0.5 * units.elmark, canvas);
            }
            else {
                path.apply_offset(0.5 * units.elmark, canvas);
            }
            canvas.stroke();
        }

        // Classification
        if !self.station && self.category.has_class() {
            self.apply_class_properties(canvas, units);
            canvas.set_line_width(units.mark);
            if self.flip {
                path.apply_offset(0.5 * units.mark, canvas);
            }
            else {
                path.apply_offset(-0.5 * units.mark, canvas);
            }
            canvas.stroke();
        }

        // Base track
        if self.category.has_base() {
            self.apply_base_properties(canvas, units);
            path.apply(canvas);
            canvas.stroke();
        }
     }

    fn render_double(&self, canvas: &Canvas, units: Units, path: &Path) {
        // Catenary electrification
        if self.cat.present() {
            self.apply_cat_properties(canvas, units);
            canvas.set_line_width(units.dt);
            path.apply(canvas);
            canvas.stroke();
        }

        // Third-rail electrification
        if self.rail.present() {
            self.apply_rail_properties(canvas, units);
            canvas.set_line_width(units.dt);
            path.apply(canvas);
            canvas.stroke();
        }

        // Classification
        if !self.station && self.category.has_class() {
            self.apply_class_properties(canvas, units);
            canvas.set_line_width(units.mark);
            if self.flip {
                path.apply_offset(0.5 * (units.mark + units.dt), canvas);
            }
            else {
                path.apply_offset(-0.5 * (units.mark + units.dt), canvas);
            }
            canvas.stroke();
        }

        // Base tracks
        if self.category.has_base() {
            self.apply_base_properties(canvas, units);
            path.apply_offset(0.5 * units.dt, canvas);
            path.apply_offset(-0.5 * units.dt, canvas);
            canvas.stroke();
        }
    }
}

impl TrackContour {
    fn apply_base_properties(&self, canvas: &Canvas, units: Units) {
        canvas.set_dash(&[], 0.);
        canvas.set_line_width(
            if self.category.is_line() { units.line_width }
            else { units.other_width }
        );
        self.palette.stroke.apply(canvas);
    }

    fn apply_cat_properties(&self, canvas: &Canvas, units: Units) {
        if self.rail.present() {
            canvas.set_dash(
                &[units.seg / 4., 7. * units.seg / 4.],
                5. * units.seg / 8.
            );
        }
        else {
            canvas.set_dash(
                &[units.seg / 4., 3. * units.seg / 4.],
                5. * units.seg / 8.
            );
        }
        if self.cat.is_ex() {
            Palette::REMOVED.fill.apply(canvas)
        }
        else {
            self.palette.fill.apply(canvas)
        }
    }

    fn apply_rail_properties(&self, canvas: &Canvas, units: Units) {
        if self.cat.present() {
            canvas.set_dash(
                &[
                    3. * units.seg / 32., 2. * units.seg / 32.,
                    3. * units.seg / 32., 7. * units.seg / 4.
                 ],
                13. * units.seg / 8.
            );
        }
        else {
            canvas.set_dash(
                &[
                    3. * units.seg / 32., 2. * units.seg / 32.,
                    3. * units.seg / 32., 3. * units.seg / 4.
                 ],
                5. * units.seg / 8.
            );
        }
        if self.rail.is_ex() {
            Palette::REMOVED.fill.apply(canvas)
        }
        else {
            self.palette.fill.apply(canvas)
        }
    }

    fn apply_class_properties(&self, canvas: &Canvas, units: Units) {
        match self.category {
            Category::Second => {
                canvas.set_dash(
                    &[ units.line_width, units.seg - units.line_width ],
                    0.
                )
            }
            Category::Third => {
                canvas.set_dash(
                    &[
                        units.line_width,
                        units.line_width,
                        units.line_width,
                        units.seg - 3. * units.line_width
                    ],
                    1.5 * units.line_width
                )
            }
            _ => unreachable!()
        }
        self.palette.stroke.apply(canvas);
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
}

impl Category {
    fn from_symbols(symbols: &SymbolSet) -> Self {
        if symbols.contains("none") { Category::None }
        else if symbols.contains("first") { Category::First }
        else if symbols.contains("second") { Category::Second }
        else if symbols.contains("third") { Category::Third }
        else if symbols.contains("tram") { Category::Tram }
        else if symbols.contains("private") { Category::Private }
        else { Category::Station }
    }

    fn is_line(self) -> bool {
        match self {
            Category::First | Category::Second | Category::Third => true,
            _ => false
        }
    }

    fn has_base(self) -> bool {
        match self {
            Category::None => false,
            _ => true
        }
    }

    fn has_class(self) -> bool {
        match self {
            Category::Second => true,
            Category::Third => true,
            _ => false,
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

    fn is_ex(self) -> bool {
        match self {
            Status::Ex => true,
            _ => false,
        }
    }
}

