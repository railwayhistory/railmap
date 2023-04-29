
use std::sync::Arc;
use femtomap::path::Distance;
use femtomap::feature::FeatureSetBuilder;
use femtomap::render::Canvas;
use hyper::Body;
use crate::config::Config;
use crate::import::Failed;
use crate::import::{ast, eval};
use crate::theme;
use crate::tile::{TileId, TileFormat};
use super::feature::{Feature, Stage};
use super::mapkey::map_key;
use super::style::{ColorSet, Style};


//------------ Railwayhistory ------------------------------------------------

#[derive(Clone, Default)]
pub struct Railwayhistory {
    colors: Arc<ColorSet>,
}

impl theme::Theme for Railwayhistory {
    type Function = super::functions::Function;
    type Procedure = super::procedures::Procedure;
    type CustomExpr = super::feature::label::Layout;
    type RenderParams = RenderParams;
    type Style = Style;
    type Feature = Feature;
    type Stage = Stage;

    fn config(&mut self, config: &Config) {
        let mut colors = ColorSet::default();
        colors.update(&config.colors);
        self.colors = Arc::new(colors);
    }

    fn eval_distance(
        &self, number: f64, unit: &str,
        _scope: &eval::Scope<Self>,
        pos: ast::Pos, err: &mut eval::Error,
    ) -> Result<Distance, Failed> {
        super::units::resolve_unit(number, unit).ok_or_else(|| {
            err.add(pos, format!("unknown unit '{}'", unit));
            Failed
        })
    }

    fn lookup_function(
        &self, name: &str
    ) -> Option<Self::Function> {
        super::functions::Function::lookup(name)
    }

    fn lookup_procedure(
        &self, name: &str
    ) -> Option<Self::Procedure> {
        super::procedures::Procedure::lookup(name)
    }

    fn eval_function(
        &self,
        function: &Self::Function,
        args: eval::ArgumentList<Self>,
        scope: &eval::Scope<Self>,
        err: &mut eval::Error,
    ) -> Result<eval::ExprVal<Self>, Result<eval::ArgumentList<Self>, Failed>> {
        function.eval(args, scope, err)
    }

    fn eval_procedure(
        &self,
        procedure: &Self::Procedure,
        pos: ast::Pos,
        args: eval::ArgumentList<Self>,
        scope: &eval::Scope<Self>,
        features: &mut FeatureSetBuilder<Self::Feature>,
        err: &mut eval::Error,
    ) -> Result<(), Failed> {
        procedure.eval(pos, args, scope, features, err)
    }

    fn update_render_params(
        &self,
        param: &mut Self::RenderParams,
        target: &str,
        value: eval::Expression<Self>,
        pos: ast::Pos,
        err: &mut eval::Error
    ) -> Result<(), Failed> {
        param.update(target, value, pos, err)
    }

    fn style(
        &self,
        tile_id: &TileId<<Self::Style as theme::Style>::StyleId>,
    ) -> Self::Style {
        Style::new(tile_id, self.colors.clone())
    }

    fn index_page(&self) -> &'static [u8] {
        include_bytes!("../../html/railwayhistory/index.html").as_ref()
    }

    fn map_key(
        &self, zoom: u8,
        style: <Self::Style as theme::Style>::StyleId,
        format: TileFormat
    ) -> Body {
        map_key(
            Style::new_map_key(zoom, style, format, self.colors.clone()),
            format,
        )
    }

    fn render_shape<'a>(
        &self,
        shape: &<Self::Feature as femtomap::feature::Feature>::Shape<'a>,
        stage: &Self::Stage,
        style: &Self::Style,
        canvas: &mut Canvas,
    ) {
        shape.render(*stage, style, canvas);
    }
}


//------------ RenderParams --------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct RenderParams {
    detail: Option<(f64, f64)>,
    layer: i16,
    style: Option<ast::ShortString>,
}

impl RenderParams {
    fn update(
        &mut self,
        target: &str,
        value: eval::Expression<Railwayhistory>,
        pos: ast::Pos,
        err: &mut eval::Error
    ) -> Result<(), Failed> {
        match target {
            "detail" => self.update_detail(value, err),
            "layer" => self.update_layer(value, err),
            "link" => self.update_link(value, err),
            "style" => self.update_style(value, err),
            _ => {
                err.add(pos, format!("unknown render param {}", target));
                return Err(Failed)
            }
        }
        Ok(())
    }

    fn update_detail(
        &mut self,
        value: eval::Expression<Railwayhistory>,
        err: &mut eval::Error
    ) {
        match value.value {
            eval::ExprVal::Number(val) => {
                match val.into_u8() {
                    Ok(val) => self.detail = Some((val as f64, val as f64)),
                    Err(_) => err.add(value.pos, "expected 8-bit integer"),
                }
            }
            eval::ExprVal::List(val) => {
                if val.len() != 2 {
                    err.add(value.pos, "expected number or pair of numbers");
                    return;
                }
                let mut val = val.into_iter();
                let left = match val.next().unwrap().into_u8(err) {
                    Ok(left) => left.0 as f64,
                    Err(_) => return,
                };
                let right = match val.next().unwrap().into_u8(err) {
                    Ok(right) => right.0 as f64,
                    Err(_) => return,
                };
                self.detail = Some(if left < right {
                    (left, right)
                }
                else {
                    (right, left)
                });
            }
            _ => err.add(value.pos, "expected number or pair of numbers"),
        }
    }

    fn update_layer(
        &mut self,
        value: eval::Expression<Railwayhistory>,
        err: &mut eval::Error
    ) {
        if let Ok(val) = value.into_i16(err) {
            self.layer = val.0
        }
    }

    fn update_link(
        &mut self,
        value: eval::Expression<Railwayhistory>,
        err: &mut eval::Error
    ) {
        let _ = value.into_text(err);
    }

    fn update_style(
        &mut self,
        value: eval::Expression<Railwayhistory>,
        err: &mut eval::Error
    ) {
        if let Ok(value) = value.into_symbol(err) {
            self.style = Some(value.0)
        }
    }

    pub fn detail(
        &self, pos: ast::Pos, err: &mut eval::Error
    ) -> Result<(f64, f64), Failed> {
        match self.detail {
            Some(detail) => Ok(detail),
            None => {
                err.add(pos, "no detail level selected yet");
                Err(Failed)
            }
        }
    }

    pub fn layer(&self) -> i16 {
        self.layer
    }

    pub fn style(&self) -> Option<&str> {
        self.style.as_ref().map(ast::ShortString::as_str)
    }
}

