use std::ops;
use std::cmp::Ordering;
use std::convert::TryInto;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use crate::features::label;
use crate::features::{
    //Color, ContourRule, Distance,
    FeatureSet,
    //SymbolRule,
};
use crate::features::path::{Distance, Path, Position, Subpath};
use crate::import::Failed;
use crate::import::path::{ImportPath, PathSet};
use crate::library::units;
use crate::library::{Function, Procedure};
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

    pub fn params(&self) -> &RenderParams {
        &self.params
    }
}


//------------ RenderParams --------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct RenderParams {
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
        match target {
            "detail" => self.update_detail(value, err),
            "layer" => self.update_layer(value, err),
            _ => {
                err.add(pos, format!("unknown render param {}", target));
            }
        }
    }

    fn update_detail(
        &mut self,
        value: Expression,
        err: &mut Error
    ) {
        match value.value {
            ExprVal::Number(val) => {
                match val.into_u8() {
                    Ok(val) => self.detail = Some((val, val)),
                    Err(_) => err.add(value.pos, "expected 8-bit integer"),
                }
            }
            ExprVal::List(val) => {
                if val.len() != 2 {
                    err.add(value.pos, "expected number or pair of numbers");
                    return;
                }
                let mut val = val.into_iter();
                let left = match val.next().unwrap().into_u8(err) {
                    Ok(left) => left.0,
                    Err(_) => return,
                };
                let right = match val.next().unwrap().into_u8(err) {
                    Ok(right) => right.0,
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
        value: Expression,
        err: &mut Error
    ) {
        match value.value {
            ExprVal::Number(val) => {
                self.layer = val.into_f64();
            }
            _ => err.add(value.pos, "expected number"),
        }
    }

    pub fn detail(
        &self, pos: ast::Pos, err: &mut Error
    ) -> Result<(u8, u8), Failed> {
        match self.detail {
            Some(detail) => Ok(detail),
            None => {
                err.add(pos, "no detail level selected yet");
                Err(Failed)
            }
        }
    }

    pub fn layer(&self) -> f64 {
        self.layer
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

    pub fn into_layout(
        self, err: &mut Error
    ) -> Result<(label::Layout, ast::Pos), Failed> {
        match self.value {
            ExprVal::Layout(val) => Ok((val, self.pos)),
            _ => {
                err.add(self.pos, "expected layout");
                Err(Failed)
            }
        }
    }

    pub fn into_layout_or_text(
        self, err: &mut Error
    ) -> Result<(Result<label::Layout, String>, ast::Pos), Failed> {
        match self.value {
            ExprVal::Layout(val) => Ok((Ok(val), self.pos)),
            ExprVal::Text(val) => Ok((Err(val), self.pos)),
            _ => {
                err.add(self.pos, "expected layout or text");
                Err(Failed)
            }
        }
    }

    pub fn into_based_layout(
        self, font: label::Font, err: &mut Error
    ) -> Result<(label::Layout, ast::Pos), Failed> {
        match self.value {
            ExprVal::Layout(mut val) => {
                val.rebase_font(font);
                Ok((val, self.pos))
            }
            ExprVal::Text(val) => {
                Ok((label::Layout::span(font, val), self.pos))
            }
            _ => {
                err.add(self.pos, "expected layout or text");
                Err(Failed)
            }
        }

    }

    pub fn into_number(
        self, err: &mut Error
    ) -> Result<(Number, ast::Pos), Failed> {
        match self.value {
            ExprVal::Number(val) => Ok((val, self.pos)),
            _ => {
                err.add(self.pos, "expected number");
                Err(Failed)
            }
        }
    }

    pub fn into_u8(
        self, err: &mut Error
    ) -> Result<(u8, ast::Pos), Failed> {
        let (val, pos) = self.into_number(err)?;
        match val.into_u8() {
            Ok(val) => Ok((val, pos)),
            Err(msg) => {
                err.add(pos, msg);
                Err(Failed)
            }
        }
    }

    pub fn into_path(
        self, err: &mut Error
    ) -> Result<(Path, ast::Pos), Failed> {
        match self.value {
            ExprVal::Path(val) => Ok((val, self.pos)),
            _ => {
                err.add(self.pos, "expected path");
                Err(Failed)
            }
        }
    }

    pub fn into_position(
        self, err: &mut Error
    ) -> Result<(Position, ast::Pos), Failed> {
        match self.value {
            ExprVal::Position(val) => Ok((val, self.pos)),
            _ => {
                err.add(self.pos, "expected position");
                Err(Failed)
            }
        }
    }

    pub fn into_symbol(
        self, err: &mut Error
    ) -> Result<(String, ast::Pos), Failed> {
        match self.value {
            ExprVal::SymbolSet(set) => {
                if set.len() != 1 {
                    err.add(self.pos, "expected exactly one symbol");
                    Err(Failed)
                }
                else {
                    Ok((set.into_iter().next().unwrap(), self.pos))
                }
            }
            _ => {
                err.add(self.pos, "expected symbol");
                Err(Failed)
            }
        }
    }

    pub fn into_symbol_set(
        self, err: &mut Error
    ) -> Result<(SymbolSet, ast::Pos), Failed> {
        match self.value {
            ExprVal::SymbolSet(val) => Ok((val, self.pos)),
            _ => {
                err.add(self.pos, "expected symbol set");
                Err(Failed)
            }
        }
    }

    pub fn into_text(
        self, err: &mut Error
    ) -> Result<(String, ast::Pos), Failed> {
        match self.value {
            ExprVal::Text(val) => Ok((val, self.pos)),
            _ => {
                err.add(self.pos, "expected text");
                Err(Failed)
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
    ImportPath(usize),
    Layout(label::Layout),
    List(Vec<Expression>),
    Number(Number),
    Partial(Partial),
    Path(Path),
    Position(Position),
    SymbolSet(SymbolSet),
    Text(String),
    Vector((Distance, Distance)),
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


//------------ Partial -------------------------------------------------------

/// A partially applied function or procedure.
///
/// When a function is called in a let expression with an incomplete set of
/// arguments or a procedure is called, its execution is delayed and a partial
/// expression is retained instead. This can be called again supplying the
/// missing arguments or adding more in another let expression.
#[derive(Clone, Debug)]
pub struct Partial {
    /// The function or procedure to eventually execute.
    ///
    /// We abuse `Result` here as an either type.
    function: Result<Function, Procedure>,

    /// The arguments of the function.
    ///
    /// This is updated every time the partial function is evaluated again.
    args: ArgumentList,

    /// The position of the function.
    pos: ast::Pos,
}

impl Partial {
    fn new(
        name: &str, args: ArgumentList, pos: ast::Pos, err: &mut Error
    ) -> Result<Self, Failed> {
        let function = Function::lookup(name).map(Ok).or_else(|| {
            Procedure::lookup(name).map(Err)
        }).or_else(|| {
            err.add(
                pos,
                "expected function or function name"
            );
            None
        });
        match function {
            Some(function) => Ok(Partial { function, args, pos }),
            None => Err(Failed)
        }
    }

    fn extend(&mut self, args: ArgumentList, pos: ast::Pos) {
        self.args.extend(args);
        self.pos = pos;
    }

    fn eval_expr(
        mut self, scope: &Scope, err: &mut Error
    ) -> Result<ExprVal, Failed> {
        if let Ok(function) = self.function {
            match function.eval(self.args, scope, err) {
                Ok(res) => Ok(res),
                Err(Ok(args)) => {
                    self.args = args;
                    Ok(ExprVal::Partial(self))
                }
                Err(Err(_)) => Err(Failed)
            }
        }
        else {
            Ok(ExprVal::Partial(self))
        }
    }

    fn eval_procedure(
        self, 
        scope: &mut Scope,
        features: &mut FeatureSet,
        err: &mut Error
    ) -> Result<(), Failed> {
        match self.function {
            Ok(_) => {
                err.add(self.pos, "expected procedure");
                Err(Failed)
            }
            Err(procedure) => {
                procedure.eval(self.pos, self.args, scope, features, err)
            }
        }
    }
}


//------------ SymbolSet -----------------------------------------------------

/// A set of symbols.
//
//  XXX This should probably be moved.
pub type SymbolSet = HashSet<String>;


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

    pub fn positional(&self) -> &[Expression] {
        &self.positional
    }

    pub fn into_positionals(
        self,
        err: &mut Error,
        test: impl FnOnce(&Self, &mut Error) -> Result<bool, Failed>
    ) -> Result<Vec<Expression>, Result<Self, Failed>> {
        if !self.keyword.is_empty() {
            err.add(self.pos, "expected positional arguments only");
            Err(Err(Failed))
        }
        else {
            match test(&self, err) {
                Ok(true) => Ok(self.positional),
                Ok(false) => Err(Ok(self)),
                Err(_) => Err(Err(Failed))
            }
        }
    }

    /// Returns exactly n positional arguments.
    ///
    /// Fails if there are keyword arguments or more than n positional
    /// arguments. Returns `Ok(None)` if there are less than n positional
    /// arguments.
    pub fn into_n_positionals(
        self, n: usize, err: &mut Error
    ) -> Result<Vec<Expression>, Result<ArgumentList, Failed>> {
        self.into_positionals(err, |args, err| {
            match n.cmp(&args.positional().len()) {
                Ordering::Greater => Ok(false),
                Ordering::Equal => Ok(true),
                Ordering::Less => {
                    err.add(
                        args.pos(),
                        format!("expected exactly {} positional arguments", n)
                    );
                    Err(Failed)
                }
            }
        })
    }

    /// Returns the only positional argument.
    pub fn into_sole_positional(
        self, err: &mut Error
    ) -> Result<Expression, Result<ArgumentList, Failed>> {
        self.into_n_positionals(1, err).map(|mut res| res.pop().unwrap())
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


//------------ PositionOffset ------------------------------------------------

/// The combined offset of a position.
#[derive(Clone, Copy, Debug, Default)]
struct PositionOffset {
    sideways: Distance,
    shift: (Distance, Distance),
    rotation: Option<f64>,
}

impl PositionOffset {
    fn sideways(sideways: Distance) -> Self {
        PositionOffset {
            sideways,
            shift: (Default::default(), Default::default()),
            rotation: None,
        }
    }

    fn shift(shift: (Distance, Distance)) -> Self {
        PositionOffset {
            sideways: Default::default(),
            shift,
            rotation: None,
        }
    }

    fn rotation(rotation: f64) -> Self {
        PositionOffset {
            sideways: Default::default(),
            shift: (Default::default(), Default::default()),
            rotation: Some(rotation),
        }
    }

}

impl ops::Add for PositionOffset {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        PositionOffset {
            sideways: self.sideways + other.sideways,
            shift: (
                self.shift.0 + other.shift.0,
                self.shift.1 + other.shift.1
            ),
            rotation: match (self.rotation, other.rotation) {
                (Some(l), Some(r)) => Some(l + r),
                (Some(l), None) => Some(l),
                (None, Some(r)) => Some(r),
                (None, None) => None
            },
        }
    }
}


//============ Evaluations for AST Types =====================================

//------------ Statements ----------------------------------------------------

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

impl ast::Statement {
    pub fn eval(
        self,
        scope: &mut Scope,
        features: &mut FeatureSet,
        err: &mut Error
    ) {
        match self {
            ast::Statement::Let(stm) => stm.eval(scope, err),
            ast::Statement::NoOp(_) => { },
            ast::Statement::Procedure(stm) => {
                let _ = stm.eval(scope, features, err);
            }
            ast::Statement::With(stm) => stm.eval(scope, features, err),
        }
    }
}

impl ast::Let {
    fn eval(self, scope: &mut Scope, err: &mut Error) {
        for assignment in self.assignments.assignments {
            let target = assignment.target.eval();
            let expression = match assignment.expression.eval(scope, err) {
                Ok(expression) => expression,
                Err(_) => continue,
            };
            scope.set_var(target, expression.value);
        }
    }
}

impl ast::Procedure {
    fn eval(
        self, 
        scope: &mut Scope,
        features: &mut FeatureSet,
        err: &mut Error
    ) -> Result<(), Failed> {
        let args = self.args.eval(scope, err)?;
        
        match scope.get_var(self.ident.as_ref()) {
            Some(ExprVal::Partial(mut func)) => {
                func.extend(args, self.pos);
                func.eval_procedure(scope, features, err)
            }
            Some(_) => {
                err.add(
                    self.ident.pos,
                    "expected procedure variable"
                );
                Err(Failed)
            }
            None => {
                match Procedure::lookup(self.ident.as_ref()) {
                    Some(procedure) => {
                        procedure.eval(
                            self.pos, args, scope, features, err
                        )
                    }
                    None => {
                        err.add(
                            self.ident.pos,
                            format!("unresolved variable '{}'", self.ident)
                        );
                        Err(Failed)
                    }
                }
            }
        }
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


//------------ Assignments and Arguments -------------------------------------

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
                Ok(expression) => expression,
                Err(_) => continue,
            };
            params.update(&target, expression, item.pos, err);
        }
    }
}

impl ast::ArgumentList {
    fn eval(
        self, scope: &Scope, err: &mut Error
    ) -> Result<ArgumentList, Failed> {
        let mut good = true;
        let mut res = ArgumentList::new(self.pos);
        for argument in self.arguments {
            match argument {
                ast::Argument::Keyword(assignment) => {
                    match assignment.expression.eval(scope, err) {
                        Ok(expr) => {
                            res.keyword.insert(assignment.target, expr);
                        }
                        Err(_) => good = false,
                    }
                }
                ast::Argument::Pos(expr) => {
                    match expr.eval(scope, err) {
                        Ok(expr) => res.positional.push(expr),
                        Err(_) => good = false,
                    }
                }
            }
        }
        if good {
            Ok(res)
        }
        else {
            Err(Failed)
        }
    }
}


//------------ Expressions ---------------------------------------------------

impl ast::Expression {
    fn eval(
        self, scope: &Scope, err: &mut Error
    ) -> Result<Expression, Failed> {
        if self.connected.is_empty() {
            Ok(Expression::new(self.first.eval_simple(scope, err)?, self.pos))
        }
        else {
            let mut path = Path::new(self.first.eval_subpath(scope, err)?);
            for (conn, frag) in self.connected {
                let (post, pre) = conn.tension();
                path.push(post, pre, frag.eval_subpath(scope, err)?);
            }
            Ok(Expression::new(ExprVal::Path(path), self.pos))
        }
    }
}

impl ast::Fragment {
    fn eval_simple(
        self, scope: &Scope, err: &mut Error
    ) -> Result<ExprVal, Failed> {
        match self {
            ast::Fragment::Complex(frag) => frag.eval_expr(scope, err),
            ast::Fragment::List(frag) => frag.eval_expr(scope, err),
            ast::Fragment::Vector(frag) => frag.eval_expr(scope, err),
            ast::Fragment::Atom(frag) => frag.eval(scope, err),
        }
    }

    fn eval_subpath(
        self, scope: &Scope, err: &mut Error
    ) -> Result<Subpath, Failed> {
        match self {
            ast::Fragment::Complex(frag) => frag.eval_subpath(scope, err),
            _ => {
                err.add(self.pos(), "expected section");
                Err(Failed)
            }
        }
    }
}

impl ast::Complex {
    fn eval_expr(
        self, scope: &Scope, err: &mut Error
    ) -> Result<ExprVal, Failed> {
        match self.section {
            Some(section) => {
                let base = self.external.eval_import_path(scope, err)?;
                section.eval_expr(base, scope, err)
            }
            None => self.external.eval_expr(scope, err),
        }
    }

    fn eval_subpath(
        self, scope: &Scope, err: &mut Error
    ) -> Result<Subpath, Failed> {
        let base = self.external.eval_import_path(scope, err);
        match self.section {
            Some(section) => section.eval_subpath(base?, scope, err),
            None => {
                err.add(self.pos, "expected section");
                Err(Failed)
            }
        }
    }
}

impl ast::External {
    fn eval_expr(
        self, scope: &Scope, err: &mut Error
    ) -> Result<ExprVal, Failed> {
        match self.args {
            Some(args) => {
                let args = args.eval(scope, err)?;
                let func = match scope.get_var(&self.ident.ident) {
                    Some(ExprVal::Partial(mut func)) => {
                        func.extend(args, self.pos);
                        Ok(func)
                    }
                    Some(_) => {
                        err.add(
                            self.pos,
                            "expected partial function"
                        );
                        Err(Failed)
                    }
                    None => {
                        Partial::new(
                            &self.ident.ident, args, self.pos, err
                        )
                    }
                }?;

                func.eval_expr(scope, err)
            }
            None => {
                match scope.get_var(&self.ident.ident) {
                    Some(val) => Ok(val),
                    None => {
                        err.add(
                            self.pos,
                            format!(
                                "undefined variable '{}'",
                                self.ident.ident
                            )
                        );
                        Err(Failed)
                    }
                }
            }
        }
    }

    fn eval_import_path<'s>(
        self, scope: &'s Scope, err: &mut Error
    ) -> Result<&'s ImportPath, Failed> {
        let pos = self.pos;
        match self.eval_expr(scope, err)? {
            ExprVal::ImportPath(idx) => {
                Ok(scope.paths().get(idx).unwrap())
            }
            _ => {
                err.add(pos, "expected import path");
                Err(Failed)
            }
        }
    }
}

impl ast::Section {
    fn eval_subpath(
        self, base: &ImportPath, scope: &Scope, err: &mut Error
    ) -> Result<Subpath, Failed> {
        let start = self.start.eval(&base, scope, err);
        let end = match self.end {
            Some(end) => end.eval(&base, scope, err),
            None => {
                err.add(self.pos, "expected subpath section");
                return Err(Failed)
            }
        };
        let offset = self.offset.into_iter().fold(
            Ok(Distance::default()),
            |res, item| {
                let item = item.eval_subpath(scope, err);
                if let (Ok(res), Ok(item)) = (res, item) {
                    Ok(res + item)
                }
                else {
                    Err(Failed)
                }
            }
        )?;
        let start = start?;
        let end = end?;
        Ok(Subpath::eval(
            base.path(), start.0, start.1, end.0, end.1, offset
        ))
    }

    fn eval_position(
        self, base: &ImportPath, scope: &Scope, err: &mut Error
    ) -> Result<Position, Failed> {
        if self.end.is_some() {
            err.add(self.pos, "expected position section");
            return Err(Failed)
        }
        let start = self.start.eval(&base, scope, err);
        let offset = self.offset.into_iter().fold(
            Ok(PositionOffset::default()),
            |res, item| {
                let item = item.eval_position(scope, err);
                if let (Ok(res), Ok(item)) = (res, item) {
                    Ok(res + item)
                }
                else {
                    Err(Failed)
                }
            }
        )?;
        let start = start?;
        Ok(Position::eval(
            base.path(),
            start.0, start.1,
            offset.sideways, offset.shift, offset.rotation
        ))
    }

    fn eval_expr(
        self, base: &ImportPath, scope: &Scope, err: &mut Error
    ) -> Result<ExprVal, Failed> {
        if self.end.is_some() {
            self.eval_subpath(base, scope, err).map(|subpath| {
                ExprVal::Path(Path::new(subpath))
            })
        }
        else {
            self.eval_position(base, scope, err).map(ExprVal::Position)
        }
    }
}

impl ast::Location {
    fn eval(
        self, base: &ImportPath, scope: &Scope, err: &mut Error
    ) -> Result<(u32, Distance), Failed> {
        let node = match base.get_named(self.node.as_ref()) {
            Some(node) => Ok(node),
            None => {
                err.add(
                    self.node.pos,
                    format!("unresolved path node '{}'", self.node.as_ref())
                );
                 Err(Failed)
            }
        };
        let distance = self.distance.into_iter().fold(
            Ok(Distance::default()),
            |res, item| {
                match (res, item.eval(scope, err)) {
                    (Ok(res), Ok(item)) => Ok(res + item),
                    _ => Err(Failed),
                }
            }
        )?;
        let node = node?;
        Ok((node, distance))
    }
}

impl ast::Distance {
    fn eval(
        self, scope: &Scope, err: &mut Error
    ) -> Result<Distance, Failed> {
        let value = self.value.eval(scope, err)?;
        match self.op {
            ast::AddSub::Add => Ok(value),
            ast::AddSub::Sub => Ok(-value),
        }
    }
}

impl ast::Offset {
    fn eval_subpath(
        self, scope: &Scope, err: &mut Error
    ) -> Result<Distance, Failed> {
        match self {
            ast::Offset::Sideways(sideways) => {
                let value = sideways.value.eval(scope, err)?;
                match sideways.direction {
                    ast::Direction::Left => Ok(value),
                    ast::Direction::Right => Ok(-value),
                }
            }
            ast::Offset::Shift(shift) => {
                err.add(shift.pos, "expected sideways offset");
                Err(Failed)
            }
            ast::Offset::Angle(angle) => {
                err.add(angle.pos, "expected sideways offset");
                Err(Failed)
            }
        }
    }

    fn eval_position(
        self, scope: &Scope, err: &mut Error
    ) -> Result<PositionOffset, Failed> {
        match self {
            ast::Offset::Sideways(sideways) => {
                let value = sideways.value.eval(scope, err)?;
                let value = match sideways.direction {
                    ast::Direction::Left => value,
                    ast::Direction::Right => -value,
                };
                Ok(PositionOffset::sideways(value))
            }
            ast::Offset::Shift(shift) => {
                let (x, y) = shift.value.eval(scope, err)?;
                let value = match shift.op {
                    ast::AddSub::Add => (x, y),
                    ast::AddSub::Sub => (-x, -y),
                };
                Ok(PositionOffset::shift(value))
            }
            ast::Offset::Angle(angle) => {
                Ok(PositionOffset::rotation(angle.value.eval_float()))
            }
        }
    }
}

impl ast::List {
    fn eval_expr(
        self, scope: &Scope, err: &mut Error
    ) -> Result<ExprVal, Failed> {
        self.content.into_iter().fold(Ok(Vec::new()), |res, expr| {
            match (res, expr.eval(scope, err)) {
                (Ok(mut res), Ok(expr)) => {
                    res.push(expr);
                    Ok(res)
                }
                _ => Err(Failed)
            }
        }).map(ExprVal::List)
    }
}

impl ast::Vector {
    fn eval_expr(
        self, scope: &Scope, err: &mut Error
    ) -> Result<ExprVal, Failed> {
        self.eval(scope, err).map(|res| ExprVal::Vector(res))
    }

    fn eval(
        self, scope: &Scope, err: &mut Error
    ) -> Result<(Distance, Distance), Failed> {
        let x = self.x.eval(scope, err);
        let y = self.y.eval(scope, err)?;
        Ok((x?, y))
    }
}

impl ast::Atom {
    fn eval(
        self, scope: &Scope, err: &mut Error
    ) -> Result<ExprVal, Failed> {
        match self {
            ast::Atom::Number(atom) => atom.eval_expr(scope, err),
            ast::Atom::SymbolSet(atom) => atom.eval_expr(scope, err),
            ast::Atom::Text(atom) => atom.eval_expr(scope, err),
            ast::Atom::UnitNumber(atom) => atom.eval_expr(scope, err),
        }
    }
}

impl ast::Number {
    fn eval_expr(
        self, _scope: &Scope, _err: &mut Error
    ) -> Result<ExprVal, Failed> {
        if let Ok(value) = i32::from_str(&self.value) {
            Ok(ExprVal::Number(Number::Int(value)))
        }
        else {
            Ok(ExprVal::Number(
                Number::Float(f64::from_str(&self.value).unwrap())
            ))
        }
    }

    fn eval_float(self) -> f64 {
        f64::from_str(&self.value).unwrap()
    }
}

impl ast::SymbolSet {
    fn eval_expr(
        self, _scope: &Scope, _err: &mut Error
    ) -> Result<ExprVal, Failed> {
        Ok(ExprVal::SymbolSet(
            self.symbols.into_iter().map(Into::into).collect()
        ))
    }
}

impl ast::Text {
    fn eval_expr(
        self, _scope: &Scope, _err: &mut Error
    ) -> Result<ExprVal, Failed> {
        let mut res = self.first.content;
        self.others.into_iter().for_each(|val| res.push_str(&val.content));
        Ok(ExprVal::Text(res))
    }
}

impl ast::UnitNumber {
    fn eval_expr(
        self, scope: &Scope, err: &mut Error
    ) -> Result<ExprVal, Failed> {
        self.eval(scope, err).map(|res| ExprVal::Distance(res))
    }

    fn eval(
        self, _scope: &Scope, err: &mut Error
    ) -> Result<Distance, Failed> {
        for (unit, factor) in units::WORLD_DISTANCES {
            if self.unit == unit {
                return Ok(Distance::new(
                    Some(self.number.eval_float() * factor), None
                ))
            }
        }
        for (unit, factor) in units::CANVAS_DISTANCES {
            if self.unit == unit {
                return Ok(Distance::new(
                    None, Some(self.number.eval_float() * factor)
                ))
            }
        }
        err.add(self.pos, format!("unknown unit '{}'", self.unit));
        Err(Failed)
    }
}

impl ast::Identifier {
    fn eval(self) -> String {
        self.ident
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

