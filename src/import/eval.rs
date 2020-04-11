use std::ops;
use std::convert::TryInto;
use std::collections::HashMap;
use std::str::FromStr;
use crate::features::path;
use crate::features::{FeatureSet, Path};
use crate::features::contour::{Contour, ContourRule};
use crate::import::functions;
use crate::import::units;
use crate::import::path::{ImportPath, PathSet};
use super::ast;


//------------ Scope ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Scope<'a> {
    paths: &'a PathSet,
    variables: HashMap<String, ExprVal>,
    params: RenderParams,
}

impl<'a> Scope<'a> {
    pub fn new(paths: &'a PathSet) -> Self {
        Scope {
            paths,
            variables: HashMap::new(),
            params: Default::default(),
        }
    }

    pub fn get_path(&self, name: &str) -> Option<&ImportPath> {
        self.paths.get(name.as_ref())
    }

    pub fn get_var(&self, ident: &str) -> Option<ExprVal> {
        self.variables.get(ident).cloned()
    }

    pub fn set_var(&mut self, ident: String, value: ExprVal) {
        self.variables.insert(ident.clone(), value);
    }
}


//------------ RenderParams --------------------------------------------------

#[derive(Clone, Debug, Default)]
struct RenderParams
{
    detail: Option<(u8, u8)>,
    layer: f64,
}

impl RenderParams {
    fn update(
        &mut self,
        target: &str,
        value: Expression,
        pos: ast::Pos,
        err: &mut Error
    ) {
        let res = match target {
            "detail" => {
                match value.value {
                    ExprVal::Number(number) => {
                        number.into_u8().map(|num| {
                            self.detail = Some((num, num));
                        })
                    }
                    ExprVal::Range(range) => {
                        range.into_u8().map(|num| {
                            self.detail = Some(num);
                        })
                    }
                    _ => {
                        Err("expected number or range".into())
                    }
                }
            }
            "layer" => {
                match value.value {
                    ExprVal::Number(val) => {
                        self.layer = val.into_f64();
                        Ok(())
                    }
                    _ => Err("expected number".into())
                }
            }
            _ => {
                Err(format!("unknown render param {}", target))
            }
        };
        if let Err(e) = res {
            err.add(pos, e)
        }
    }
}


//------------ Expression ----------------------------------------------------

/// An expression that has been evaluated for the current scope.
///
/// The variants are the concrete types that we have.
#[derive(Clone, Debug)]
pub struct Expression {
    value: ExprVal,
    pos: ast::Pos
}

/// The value of a resolved expression.
///
/// This has a shorthand name because we are going to type it a lot.
#[derive(Clone, Debug)]
pub enum ExprVal {
    Distance(Distance),
    Range(Range),
    Number(Number),
    Text(String),
    ContourRule(ContourRule),
    Path(path::Segment),
}

impl Expression {
    fn new(value: ExprVal, pos: ast::Pos) -> Self {
        Expression { value, pos }
    }

    pub fn into_contour_rule(self, err: &mut Error) -> Option<ContourRule> {
        match self.value {
            ExprVal::ContourRule(rule) => Some(rule),
            _ => {
                err.add(self.pos, "expected contour rule");
                None
            }
        }
    }

    pub fn into_path(
        self, err: &mut Error
    ) -> Option<path::Segment> {
        match self.value {
            ExprVal::Path(path) => Some(path),
            _ => {
                err.add(self.pos, "expected path segment");
                None
            }
        }
    }
}


//------------ Distance ------------------------------------------------------

/// An evaluated distance.
#[derive(Clone, Copy, Debug, Default)]
pub struct Distance {
    /// The world component of the distance.
    ///
    /// This is not yet scaled to storage coordinates, i.e., this value is the
    /// acutal distance along the face of the Earth in _bp._
    world: f64,

    /// The canvas component of the distance.
    ///
    /// This is the distance along the canvas in _bp._
    canvas: f64,
}

impl Distance {
    /// Creates a new distance from the world and canvas components.
    fn new(world: f64, canvas: f64) -> Self {
        Distance { world, canvas }
    }
}

impl ops::AddAssign for Distance {
    fn add_assign(&mut self, other: Distance) {
        self.world += other.world;
        self.canvas += other.canvas;
    }
}

impl ops::SubAssign for Distance {
    fn sub_assign(&mut self, other: Distance) {
        self.world -= other.world;
        self.canvas -= other.canvas;
    }
}


//------------ Range ---------------------------------------------------------

/// An evaluated range expression.
///
/// This contains the lower and upper bounds as numbers.
#[derive(Clone, Copy, Debug)]
pub struct Range {
    first: Number,
    second: Number,
}

