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

pub mod oldmarker;

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
    ) -> AnyShape<'_>;
}

//------------ Shape ---------------------------------------------------------

pub trait Shape<'a> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas);

    fn stages(&self) -> StageSet;
}

impl<'a, T0, T1> Shape<'a> for (T0, T1)
where
    T0: Shape<'a>,
    T1: Shape<'a>
{
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        self.0.render(stage, style, canvas);
        self.1.render(stage, style, canvas);
    }

    fn stages(&self) -> StageSet {
        self.0.stages().add_set(self.1.stages())
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
        Self::from(BaseFnShape { op })
    }

    pub fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        self.0.render(stage, style, canvas)
    }

    pub fn stages(&self) -> StageSet {
        self.0.stages()
    }
}

impl<'a, T: Shape<'a> + 'a> From<T> for AnyShape<'a> {
    fn from(src: T) -> Self {
        AnyShape(Box::new(src))
    }
}


//------------ BaseFnShape ---------------------------------------------------

struct BaseFnShape<F: Fn(&Style, &mut Canvas)> {
    op: F
}

impl<'a, F: Fn(&Style, &mut Canvas) + 'a> Shape<'a> for BaseFnShape<F> {
    fn render(&self, stage: Stage, style: &Style, canvas: &mut Canvas) {
        if matches!(stage, Stage::Base) {
            (self.op)(style, canvas)
        }
    }

    fn stages(&self) -> StageSet {
        StageSet::default().add(Stage::Base)
    }
}


//------------ Stage ---------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
#[repr(u16)]
pub enum Stage {
    /// Background.
    #[default]
    Back = 0,

    Casing,

    AbandonedBase,
    AbandonedMarking,

    LimitedBase,
    LimitedMarking,

    Base,
    Marking,

    MarkerCasing,
    MarkerBase,
    MarkerMarking,
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
                Stage::Casing => Some(Stage::AbandonedBase),
                Stage::AbandonedBase => Some(Stage::AbandonedMarking),
                Stage::AbandonedMarking => Some(Stage::LimitedBase),
                Stage::LimitedBase => Some(Stage::LimitedMarking),
                Stage::LimitedMarking => Some(Stage::Base),
                Stage::Base => Some(Stage::Marking),
                Stage::Marking => Some(Stage::MarkerCasing),
                Stage::MarkerCasing => Some(Stage::MarkerBase),
                Stage::MarkerBase => Some(Stage::MarkerMarking),
                Stage::MarkerMarking => None,
            };
            self.0 = next;
        }
        res 
    }
}


//------------ StageSet ------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
pub struct StageSet(u16);

impl StageSet {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn all() -> Self {
        Self(u16::MAX)
    }

    pub const fn from_slice(mut stages: &[Stage]) -> Self {
        let mut res = Self::empty();
        while let Some((head, tail)) = stages.split_first() {
            res = res.add(*head);
            stages = tail;
        }
        res
    }

    pub const fn add(self, stage: Stage) -> Self {
        Self(self.0 | (stage as u16))
    }

    pub const fn add_set(self, set: StageSet) -> Self {
        Self(self.0 | set.0)
    }

    pub fn contains(self, stage: Stage) -> bool {
        self.0 & (stage as u16) != 0
    }

    pub fn iter(self) -> impl Iterator<Item = Stage> {
        Stage::default().into_iter().filter(move |stage| self.contains(*stage))
    }
}

impl From<Stage> for StageSet {
    fn from(stage: Stage) -> Self {
        Self::default().add(stage)
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

