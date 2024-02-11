//! The connection to femtomap’s eval machinery.

use std::sync::{Arc, Mutex};
use femtomap::import::eval;
use femtomap::import::ast::Pos;
use femtomap::import::eval::{EvalErrors, Failed, SymbolSet};
use femtomap::import::path::{ImportPathSet};
use femtomap::path::Distance;
use crate::feature::StoreBuilder;
use crate::feature::label::Layout;
use super::{functions, procedures, units};

pub type ArgumentList<'s> = eval::ArgumentList<'s, Builtin>;
pub type Expression<'s> = eval::Expression<'s, Builtin>;
pub type Scope<'s> = eval::Scope<'s, Builtin>;
pub type Value<'s> = eval::Value<'s, Builtin>;

pub struct Builtin {
    paths: ImportPathSet,
    store: Arc<Mutex<StoreBuilder>>,
}

impl Builtin {
    pub fn new(paths: ImportPathSet, store: Arc<Mutex<StoreBuilder>>) -> Self {
        Self { paths, store }
    }

    pub fn with_store<F, T>(&self, op: F) -> T
    where F: FnOnce(&mut StoreBuilder) -> T {
        op(&mut self.store.lock().unwrap())
    }
}

impl eval::Builtin for Builtin {
    type Scope = RenderParams;
    type Value = Custom;

    fn eval_distance(
        &self,
        number: f64,
        unit: &str,
        _scope: &eval::Scope<Self>,
        pos: Pos, err: &mut EvalErrors,
    ) -> Result<Distance, Failed> {
        for (name, factor) in units::WORLD_DISTANCES {
            if unit == *name {
                return Ok(Distance::world(number * factor))
            }
        }
        for (name, index, factor) in units::MAP_DISTANCES {
            if unit == *name {
                return Ok(Distance::map(number * factor, *index))
            }
        }
        err.add(pos, format!("unknown distance unit '{}'", unit));
        Err(Failed)
    }

    fn eval_function<'s>(
        &'s self,
        name: &str,
        args: eval::ArgumentList<'s, Self>,
        scope: &eval::Scope<'s, Self>,
        pos: Pos,
        err: &mut EvalErrors,
    ) -> Result<eval::Value<'s, Self>, Failed> {
        functions::eval(name, args, scope, &self.paths, pos, err)
    }

    fn eval_procedure(
        &self,
        name: &str,
        args: eval::ArgumentList<Self>,
        scope: &eval::Scope<Self>,
        pos: Pos,
        err: &mut EvalErrors,
    ) -> Result<(), Failed> {
        procedures::eval(name, args, scope, pos, err)
    }

    fn eval_render_param(
        &self,
        name: &str,
        value: eval::Expression<Self>,
        scope: &mut eval::Scope<Self>,
        pos: Pos,
        err: &mut EvalErrors
    ) -> Result<(), Failed> {
        scope.custom_mut().update(name, value, pos, err)
    }
}


#[derive(Clone)]
pub enum Custom {
    Layout(Layout),
}

impl From<Layout> for Custom {
    fn from(src: Layout) -> Self {
        Self::Layout(src)
    }
}


//------------ RenderParams --------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct RenderParams {
    detail: Option<(f64, f64)>,
    zoom: Option<Zoom>,
    layer: Option<i16>,
}

impl RenderParams {
    fn update(
        &mut self,
        target: &str,
        value: Expression,
        pos: Pos, err: &mut EvalErrors
    ) -> Result<(), Failed> {
        match target {
            "detail" => self.update_detail(value, err),
            "layer" => self.update_layer(value, err),
            "link" => self.update_link(value, err),
            "zoom" => self.update_zoom(value, err),
            "style" => { } // XXX Deprecated
            _ => {
                err.add(pos, format!("unknown render param {}", target));
                return Err(Failed)
            }
        }
        Ok(())
    }

