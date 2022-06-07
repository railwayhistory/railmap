//! The theme guiding the import.

use std::hash::Hash;
use std::str::FromStr;
use crate::import::{ast, eval};
use crate::import::Failed;
use crate::render::feature::{Feature, FeatureSet};
use crate::render::label::Span;
use crate::render::path::Distance;
use crate::tile::TileId;


//------------ Theme ---------------------------------------------------------

pub trait Theme: Sized + Clone + Send + Sync + 'static {
    type Function: Clone;
    type Procedure: Clone;
    type CustomExpr: Clone;
    type RenderParams: Default + Clone;
    type Style: Style;
    type Feature: Feature<Self> + Send + Sync + 'static;
    type Span: Span<Self>;

    fn eval_unit(
        &self, number: f64, unit: &str, pos: ast::Pos, err: &mut eval::Error,
    ) -> Result<Distance, Failed>;

    fn lookup_function(&self, name: &str) -> Option<Self::Function>;
    fn lookup_procedure(&self, name: &str) -> Option<Self::Procedure>;

    fn eval_function(
        &self,
        function: &Self::Function,
        args: eval::ArgumentList<Self>,
        scope: &eval::Scope<Self>,
        err: &mut eval::Error,
    ) -> Result<eval::ExprVal<Self>, Result<eval::ArgumentList<Self>, Failed>>;

    fn eval_procedure(
        &self,
        procedure: &Self::Procedure,
        pos: ast::Pos,
        args: eval::ArgumentList<Self>,
        scope: &eval::Scope<Self>,
        features: &mut FeatureSet<Self>,
        err: &mut eval::Error,
    ) -> Result<(), Failed>;

    fn update_render_params(
        &self,
        param: &mut Self::RenderParams,
        target: &str,
        value: eval::Expression<Self>,
        pos: ast::Pos,
        err: &mut eval::Error
    ) -> Result<(), Failed>;

    fn style(
        &self, tile_id: &TileId<<Self::Style as Style>::StyleId>,
    ) -> Self::Style;

    fn index_page(&self) -> &'static [u8];
}


//------------ Style ---------------------------------------------------------

pub trait Style {
    type StyleId: Send + Sync + 'static + Clone + Hash + Eq + FromStr;

    /// Returns the a multiplier by which to grow the bounds.
    ///
    /// This value is used to increase the size of the rendered area in order
    /// to correct for incorrect storage bounds of features.
    fn bounds_correction(&self) -> f64;

    /// Returns the magnification factor.
    ///
    /// Canvas lengths will be scaled by this value.
    fn mag(&self) -> f64;

    fn detail(&self) -> u8;

    fn scale(&mut self, canvas_bp: f64);
}

