
use std::f64::consts::PI;
use femtomap::import::ast::ShortString;
use femtomap::import::eval::{EvalErrors, Failed, SymbolSet};
use femtomap::path::Position;
use femtomap::render::{Canvas, Color, Group, LineCap, Matrix};
use femtomap::world::Rect;
use kurbo::Point;
use crate::railway::class::Railway;
use crate::railway::import::eval::Scope;
use crate::railway::measures::Measures;
use crate::railway::style::Style;
use super::{AnyFeature, AnyShape, Category, Feature, Shape, Stage, StageSet};


//------------ from_args -----------------------------------------------------

pub fn from_args(
    symbols: SymbolSet,
    position: Position,
    extent: Option<Position>,
    scope: &Scope,
    err: &mut EvalErrors,
) -> Result<AnyFeature, Failed> {
    Marker::from_args(symbols, position, extent, scope, err)
}


//------------ Marker --------------------------------------------------------

pub struct Marker {
    /// The position the marker is attached to.
    position: Position,

    /// The position of the extent of the marker’s validity.
    extent: Option<Position>,

    /// Orientation of the marker.
    ///
    /// If this in `None` the marker doesn’t need to be oriented at all.
    /// Otherwise the value is the angle to be added to rotation from the
    /// position.
    orientation: f64,

    /// Are we drawing casing?
    casing: bool,

    /// The feature class.
    class: Railway,

    /// The marker
    marker: &'static dyn RenderMarker,
}

impl Marker {
    pub fn from_args(
        mut symbols: SymbolSet,
        position: Position,
        extent: Option<Position>,
        scope: &Scope,
        err: &mut EvalErrors,
    ) -> Result<AnyFeature, Failed> {
        let orientation = Self::rotation_from_symbols(&mut symbols, err)?;
        let class = Railway::from_symbols(&mut symbols, scope);
        let casing = symbols.take("casing");
        let pos = symbols.pos();
        let marker = match symbols.take_final(err)? {
            Some(marker) => marker,
            None => {
                err.add(pos, "missing marker");
                return Err(Failed)
            }
        };

        // We only need a d3 marker.
        match Self::find_marker(&marker, MARKERS) {
            Some(marker) => {
                return Ok(Marker {
                    position, orientation, extent, casing, class,
                    marker,
                }.into())
            }
            None => {
                /*
                err.add(pos, "unknown marker");
                return Err(Failed)
                */
            }
        }

        // Didn’t find anything. Try the old marker for now.
        super::oldmarker::StandardMarker::new(
            position, orientation, class, marker, pos, err
        )
    }

    fn rotation_from_symbols(
        symbols: &mut SymbolSet,
        _err: &mut EvalErrors
    ) -> Result<f64, Failed> {
        if symbols.take("top") {
            Ok(1.5 * PI)
        }
        else if symbols.take("left") {
            Ok(PI)
        }
        else if symbols.take("bottom") {
            Ok(0.5 * PI)
        }
        else if symbols.take("right") {
            Ok(0.)
        }
        else {
            Ok(0.)
            /*
            err.add(pos, "missing orientation");
            Err(Failed)
                */
        }
    }

    fn find_marker(
        marker: &ShortString,
        collection: &[(&str, &'static dyn RenderMarker)]
    ) -> Option<&'static dyn RenderMarker> {
        collection.iter().find_map(|(name, fun)| {
            (*name == marker.as_str()).then_some(*fun)
        })
    }

    pub fn class(&self) -> &Railway {
        &self.class
    }
}

impl Feature for Marker {
    fn storage_bounds(&self) -> Rect {
        self.position.storage_bounds()
    }

    fn group(&self) -> super::Group {
        super::Group::with_railway(Category::Marker, &self.class)
    }

    fn shape(
        &self, style: &Style, _canvas: &Canvas
    ) -> AnyShape<'_> {
        MarkerShape {
            marker: self,
            info: RenderInfo::from_style(style, &self.class),
        }.into()
    }
}


//------------ MarkerShape ---------------------------------------------------

struct MarkerShape<'a> {
    marker: &'a Marker,
    info: RenderInfo,
}

