use std::convert::TryInto;
use std::collections::HashMap;
use std::str::FromStr;
use crate::features::path;
use crate::features::{
    Color, Contour, ContourRule, Distance, FeatureSet, Path, Position,
    Symbol, SymbolRule,
};
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

    pub fn paths(&self) -> &PathSet {
        &self.paths
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
    pub value: ExprVal,
    pub pos: ast::Pos
}

impl Expression {
    fn new(value: ExprVal, pos: ast::Pos) -> Self {
        Expression { value, pos }
    }

    pub fn into_color(self, err: &mut Error) -> Option<Color> {
        match self.value {
            ExprVal::Color(value) => Some(value),
            _ => {
                err.add(self.pos, "expected color");
                None
            }
        }
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

    pub fn into_symbol_rule(self, err: &mut Error) -> Option<SymbolRule> {
        match self.value {
            ExprVal::SymbolRule(rule) => Some(rule),
            _ => {
                err.add(self.pos, "expected symbol rule");
                None
            }
        }
    }

    pub fn into_distance(self, err: &mut Error) -> Option<Distance> {
        match self.value {
            ExprVal::Distance(path) => Some(path),
            _ => {
                err.add(self.pos, "expected distance");
                None
            }
        }
    }

    pub fn into_canvas_distance(self, err: &mut Error) -> Option<f64> {
        match self.value {
            ExprVal::Distance(value) => {
                if value.world.is_some() {
                    err.add(
                        self.pos,
                        "distance cannot have a world component"
                    );
                    None
                }
                else {
                    Some(value.canvas.unwrap_or_default())
                }
            }
            _ => {
                err.add(self.pos, "expected distance");
                None
            }
        }
    }

    pub fn into_number(self, err: &mut Error) -> Option<Number> {
        match self.value {
            ExprVal::Number(value) => Some(value),
            _ => {
                err.add(self.pos, "expected number");
                None
            }
        }
    }

    pub fn into_path<'s>(
        self, scope: &'s Scope, err: &mut Error
    ) -> Option<&'s ImportPath> {
        match self.value {
            ExprVal::Path(path) => Some(scope.paths().get(path).unwrap()),
            _ => {
                err.add(self.pos, "expected path segment");
                None
            }
        }
    }

    pub fn into_text(self, err: &mut Error) -> Option<String> {
        match self.value {
            ExprVal::Text(val) => Some(val),
            _ => {
                err.add(self.pos, "expected text");
                None
            }
        }
    }
}


//------------ ExprVal -------------------------------------------------------

/// The value of a resolved expression.
///
/// This has a shorthand name because we are going to type it a lot.
#[derive(Clone, Debug)]
pub enum ExprVal {
    Distance(Distance),
    Range(Range),
    Number(Number),
    Text(String),
    Color(Color),
    ContourRule(ContourRule),
    SymbolRule(SymbolRule),
    Path(usize),
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
    pub fn into_u8(self) -> Result<u8, String> {
        match self {
            Number::Int(val) => {
                val.try_into().map_err(|_| "value out of range".into())
            }
            Number::Float(_) => {
                Err("integer number expected".into())
            }
        }
    }

    pub fn into_f64(self) -> f64 {
        match self {
            Number::Int(val) => val.into(),
            Number::Float(val) => val
        }
    }
}


//------------ ArgumentList --------------------------------------------------

/// Evaluated arguments of a function.
#[derive(Clone, Debug)]
pub struct ArgumentList {
    /// The list of arguments.
    ///
    /// The first element is the keyword if this is a keyword argument.
    /// Otherwise it is a positional argument.
    arguments: Vec<(Option<String>, Expression)>,

    /// The position of argument list.
    pos: ast::Pos,
}

impl ArgumentList {
    fn new(pos: ast::Pos) -> Self {
        ArgumentList {
            arguments: Vec::new(),
            pos
        }
    }

    /// Converts the list into an iterator over positional arguments.
    ///
    /// Returns an error if there are any keyword arguments.
    pub fn into_positional(
        self, err: &mut Error
    ) -> Option<impl Iterator<Item = Expression>> {
        for item in &self.arguments {
            if item.0.is_some() {
                err.add(item.1.pos, "expected positiona arguments only");
                return None
            }
        }
        Some(self.arguments.into_iter().map(|item| item.1))
    }

