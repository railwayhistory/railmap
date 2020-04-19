use std::cmp::Ordering;
use std::convert::TryInto;
use std::collections::HashMap;
use std::str::FromStr;
use crate::features::{path, label};
use crate::features::{
    Color, Contour, ContourRule, Distance, FeatureSet, Label, Layout, Path,
    Position, Symbol, SymbolRule,
};
use crate::import::Failed;
use crate::import::functions::Function;
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

    pub fn to_align(&self, err: &mut Error) -> Result<label::Align, Failed> {
        match self.value {
            ExprVal::Align(value) => Ok(value),
            _ => {
                err.add(self.pos, "expected alignment");
                Err(Failed)
            }
        }
    }

    pub fn to_color(&self, err: &mut Error) -> Result<Color, Failed> {
        match self.value {
            ExprVal::Color(value) => Ok(value),
            _ => {
                err.add(self.pos, "expected color");
                Err(Failed)
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

    pub fn to_distance(&self, err: &mut Error) -> Option<Distance> {
        match self.value {
            ExprVal::Distance(path) => Some(path),
            _ => {
                err.add(self.pos, "expected distance");
                None
            }
        }
    }
    pub fn into_distance(self, err: &mut Error) -> Option<Distance> {
        self.to_distance(err)
    }

    pub fn to_canvas_distance(&self, err: &mut Error) -> Result<f64, Failed> {
        match self.value {
            ExprVal::Distance(value) => {
                if value.world.is_some() {
                    err.add(
                        self.pos,
                        "distance cannot have a world component"
                    );
                    Err(Failed)
                }
                else {
                    Ok(value.canvas.unwrap_or_default())
                }
            }
            _ => {
                err.add(self.pos, "expected distance");
                Err(Failed)
            }
        }
    }

    pub fn to_font(&self, err: &mut Error) -> Result<label::Font, Failed> {
        match self.value {
            ExprVal::Font(ref font) => Ok(font.clone()),
            _ => {
                err.add(self.pos, "expected font");
                Err(Failed)
            }
        }
    }

    pub fn to_layout(&self, err: &mut Error) -> Result<Layout, Failed> {
        match self.value {
            ExprVal::Layout(ref layout) => Ok(layout.clone()),
            _ => {
                err.add(self.pos, "expected layout");
                Err(Failed)
            }
        }
    }

    pub fn into_layout(self, err: &mut Error) -> Option<Layout> {
        match self.value {
            ExprVal::Layout(layout) => Some(layout),
            _ => {
                err.add(self.pos, "expected layout");
                None
            }
        }
    }

    pub fn to_number(&self, err: &mut Error) -> Result<Number, Failed> {
        match self.value {
            ExprVal::Number(value) => Ok(value),
            _ => {
                err.add(self.pos, "expected number");
                Err(Failed)
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

    pub fn into_symbol_rule(self, err: &mut Error) -> Option<SymbolRule> {
        match self.value {
            ExprVal::SymbolRule(rule) => Some(rule),
            _ => {
                err.add(self.pos, "expected symbol rule");
                None
            }
        }
    }

    pub fn to_text(&self, err: &mut Error) -> Result<&str, Failed> {
        match self.value {
            ExprVal::Text(ref val) => Ok(val),
            _ => {
                err.add(self.pos, "expected text");
                Err(Failed)
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
    Align(label::Align),
    Color(Color),
    ContourRule(ContourRule),
    Distance(Distance),
    Font(label::Font),
    Layout(label::Layout),
    Number(Number),
    PartialFunc(PartialFunc),
    Path(usize),
    Range(Range),
    SymbolRule(SymbolRule),
    Text(String),
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


//------------ PartialFunc ---------------------------------------------------

/// A partially applied function.
///
/// When a function is called in a let expression with an incomplete set of
/// arguments, its execution is delayed and a partial function expression is
/// retained instead. This can be called again supplying the missing arguments
/// or adding more in another let expression.
#[derive(Clone, Debug)]
pub struct PartialFunc {
    /// The function to eventually execute.
    function: Function,

    /// The arguments of the function.
    ///
    /// This is updated every time the partial function is evaluated again.
    args: ArgumentList,
}

impl PartialFunc {
    fn new(name: &str, args: ArgumentList) -> Option<Self> {
        Function::lookup(name).map(|function| {
            PartialFunc { function, args }
        })
    }

    fn eval(self, scope: &Scope, err: &mut Error) -> Option<ExprVal> {
        match self.function.eval(&self.args, scope, err) {
            Ok(Some(res)) => Some(res),
            Ok(None) => Some(ExprVal::PartialFunc(self)),
            Err(_) => None
        }
    }
}


//------------ ArgumentList --------------------------------------------------

/// Evaluated arguments of a function.
#[derive(Clone, Debug)]
pub struct ArgumentList {
    /// The positional arguments.
    positional: Vec<Expression>,

    /// The keyword arguments
    keyword: HashMap<ast::Identifier, Expression>,

    /// The start of this argument list in its source.
    pos: ast::Pos,
}

impl ArgumentList {
    fn new(pos: ast::Pos) -> Self {
        ArgumentList {
            positional: Vec::new(),
            keyword: HashMap::new(),
            pos
        }
    }

    fn extend(&mut self, args: Self) {
        self.positional.extend(args.positional.into_iter());
        self.keyword.extend(args.keyword.into_iter());
        self.pos = args.pos;
    }

    pub fn pos(&self) -> ast::Pos {
        self.pos
    }

    /// Returns the positional arguments.
    ///
    /// Fails if there are keyword arguments.
    pub fn positional_only(
        &self, err: &mut Error
    ) -> Result<&[Expression], Failed> {
        if !self.keyword.is_empty() {
            err.add(self.pos, "expected positional arguments only");
            Err(Failed)
        }
        else {
            Ok(&self.positional)
        }
    }

    /// Returns exactly n positional arguments.
    ///
    /// Fails if there are keyword arguments or more than n positional
    /// arguments. Returns `Ok(None)` if there are less than n positional
    /// arguments.
    pub fn n_positional_only(
        &self, n: usize, err: &mut Error
    ) -> Result<Option<&[Expression]>, Failed> {
        self.positional_only(err).and_then(|res| {
            match res.len().cmp(&n) {
                Ordering::Less => Ok(None),
                Ordering::Equal => Ok(Some(res)),
                Ordering::Greater => {
                    err.add(
                        self.pos,
                        format!("expected exactly {} positional arguments", n)
                    );
                    Err(Failed)
                }
            }
        })
    }

    /// Returns the only positional argument.
    pub fn sole_positional(
        &self, err: &mut Error
    ) -> Result<Option<&Expression>, Failed> {
        self.n_positional_only(1, err).map(|res| res.map(|res| &res[0]))
    }

    /// Checks that there are keyword arguments only.
    pub fn keyword_only(&self, err: &mut Error) -> Result<(), Failed> {
        if !self.positional.is_empty() {
            err.add(self.pos, "expected keyword arguments only");
            Err(Failed)
        }
        else {
            Ok(())
        }
    }

    /// Returns a keyword argument.
    pub fn get_keyword(&self, key: &str) -> Option<&Expression> {
        self.keyword.get(key)
    }
}


//============ Evaluations for AST Types =====================================
//
// These are here in alphabetical order.

impl ast::ArgumentList {
    fn eval(self, scope: &Scope, err: &mut Error) -> Option<ArgumentList> {
        let mut good = true;
        let mut res = ArgumentList::new(self.pos);
        for argument in self.arguments {
            match argument {
                ast::Argument::Keyword(assignment) => {
                    match assignment.expression.eval(scope, err) {
                        Some(expr) => {
                            res.keyword.insert(assignment.target, expr);
                        }
                        None => good = false,
                    }
                }
                ast::Argument::Pos(expr) => {
                    match expr.eval(scope, err) {
                        Some(expr) => res.positional.push(expr),
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

    fn eval_loc(self, neg_first: bool, err: &mut Error) -> Option<Distance> {
        let mut res = self.first.eval(err).map(|res| {
            if neg_first { -res } else { res }
        });
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

    fn eval_layout(
        self, scope: &Scope, err: &mut Error
    ) -> Option<Layout> {
        self.eval(scope, err).and_then(|expr| expr.into_layout(err))
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
        let args = match self.args {
            Some(args) => args.eval(scope, err)?,
            None => ArgumentList::new(self.name.pos)
        };

        let func = match scope.get_var(self.name.as_ref()) {
            Some(ExprVal::PartialFunc(mut func)) => {
                func.args.extend(args);
                func
            }
            Some(_) => {
                err.add(
                    self.name.pos,
                    "expected partial function or function name"
                );
                return None
            }
            None => match PartialFunc::new(self.name.as_ref(), args) {
                Some(func) => func,
                None => {
                    err.add(
                        self.name.pos,
                        "expected partial function or function name"
                    );
                    return None
                }
            }
        };
        func.eval(scope, err)
    }
}

impl ast::Identifier {
    fn eval(self) -> String {
        self.ident
    }
}

impl ast::Label {
    fn eval(
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
        let layout = self.layout.eval_layout(scope, err);
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
        if let (Some(layout), Some(position)) = (layout, position) {
            features.insert(
                Label::new(position, layout),
                detail,  params.layer   
            )
        }
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
                match addsub {
                    ast::AddSub::Add => distance.eval_loc(false, err)?,
                    ast::AddSub::Sub => distance.eval_loc(true, err)?,
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
            ast::Statement::Contour(stm) => stm.eval(scope, features, err),
            ast::Statement::Label(stm) => stm.eval(scope, features, err),
            ast::Statement::Let(stm) => stm.eval(scope, err),
            ast::Statement::Symbol(stm) => stm.eval(scope, features, err),
            ast::Statement::With(stm) => stm.eval(scope, features, err),
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