impl MarkerShape<'_> {
    fn prepare_canvas<'c>(
        &self, style: &Style, canvas: &'c mut Canvas
    ) -> (Group<'c>, Option<Point>) {
        let mut canvas = canvas.sketch().into_group();
        let (point, angle) = self.marker.position.resolve(style);
        let matrix = Matrix::identity().translate(
            point
        ).rotate(angle + self.marker.orientation);
        let extent = self.marker.extent.as_ref().map(|extent| {
            let (extent, _) = extent.resolve(style);
            matrix.clone().invert().transform_point(extent)
        });

        canvas.apply(matrix);
        (canvas, extent)
    }
}

impl<'a> Shape<'a> for MarkerShape<'a> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        match stage {
            Stage::Casing
                if self.marker.casing && self.marker.extent.is_some() =>
            {
                let (mut canvas, extent) = self.prepare_canvas(
                    style, canvas
                );
                self.marker.marker.track_casing(
                    &self.info, extent, &mut canvas
                );
            }
            Stage::MarkerCasing if self.marker.casing => {
                let (mut canvas, extent) = self.prepare_canvas(
                    style, canvas
                );
                self.marker.marker.casing(&self.info, extent, &mut canvas);
            }
            Stage::MarkerBase => {
                let (mut canvas, extent) = self.prepare_canvas(
                    style, canvas
                );
                self.marker.marker.base(&self.info, extent, &mut canvas);
            }
            _ => { }
        }
    }

    fn stages(&self) -> StageSet {
        let mut res = StageSet::empty();
        res = res.add(Stage::MarkerBase);
        if self.marker.casing {
            res = res.add(Stage::MarkerCasing);
            if self.marker.extent.is_some() {
                res = res.add(Stage::Casing);
            }
        }
        res
    }
}



//------------ RenderInfo ----------------------------------------------------

/// Information we need to render a shaped marker.
#[allow(dead_code)]
struct RenderInfo {
    /// The detail level.
    detail: u8,

    /// The measures according to the style.
    m: Measures,

    /// The track width according to the class.
    ct: f64,

    /// Double track width according to the class.
    cd: f64,

    /// Space between two tracks according to the class.
    cs: f64,

    /// The foreground color for the marker.
    color: Color,

    /// The fill color for the marker.
    empty_color: Color,

    /// The casing color for the marker.
    casing_color: Color,
}

impl RenderInfo {
    fn from_style(style: &Style, class: &Railway) -> Self {
        RenderInfo {
            detail: style.detail(),
            m: style.measures(),
            ct: style.measures().class_track(class),
            cd: style.measures().class_double(class),
            cs: style.measures().class_skip(class),
            color: style.primary_marker_color(class),
            empty_color: Color::WHITE,
            casing_color: style.casing_color(),
        }
    }
}


//------------ RenderMarker --------------------------------------------------

trait RenderMarker: Send + Sync {
    fn base(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        let _ = (info, extent, canvas);
    }

    fn casing(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        let _ = (info, extent, canvas);
    }

    fn track_casing(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        let _ = (info, extent, canvas);
    }
}


//------------ DetailRenderMarker --------------------------------------------

trait DetailRenderMarker: Send + Sync {
    fn d3_base(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        let _ = (info, extent, canvas);
    }

    fn d3_casing(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        let _ = (info, extent, canvas);
    }

    fn d3_track_casing(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        let _ = (info, extent, canvas);
    }

    fn d4_base(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        let _ = (info, extent, canvas);
    }

    fn d4_casing(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        let _ = (info, extent, canvas);
    }

    fn d4_track_casing(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        let _ = (info, extent, canvas);
    }
}

impl<T: DetailRenderMarker> RenderMarker for T {
    fn base(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        if info.detail < 4 {
            self.d3_base(info, extent, canvas)
        }
        else {
            self.d4_base(info, extent, canvas)
        }
    }

    fn casing(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        if info.detail < 4 {
            self.d3_casing(info, extent, canvas)
        }
        else {
            self.d4_casing(info, extent, canvas)
        }
    }