    /// Converts the list into an iterator over `len`  positional arguments.
    ///
    /// Returns an error if there are any keyword arguments or if there is
    /// the wrong number of arguments.
    pub fn into_n_positional(
        self, len: usize, err: &mut Error
    ) -> Option<impl Iterator<Item = Expression>> {
        if self.arguments.len() != len {
            err.add(
                self.pos,
                format!("expected exactly {} arguments", len)
            );
            return None
        }
        self.into_positional(err)
    }

    /// Converts the list into its sole positional argument or errors.
    pub fn single_positional(self, err: &mut Error) -> Option<Expression> {
        if self.arguments.len() != 1 {
            err.add(self.pos, "expected a single positional argument");
            return None
        }
        if self.arguments[0].0.is_some() {
            err.add(self.pos, "expected a single positional argument");
            return None
        }
        Some(self.arguments.into_iter().next().unwrap().1)
    }

    /// Converts the arguments into keyword arguments.
    ///
    /// If there are any positional arguments, returns an error.
    pub fn into_keyword(
        self, err: &mut Error
    ) -> Option<KeywordArguments> {
        for item in &self.arguments {
            if item.0.is_none() {
                err.add(item.1.pos, "expected keyword arguments only");
                return None
            }
        }
        Some(KeywordArguments {
            args: self.arguments
                .into_iter()
                .map(|item| (item.0.unwrap(), item.1))
                .collect(),
            pos: self.pos
        })
    }
}

impl IntoIterator for ArgumentList {
    type Item = (Option<String>, Expression);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.arguments.into_iter()
    }
}


//------------ KeywordArguments ----------------------------------------------

#[derive(Clone, Debug)]
pub struct KeywordArguments {
    args: HashMap<String, Expression>,
    pos: ast::Pos,
}

impl KeywordArguments {
    pub fn take(
        &mut self, keyword: &str, err: &mut Error
    ) -> Option<Expression> {
        match self.args.remove(keyword) {
            Some(res) => Some(res),
            None => {
                err.add(
                    self.pos,
                    format!("missing keyword argument '{}'", keyword)
                );
                None
            }
        }
    }

    pub fn take_opt(&mut self, keyword: &str) -> Option<Expression> {
        self.args.remove(keyword) 
    }

    pub fn check_empty(self, err: &mut Error) -> Option<()> {
        if self.args.is_empty() {
            Some(())
        }
        else {
            err.add(self.pos, "unrecognized keyword arguments");
            None
        }
    }
}


//============ Evaluations for AST Types =====================================
//
// These are here in alphabetical order.

impl ast::ArgumentList {
    fn eval(self, scope: &Scope, err: &mut Error) -> Option<ArgumentList> {
        let mut good = true;
        let mut res = ArgumentList { arguments: Vec::new(), pos: self.pos };
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

    fn eval_symbol_rule(
        self, scope: &Scope, err: &mut Error
    ) -> Option<SymbolRule> {
        self.eval(scope, err).and_then(|expr| expr.into_symbol_rule(err))
    }

    fn eval_path<'s>(
        self, scope: &'s Scope, err: &mut Error
    ) -> Option<&'s ImportPath> {
        self.eval(scope, err).and_then(|expr| expr.into_path(scope, err))
    }
}