impl Range {
    fn into_u8(self) -> Result<(u8, u8), String> {
        let first = self.first.into_u8()?;
        let second = self.second.into_u8()?;
        Ok((first, second))
    }
}


//------------ Number --------------------------------------------------------

/// An evaluated number.
///
/// This number can either be an integer or a float. Note that the integer
/// variant is limited to a `i32`. Integers outside its range will be
/// represented by the float variant.
#[derive(Clone, Copy, Debug)]
pub enum Number {
    Int(i32),
    Float(f64),
}

impl Number {
    fn into_u8(self) -> Result<u8, String> {
        match self {
            Number::Int(val) => {
                val.try_into().map_err(|_| "value out of range".into())
            }
            Number::Float(_) => {
                Err("integer number expected".into())
            }
        }
    }

    fn into_f64(self) -> f64 {
        match self {
            Number::Int(val) => val.into(),
            Number::Float(val) => val
        }
    }
}


//------------ ArgumentList --------------------------------------------------

/// Evaluated arguments of a function.
#[derive(Clone, Debug, Default)]
pub struct ArgumentList {
    /// The list of arguments.
    ///
    /// The first element is the keyword if this is a keyword argument.
    /// Otherwise it is a positional argument.
    arguments: Vec<(Option<String>, Expression)>,
}


//============ Evaluations for AST Types =====================================
//
// These are here in alphabetical order.

impl ast::ArgumentList {
    fn eval(self, scope: &Scope, err: &mut Error) -> Option<ArgumentList> {
        let mut good = true;
        let mut res = ArgumentList::default();
        for argument in self.arguments {
            match argument {
                ast::Argument::Keyword(assignment) => {
                    match assignment.expression.eval(scope, err) {
                        Some(expr) => {
                            res.arguments.push(
                                (Some(assignment.target.eval()), expr)
                            )
                        }
                        None => good = false,
                    }
                }
                ast::Argument::Pos(expr) => {
                    match expr.eval(scope, err) {
                        Some(expr) => res.arguments.push((None, expr)),
                        None => good = false,
                    }
                }
            }
        }
        if good {
            Some(res)
        }
        else {
            None
        }
    }
}

impl ast::AssignmentList {
    fn eval_params(
        self,
        params: &mut RenderParams,
        scope: &Scope,
        err: &mut Error
    ) {
        for item in self.assignments {
            let target = item.target.eval();
            let expression = match item.expression.eval(&scope, err) {
                Some(expression) => expression,
                None => continue,
            };
            params.update(&target, expression, item.pos, err);
        }
    }
}

impl ast::Contour {
    pub fn eval(
        self,
        scope: &mut Scope,
        features: &mut FeatureSet,
        err: &mut Error
    ) {
        // Get the rendering parameters for this contour.
        let mut params = scope.params.clone();
        if let Some(value) = self.params {
            value.eval_params(&mut params, scope, err);
        }

        // Get all the parts.
        let rule = self.rule.eval_contour_rule(scope, err);
        let path = self.path.eval(scope, err);

        // If we don’t have a detail, complain.
        let detail = match params.detail {
            Some(detail) => detail,
            None => {
                err.add(self.pos, "'detail' rendering parameter not yet set");
                return
            }
        };

        // If we have both a rule and a path, create the feature.
        if let (Some(rule), Some(path)) = (rule, path) {
            features.insert(
                Contour::new(path, rule),
                detail,  params.layer   
            )
        }
    }
}

impl ast::Distance {
    fn eval(self, err: &mut Error) -> Option<Distance> {
        let mut res = self.first.eval(err);
        for (addsub, value) in self.others {
            if let Some(distance) = value.eval(err) {
                if let Some(res) = res.as_mut() {
                    match addsub {
                        ast::AddSub::Add => *res += distance,
                        ast::AddSub::Sub => *res -= distance,
                    }
                }
            }
        }
        res
    }
}

impl ast::Expression {
    fn eval(self, scope: &Scope, err: &mut Error) -> Option<Expression> {
        let pos = self.pos();
        let res = match self {
            ast::Expression::Distance(distance) => {
                distance.eval(err).map(ExprVal::Distance)
            }
            ast::Expression::Range(range) => {
                Some(ExprVal::Range(range.eval()))
            }
            ast::Expression::Number(number) => {
                Some(ExprVal::Number(number.eval()))
            }
            ast::Expression::Text(text) => {
                Some(ExprVal::Text(text.eval()))
            }
            ast::Expression::Function(function) => {
                function.eval(scope, err)
            }
            ast::Expression::Variable(ident) => {
                let pos = ident.pos;
                let ident = ident.eval();
                match scope.get_var(&ident) {
                    Some(expr) => Some(expr),
                    None => {
                        err.add(
                            pos,
                            format!("unresolved variable '{}'", ident)
                        );
                        None
                    }
                }
            }
        };
        res.map(|val| Expression::new(val, pos))
    }