    fn track_casing(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        if info.detail < 4 {
            self.d3_track_casing(info, extent, canvas)
        }
        else {
            self.d4_track_casing(info, extent, canvas)
        }
    }
}


//============ The individual markers ========================================


const MARKERS: &[(&str, &'static dyn RenderMarker)] = &[
    ("bk", &Block),
    ("h", &Halt),
    ("exst", &ExStation),
    ("gh", &GoodsHalt),
    ("gst", &GoodsStation),
    ("inst", &IslandStation),
    ("jn", &Junction),
    ("opbound", &OperatorBoundary),
    ("st", &Station),
    ("sst", &ServiceStation),

    ("de.abzw", &Junction),
    ("de.bbf", &ServiceStation),
    ("de.bf", &Station),
    ("de.bk", &Block),
    ("de.dirgr", &OperatorBoundary),
    ("de.exbf", &ExStation),
    ("de.gbf", &GoodsStation),
    ("de.hp", &Halt),
    ("de.hp.bk", &Halt), // XXX Fix
    ("de.hst", &Hst),
    ("de.inbf", &IslandStation),
    ("de.ldst", &GoodsHalt),
];


//------------ Station -------------------------------------------------------

pub struct Station;

impl RenderMarker for Station {
    fn base(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        station(info, canvas);
        canvas.apply(info.color);
        canvas.fill();
    }

    fn casing(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        station(info, canvas);
        canvas.apply(info.casing_color);
        canvas.apply_line_width(w(info));
        canvas.stroke();
    }
}


//------------ IslandStation -------------------------------------------------

pub struct IslandStation;

impl RenderMarker for IslandStation {
    fn base(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        Self::frame(info, canvas);
        canvas.apply(info.color);
        canvas.fill();
    }

    fn casing(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        Self::frame(info, canvas);
        canvas.apply(info.casing_color);
        canvas.apply_line_width(w(info));
        canvas.stroke();
    }
}

impl IslandStation {
    fn frame(info: &RenderInfo, canvas: &mut Group) {
        canvas.move_to(x0(info), y0(info));
        canvas.line_to(x0(info), y1(info) - y0(info));
        canvas.line_to(x1(info), y1(info) - y0(info));
        canvas.line_to(x1(info), y0(info));
        canvas.close_path();
    }
}


//------------ GoodsStation --------------------------------------------------

pub struct GoodsStation;

impl RenderMarker for GoodsStation {
    fn base(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        Station.base(info, extent, canvas);
        canvas.new_path();
        canvas.move_to(x0(info) + w(info), y0(info) + w(info));
        canvas.line_to(x0(info) + w(info), y1(info) - w(info));
        canvas.line_to(x1(info) - w(info), y1(info) - w(info));
        canvas.close_path();
        canvas.apply(info.empty_color);
        canvas.fill();
    }

    fn casing(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        Halt.casing(info, extent, canvas);
    }
}


//------------ ServiceStation ------------------------------------------------

pub struct ServiceStation;

impl RenderMarker for ServiceStation {
    fn base(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        Station.base(info, extent, canvas);
        canvas.new_path();
        canvas.move_to(Self::xi0(info), Self::yi1(info));
        canvas.line_to(0., Self::yi0(info));
        canvas.line_to(Self::xi1(info), Self::yi1(info));
        canvas.close_path();
        canvas.apply(info.empty_color);
        canvas.fill();
    }

    fn casing(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        Halt.casing(info, extent, canvas);
    }
}

impl ServiceStation {
    fn xi0(info: &RenderInfo) -> f64 {
        -Self::xi1(info)
    }

    fn xi1(info: &RenderInfo) -> f64 {
        if info.detail < 4 {
            x0(info) + 0.75 * w(info)
        }
        else {
            x0(info) + 0.5 * w(info)
        }
    }

    fn yi0(info: &RenderInfo) -> f64 {
        if info.detail < 4 {
            y0(info) + 0.75 * w(info)
        }
        else {
            y0(info) + 0.5 * w(info)
        }
    }

    fn yi1(info: &RenderInfo) -> f64 {
        if info.detail < 4 {
            y1(info) - w(info)
        }
        else {
            y1(info) - 0.75 * w(info)
        }
    }
}


//------------ ExStation -----------------------------------------------------

pub struct ExStation;

impl RenderMarker for ExStation {
    fn base(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        canvas.move_to(x0s(info), Self::y(info));
        canvas.line_to(x1s(info), Self::y(info));
        canvas.apply(info.color);
        canvas.apply_line_width(Self::width(info));
        canvas.apply(LineCap::Round);
        canvas.stroke();
    }

    fn casing(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        canvas.move_to(x0s(info), y1(info) + 1.5 * w(info));
        canvas.line_to(x1s(info), y1(info) + 1.5 * w(info));
        canvas.apply(info.casing_color);
        canvas.apply_line_width(Self::width(info) + w(info));
        canvas.apply(LineCap::Round);
        canvas.stroke();
    }
}

impl ExStation {
    fn y(info: &RenderInfo) -> f64 {
        if info.detail < 4 {
            y1(info) + 1.5 * w(info)
        }
        else {
            y1(info) + 2. * w(info)
        }
    }

    fn width(info: &RenderInfo) -> f64 {
        if info.detail < 4 {
            w(info)
        }
        else {
            1.5 * w(info)
        }
    }
}


//------------ Halt ----------------------------------------------------------

pub struct Halt;

impl RenderMarker for Halt {
    fn base(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        halt(info, canvas);
        canvas.apply(info.empty_color);
        canvas.fill();
        if let Some(extent) = extent {
            canvas.move_to(0., y0s(info));
            canvas.line_to(0., extent.y);
        }
        canvas.apply(info.color);
        canvas.apply_line_width(w(info));
        canvas.apply(LineCap::Round);
        canvas.stroke();
    }

    fn casing(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        halt(info, canvas);
        canvas.apply(info.casing_color);
        canvas.apply_line_width(2. * w(info));
        canvas.stroke();
    }

    fn track_casing(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        if let Some(extent) = extent {
            canvas.move_to(0., y0s(info));
            canvas.line_to(extent.x, extent.y);
            canvas.apply(info.casing_color);
            canvas.apply_line_width(2. * w(info));
            canvas.apply(LineCap::Round);
            canvas.stroke();
        }
    }
}


//------------ Hst -----------------------------------------------------------

pub struct Hst;

impl RenderMarker for Hst {
    fn base(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        halt(info, canvas);
        canvas.apply(info.empty_color);
        canvas.fill();
        canvas.move_to(0., y0s(info));
        canvas.line_to(0., y1s(info));
        canvas.apply(info.color);
        canvas.apply_line_width(w(info));
        canvas.stroke();
    }

    fn casing(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        station(info, canvas);
        canvas.apply(info.casing_color);
        canvas.apply_line_width(w(info));
        canvas.stroke();
    }

    fn track_casing(
        &self, info: &RenderInfo, extent: Option<Point>, canvas: &mut  Group
    ) {
        Halt.track_casing(info, extent, canvas)
    }
}


//------------ GoodsHalt -----------------------------------------------------

pub struct GoodsHalt;

impl DetailRenderMarker for GoodsHalt {
    fn d3_base(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        halt(info, canvas);
        canvas.apply(info.empty_color);
        canvas.fill();
        canvas.apply(info.color);
        canvas.apply_line_width(w(info));
        canvas.stroke();
        canvas.new_path();
        canvas.move_to(x0s(info), y0s(info) + 0.5 * w(info));
        canvas.line_to(x1s(info), y1s(info) - 0.5 * w(info));
        canvas.apply_line_width(0.75 * w(info));
        canvas.stroke();
    }

    fn d3_casing(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        station(info, canvas);
        canvas.apply(info.casing_color);
        canvas.apply_line_width(w(info));
        canvas.stroke();
    }

    fn d4_base(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        halt(info, canvas);
        canvas.apply(info.empty_color);
        canvas.fill();
        canvas.move_to(x0s(info), y0s(info));
        canvas.line_to(x1s(info), y1s(info));
        canvas.apply(info.color);
        canvas.apply_line_width(w(info));
        canvas.stroke();
    }

    fn d4_casing(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        station(info, canvas);
        canvas.apply(info.casing_color);
        canvas.apply_line_width(w(info));
        canvas.stroke();
    }
}


//------------ Junction ------------------------------------------------------

pub struct Junction;

impl DetailRenderMarker for Junction {
    fn d3_base(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        chevron(canvas, 0.4 * info.m.sw(), info.cs, info.m.sh());
        canvas.apply(info.color);
        canvas.fill();
    }

    fn d4_base(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        canvas.move_to(0.7 * x0(info), y1s(info));
        canvas.line_to(0., 0.5 * w(info));
        canvas.line_to(0.7 * x1(info), y1s(info));
        canvas.apply(info.color);
        canvas.fill();
    }
}


//------------ Block ---------------------------------------------------------

pub struct Block;

impl DetailRenderMarker for Block {
    fn d3_base(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        chevron(canvas, 0.4 * info.m.sw(), info.cs, info.m.sh());
        canvas.apply(info.color);
        canvas.fill();
    }

    fn d4_base(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        canvas.move_to(0.7 * x0(info) + 0.5 * w(info), y1s(info));
        canvas.line_to(0., 1.5 * w(info));
        canvas.line_to(0.7 * x1(info) - 0.5 * w(info), y1s(info));
        canvas.apply(info.color);
        canvas.apply_line_width(w(info));
        canvas.stroke();
    }
}


//------------ OperatorBoundary ----------------------------------------------

pub struct OperatorBoundary;

impl RenderMarker for OperatorBoundary {
    fn base(
        &self, info: &RenderInfo, _extent: Option<Point>, canvas: &mut  Group
    ) {
        canvas.apply(info.color);
        canvas.move_to(0., 0.,);
        canvas.line_to(0., y1(info));
        canvas.apply_line_width(w(info));
        canvas.stroke();
        canvas.new_path();
        let radius = Self::radius(info);
        canvas.arc(0., y1(info) - radius, radius, 0., 2. * PI);
        canvas.fill();
    }
}

impl OperatorBoundary {
    fn radius(info: &RenderInfo) -> f64 {
        if info.detail < 4 {
            0.5 * (0.8 * (y1(info) - y0(info)))
        }
        else {
            0.5 * (0.66 * (y1(info) - y0(info)))
        }
    }
}


//------------ Helper Functions ----------------------------------------------

fn x0(info: &RenderInfo) -> f64 {
    -x1(info)
}

fn x1(info: &RenderInfo) -> f64 {
    0.5 * info.m.sw()
}

fn y0(info: &RenderInfo) -> f64 {
    if info.detail < 4 {
        1.5 * info.m.main_skip()
    }
    else {
        info.m.main_track()
    }
}

fn y1(info: &RenderInfo) -> f64 {
    info.m.sh()
}

fn w(info: &RenderInfo) -> f64 {
    info.m.station_stroke()
}

fn x0s(info: &RenderInfo) -> f64 {
    -x1s(info)
}

fn x1s(info: &RenderInfo) -> f64 {
    x1(info) - 0.5 * w(info)
}

fn y0s(info: &RenderInfo) -> f64 {
    y0(info) +  0.5 * w(info)
}

fn y1s(info: &RenderInfo) -> f64 {
    y1(info)  -  0.5 * w(info)
}

fn chevron(canvas: &mut Group, x: f64, y0: f64, y1: f64) {
    canvas.move_to(-x, y1);
    canvas.line_to(0., y0);
    canvas.line_to(x, y1);
}

fn station(info: &RenderInfo, canvas: &mut Group) {
    canvas.move_to(x0(info), y0(info));
    canvas.line_to(x0(info), y1(info));
    canvas.line_to(x1(info), y1(info));
    canvas.line_to(x1(info), y0(info));
    canvas.close_path();
}

fn halt(info: &RenderInfo, canvas: &mut Group) {
    canvas.move_to(x0s(info), y0s(info));
    canvas.line_to(x0s(info), y1s(info));
    canvas.line_to(x1s(info), y1s(info));
    canvas.line_to(x1s(info), y0s(info));
    canvas.close_path();
}