impl ast::Function {
    fn eval(self, scope: &Scope, err: &mut Error) -> Option<ExprVal> {
        let name = self.name.eval();
        let args = match self.args {
            Some(args) => args.eval(scope, err)?,
            None => ArgumentList::new(self.pos)
        };
        functions::eval(self.pos, name, args, scope, err)
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

impl ast::Location {
    fn eval(
        self, path: &ImportPath, err: &mut Error
    ) -> Option<(u32, Distance)> {
        let name = self.name.eval();
        let name = match path.get_named(&name) {
            Some(name) => Some(name),
            None => {
                err.add(
                    self.pos,
                    format!("unresolved path point '{}'", name)
                );
                None
            }
        };
        let distance = match self.distance {
            Some((addsub, distance)) => {
                let distance = distance.eval(err)?;
                match addsub {
                    ast::AddSub::Add => distance,
                    ast::AddSub::Sub => -distance,
                }
            }
            None => Distance::default()
        };
        let name = name?;
        // Segment numbers are the _end_ of the segment.
        Some((name + 1, distance))
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

impl ast::Offset {
    fn eval(self, err: &mut Error) -> Option<Distance> {
        let distance = self.distance.eval(err)?;
        match self.direction {
            ast::Direction::Left => Some(distance),
            ast::Direction::Right => Some(-distance),
        }
    }
}

impl ast::Path {
    fn eval(self, scope: &mut Scope, err: &mut Error) -> Option<Path> {
        let mut path = self.first.eval(scope, err).map(Path::new);
        for (conn, expr) in self.others {
            let (post, pre) = conn.tension();
            if let Some(seg) = expr.eval(scope, err) {
                if let Some(path) = path.as_mut() {
                    path.push(post, pre, seg)
                }
            }
        }
        path
    }
}

impl ast::Position {
    fn eval(self, scope: &mut Scope, err: &mut Error) -> Option<Position> {
        let path = self.path.eval_path(scope, err)?;

        let location = self.location.eval(path, err);
        let offset = match self.offset {
            Some(val) => Some(val.eval(err)?),
            None => None
        };
        let rotation = self.rotation.map(|r| r.eval_float());
        let (node, distance) = location?;
        Some(Position::eval(path.path(), node, distance, offset, rotation))
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

impl ast::Segment {
    fn eval(
        self, scope: &mut Scope, err: &mut Error
    ) -> Option<path::Subpath> {
        let path = self.path.eval_path(scope, err)?;

        let location = match self.location {
            Some(location) => {
                let start = location.0.eval(path, err);
                let end = location.1.eval(path, err);
                match (start, end) {
                    (Some(start), Some(end)) => Some(Some((start, end))),
                    _ => None
                }
            }
            None => Some(None)
        };
        let offset = match self.offset {
            Some(val) => val.eval(err).map(Some),
            None => Some(None)
        };
        let location = location?;
        let offset = offset?;

        Some(match location {
            Some((start, end)) => {
                path::Subpath::eval(
                    path.path(),
                    start.0, start.1, end.0, end.1,
                    offset
                )
            }
            None => {
                path::Subpath::eval_full(
                    path.path(),
                    offset
                )
            }
        })
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
            ast::Statement::Symbol(symbol) => {
                symbol.eval(scope, features, err)
            }
        }
    }
}

impl ast::StatementList {
    pub fn eval_all(
        self, scope: &mut Scope, features: &mut FeatureSet
    ) -> Result<(), Error> {
        let mut err = Error::default();
        self.eval(scope, features, &mut err);
        err.check()
    }

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

impl ast::Symbol {
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
        let rule = self.rule.eval_symbol_rule(scope, err);
        let position = self.position.eval(scope, err);

        // If we don’t have a detail, complain.
        let detail = match params.detail {
            Some(detail) => detail,
            None => {
                err.add(self.pos, "'detail' rendering parameter not yet set");
                return
            }
        };

        // If we have both a rule and a position, create the feature.
        if let (Some(rule), Some(position)) = (rule, position) {
            features.insert(
                Symbol::new(position, rule),
                detail,  params.layer   
            )
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
                    Some(self.number.eval_float() * factor), None
                ))
            }
        }
        for (unit, factor) in units::CANVAS_DISTANCES {
            if self.unit == unit {
                return Some(Distance::new(
                    None, Some(self.number.eval_float() * factor)
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

    pub fn check(self) -> Result<(), Self> {
        if self.errors.is_empty() {
            Ok(())
        }
        else {
            Err(self)
        }
    }

    pub fn iter<'a>(
        &'a self
    ) -> impl Iterator<Item = (ast::Pos, &'a str)> + 'a {
        self.errors.iter().map(|item| (item.0, item.1.as_ref()))
    }
}

