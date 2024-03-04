use femtomap::render::Canvas;
use femtomap::world::Rect;
use crate::railway::class;
use crate::railway::style::Style;


//------------ Submodules with actual features -------------------------------

pub mod area;
pub mod border;
pub mod dot;
pub mod guide;
pub mod label;
pub mod marker;
pub mod track;


//------------ Store and StoreBuilder ----------------------------------------

/// The various feature sets we need.
pub struct Store {
    /// Railway lines and such.
    pub railway: FeatureSet,

    /// Line labels.
    pub line_labels: FeatureSet,

    /// Timetable labels.
    pub tt_labels: FeatureSet,

    /// Borders
    pub borders: FeatureSet,
}

#[derive(Default)]
pub struct StoreBuilder {
    pub railway: FeatureSetBuilder,
    pub line_labels: FeatureSetBuilder,
    pub tt_labels: FeatureSetBuilder,
    pub borders: FeatureSetBuilder,
}

impl StoreBuilder {
    pub fn finalize(self) -> Store {
        Store {
            railway: self.railway.finalize(),
            line_labels: self.line_labels.finalize(),
            tt_labels: self.tt_labels.finalize(),
            borders: self.borders.finalize(),
        }
    }
}


//------------ FeatureSet and FeatureSetBuilder ------------------------------

pub type FeatureSet = femtomap::feature::FeatureSet<AnyFeature>;
pub type FeatureSetBuilder = femtomap::feature::FeatureSetBuilder<AnyFeature>;


//------------ Feature -------------------------------------------------------

pub trait Feature {
    fn storage_bounds(&self) -> Rect;

    fn group(&self) -> Group;

    fn shape(
        &self, style: &Style, canvas: &Canvas
    ) -> AnyShape;
}

//------------ Shape ---------------------------------------------------------

pub trait Shape<'a> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas);
}

impl<'a, F: Fn(Stage, &Style, &mut Canvas) + 'a> Shape<'a> for F {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        (*self)(stage, style, canvas)
    }
}


//------------ AnyFeature ----------------------------------------------------

pub struct AnyFeature(Box<dyn Feature + Send + Sync>);

impl femtomap::feature::Feature for AnyFeature {
    type Group = Group;
    type Style = Style;
    type Shape<'a> = AnyShape<'a>;

    fn storage_bounds(&self) -> Rect {
        self.0.storage_bounds()
    }

    fn group(&self, _: &Self::Style) -> Self::Group {
        self.0.group()
    }

    fn shape(
        &self, style: &Self::Style, canvas: &Canvas
    ) -> Option<Self::Shape<'_>> {
        Some(self.0.shape(style, canvas))
    }
}

impl<T: Feature + Send + Sync + 'static> From<T> for AnyFeature {
    fn from(src: T) -> Self {
        Self(Box::new(src))
    }
}


//------------ AnyShape ------------------------------------------------------

pub struct AnyShape<'a>(Box<dyn Shape<'a> + 'a>);

impl<'a> AnyShape<'a> {
    pub fn single_stage<F: Fn(&Style, &mut Canvas) + 'a>(
        op: F
    ) -> Self {
        Self::from(move |stage: Stage, style: &Style, canvas: &mut Canvas| {
            if matches!(stage, Stage::Base) {
                (op)(style, canvas)
            }
        })
    }

    pub fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        self.0.render(stage, style, canvas)
    }
}

impl<'a, T: Shape<'a> + 'a> From<T> for AnyShape<'a> {
    fn from(src: T) -> Self {
        AnyShape(Box::new(src))
    }
}


//------------ Stage ---------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
pub enum Stage {
    #[default]
    Back,
    Casing,

    /// The base for shapes that have an inside.
    InsideBase,

    /// The inside of shapes that have an inside.
    Inside,

    /// The base for shapes that don’t have an inside.
    ///
    /// These needs to be drawn last so they can paint over the insides.
    Base,

    /// The casing for markers.
    MarkerCasing,

    /// The base for markers.
    MarkerBase,
}

impl IntoIterator for Stage {
    type Item = Self;
    type IntoIter = StageIter;

    fn into_iter(self) -> Self::IntoIter {
        StageIter(Some(self))
    }
}


#[derive(Clone, Copy, Debug)]
pub struct StageIter(Option<Stage>);

impl Iterator for StageIter {
    type Item = Stage;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.0;
        if let Some(stage) = self.0 {
            let next = match stage {
                Stage::Back => Some(Stage::Casing),
                Stage::Casing => Some(Stage::InsideBase),
                Stage::InsideBase => Some(Stage::Inside),
                Stage::Inside => Some(Stage::Base),
                Stage::Base => Some(Stage::MarkerCasing),
                Stage::MarkerCasing => Some(Stage::MarkerBase),
                Stage::MarkerBase => None,
            };
            self.0 = next;
        }
        res 
    }
}


//------------ Category ------------------------------------------------------

/// The category of features.
///
/// This is used in [`Group`] so that features are drawn in the correct order.
/// Lowest value categories are drawn first.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Category {
    Back = 0,
    Marker,
    Track,
    Label,
}


//------------ Group ---------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Group {
    category: Category,
    status: class::Status,
    pax: class::Pax,
}

impl Group {
    fn new(
        category: Category, status: class::Status, pax: class::Pax
    ) -> Self {
        Self { category, status, pax }
    }

    pub fn with_category(category: Category) -> Self {
        Self::new(category, class::Status::Open, class::Pax::Full)
    }

    pub fn with_railway(
        category: Category, railway: &class::Railway
    ) -> Self {
        Self::new(category, railway.status(), railway.pax())
    }

    pub fn with_status(
        category: Category, status: class::Status
    ) -> Self {
        Self::new(category, status, class::Pax::Full)
    }
}


//============ Testing =======================================================

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn group_ord() {
        assert!(
            Group::new(
                Category::Track, class::Status::Open, class::Pax::None
            ) > Group::new(
                Category::Track, class::Status::Removed, class::Pax::Full
            )
        );
    }
}