    fn update_detail(
        &mut self,
        value: Expression,
        err: &mut EvalErrors
    ) {
        match value.value {
            Value::Number(val) => {
                match u8::try_from(val) {
                    Ok(val) => self.detail = Some((val.into(), val.into())),
                    Err(_) => err.add(value.pos, "expected 8-bit integer"),
                }
            }
            Value::List(val) => {
                let [left, right] = match <[_; 2]>::try_from(val) {
                    Ok(val) => val,
                    Err(_) => {
                        err.add(
                            value.pos, "expected number or pair of numbers"
                        );
                        return;
                    }
                };
                let left = left.eval::<u8>(err).map(Into::into);
                let right = right.eval::<u8>(err).map(Into::into);
                if let (Ok(left), Ok(right)) = (left, right) {
                    self.detail = Some(
                        if left < right {
                            (left, right)
                        }
                        else {
                            (right, left)
                        }
                    )
                }
            }
            _ => err.add(value.pos, "expected number or pair of numbers"),
        }
    }

    fn update_layer(
        &mut self,
        value: Expression,
        err: &mut EvalErrors
    ) {
        if let Ok((val, _)) = value.eval(err) {
            self.layer = Some(val)
        }
    }

    fn update_link(
        &mut self,
        value: Expression,
        err: &mut EvalErrors
    ) {
        let _ = value.eval::<String>(err);
    }

    fn update_zoom(
        &mut self,
        value: Expression,
        err: &mut EvalErrors
    ) {
        if let Ok(value) = Zoom::from_value(value, err) {
            self.zoom = Some(value)
        }
    }

    fn detail(scope: &Scope) -> Option<(f64, f64)> {
        if let Some(detail) = scope.custom().detail {
            return Some(detail)
        }
        match scope.parent() {
            Some(parent) =>  Self::detail(parent),
            None => None
        }
    }

    fn zoom(scope: &Scope) -> Option<Zoom> {
        if let Some(zoom) = scope.custom().zoom {
            return Some(zoom)
        }
        match scope.parent() {
            Some(parent) =>  Self::zoom(parent),
            None => None
        }
    }

    fn layer(scope: &Scope) -> Option<i16> {
        if let Some(layer) = scope.custom().layer {
            return Some(layer)
        }
        match scope.parent() {
            Some(parent) =>  Self::layer(parent),
            None => None
        }
    }
}


//----------- Extending Scope ------------------------------------------------

pub trait ScopeExt {
    fn detail(
        &self, pos: Pos, err: &mut EvalErrors
    ) -> Result<(f64, f64), Failed>;

    fn layer(&self) -> i16;
}

impl<'s> ScopeExt for Scope<'s> {
    fn detail(
        &self, pos: Pos, err: &mut EvalErrors
    ) -> Result<(f64, f64), Failed> {
        match RenderParams::detail(self) {
            Some((x, y)) => {
                Ok(match RenderParams::zoom(self) {
                    Some(Zoom::Low) => {
                        (x - 0.1, y + 0.4)
                    }
                    Some(Zoom::High) => {
                        (x + 0.4, y + 0.9)
                    }
                    None => {
                        (x - 0.1, y + 0.9)
                    }
                })
            }
            None => {
                err.add(pos, "no detail level selected yet");
                Err(Failed)
            }
        }
        
    }

    fn layer(&self) -> i16 {
        RenderParams::layer(self).unwrap_or(0)
    }
}

//------------ Zoom ----------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum Zoom {
    Low,
    High,
}

impl Zoom {
    fn from_value(
        value: Expression,
        err: &mut EvalErrors
    ) -> Result<Self, Failed> {
        let (value, pos) = value.eval::<(SymbolSet, _)>(err)?;
        if value == "high" {
            Ok(Zoom::High)
        }
        else if value == "low" {
            Ok(Zoom::Low)
        }
        else {
            err.add(pos, "expected symbol :high or :low");
            Err(Failed)
        }
    }
}