    fn eval_contour_rule(
        self, scope: &Scope, err: &mut Error
    ) -> Option<ContourRule> {
        self.eval(scope, err).and_then(|expr| expr.into_contour_rule(err))
    }

    fn eval_path(
        self, scope: &Scope, err: &mut Error
    ) -> Option<path::Segment> {
        self.eval(scope, err).and_then(|expr| expr.into_path(err))
    }
}

impl ast::Function {
    fn eval(self, scope: &Scope, err: &mut Error) -> Option<ExprVal> {
        let name = self.name.eval();
        let args = match self.args {
            Some(args) => args.eval(scope, err)?,
            None => Default::default()
        };
        functions::eval(name, args, err)
    }
}

impl ast::Identifier {
    fn eval(self) -> String {
        self.ident
    }
}

impl ast::Let {
    fn eval(self, scope: &mut Scope, err: &mut Error) {
        for assignment in self.assignments.assignments {
            let target = assignment.target.eval();
            let expression = match assignment.expression.eval(scope, err) {
                Some(expression) => expression,
                None => continue,
            };
            scope.set_var(target, expression.value);
        }
    }
}

impl ast::Number {
    fn eval(self) -> Number {
        if let Ok(value) = i32::from_str(&self.value) {
            Number::Int(value)
        }
        else {
            Number::Float(f64::from_str(&self.value).unwrap())
        }
    }

    fn eval_float(self) -> f64 {
        f64::from_str(&self.value).unwrap()
    }
}

impl ast::Path {
    fn eval(self, scope: &mut Scope, err: &mut Error) -> Option<Path> {
        let mut path = self.first.eval_path(scope, err).map(Path::new);
        for (conn, expr) in self.others {
            let (post, pre) = conn.tension();
            if let Some(seg) = expr.eval_path(scope, err) {
                if let Some(path) = path.as_mut() {
                    path.push(post, pre, seg)
                }
            }
        }
        path
    }
}

impl ast::Range {
    fn eval(self) -> Range {
        Range {
            first: self.first.eval(),
            second: self.second.eval()
        }
    }
}

impl ast::Statement {
    pub fn eval(
        self,
        scope: &mut Scope,
        features: &mut FeatureSet,
        err: &mut Error
    ) {
        match self {
            ast::Statement::Let(stm) => stm.eval(scope, err),
            ast::Statement::With(with) => with.eval(scope, features, err),
            ast::Statement::Contour(contour) => {
                contour.eval(scope, features, err)
            }
        }
    }
}

impl ast::StatementList {
    pub fn eval(
        self,
        scope: &mut Scope,
        features: &mut FeatureSet,
        err: &mut Error
    ) {
        for statement in self.statements {
            statement.eval(scope, features, err)
        }
    }
}

impl ast::Text {
    fn eval(self) -> String {
        let mut res = self.first.content;
        for item in self.others {
            res.push_str(&item.content);
        }
        res
    }
}

impl ast::UnitNumber {
    /// Evaluates the unit number.
    ///
    /// On success, returns the world component in the first element and the
    /// canvas component in the second.
    fn eval(self, err: &mut Error) -> Option<Distance> {
        for (unit, factor) in units::WORLD_DISTANCES {
            if self.unit == unit {
                return Some(Distance::new(
                    self.number.eval_float() * factor, 0.
                ))
            }
        }
        for (unit, factor) in units::CANVAS_DISTANCES {
            if self.unit == unit {
                return Some(Distance::new(
                    0., self.number.eval_float() * factor
                ))
            }
        }
        err.add(self.pos, format!("unknown unit '{}'", self.unit));
        None
    }
}

impl ast::With {
    pub fn eval(
        self,
        scope: &mut Scope,
        features: &mut FeatureSet,
        err: &mut Error
    ) {
        // We need our own scope.
        let mut scope = scope.clone();

        // Next we update the render params from self.params.
        let mut params = scope.params.clone();
        self.params.eval_params(&mut params, &scope, err);
        scope.params = params;

        // Finally we run the block.
        self.block.eval(&mut scope, features, err);
    }
}


//============ Errors ========================================================

//------------ Error ---------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Error {
    errors: Vec<(ast::Pos, String)>,
}

impl Error {
    pub fn add(&mut self, pos: ast::Pos, error: impl Into<String>) {
        self.errors.push((pos, error.into()))
    }
}

