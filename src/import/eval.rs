use std::ops;
use std::cmp::Ordering;
use std::convert::TryInto;
use std::collections::HashMap;
use std::str::FromStr;
use crate::render::color::Color;
use crate::render::feature::FeatureSet;
use crate::render::path::{Distance, Edge, Position, Subpath, Trace};
use crate::import::Failed;
use crate::import::path::{ImportPath, PathSet};
use crate::theme::Theme;
use super::ast;
use super::ast::ShortString;


//------------ Scope ---------------------------------------------------------

#[derive(Clone)]
pub struct Scope<'a, T: Theme> {
    theme: T,
    paths: &'a PathSet,
    variables: HashMap<ShortString, ExprVal<T>>,
    params: T::RenderParams,
}

impl<'a, T: Theme> Scope<'a, T> {
    pub fn new(theme: T, paths: &'a PathSet) -> Self {
        Scope {
            theme,
            paths,
            variables: HashMap::new(),
            params: Default::default(),
        }
    }

    pub fn paths(&self) -> &PathSet {
        &self.paths
    }

    pub fn set_var(&mut self, ident: ShortString, value: ExprVal<T>) {
        self.variables.insert(ident.clone(), value);
    }

    pub fn get_var(&self, ident: &str) -> Option<&ExprVal<T>> {
        self.variables.get(ident)
    }

    pub fn get_var_cloned(&self, ident: &str) -> Option<ExprVal<T>> {
        self.variables.get(ident).cloned()
    }

    pub fn params(&self) -> &T::RenderParams {
        &self.params
    }
}


//------------ Expression ----------------------------------------------------

/// An expression that has been evaluated for the current scope.
///
/// The variants are the concrete types that we have.
pub struct Expression<T: Theme> {
    pub value: ExprVal<T>,
    pub pos: ast::Pos
}

impl<T: Theme> Expression<T> {
    fn new(value: ExprVal<T>, pos: ast::Pos) -> Self {
        Expression { value, pos }
    }

    pub fn into_color(
        self, err: &mut Error
    ) -> Result<(Color, ast::Pos), Failed> {
        match self.value {
            ExprVal::Color(val) => Ok((val, self.pos)),
            _ => {
                err.add(self.pos, "expected distance");
                Err(Failed)
            }
        }
    }

    pub fn into_distance(
        self, err: &mut Error
    ) -> Result<(Distance, ast::Pos), Failed> {
        match self.value {
            ExprVal::Distance(val) => Ok((val, self.pos)),
            _ => {
                err.add(self.pos, "expected distance");
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

    pub fn into_f64(
        self, err: &mut Error
    ) -> Result<(f64, ast::Pos), Failed> {
        self.into_number(err).map(|(val, pos)| (val.into_f64(), pos))
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
    ) -> Result<(Trace, ast::Pos), Failed> {
        match self.value {
            ExprVal::Trace(val) => Ok((val, self.pos)),
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
    ) -> Result<(ShortString, ast::Pos), Failed> {
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
    ) -> Result<SymbolSet, Failed> {
        match self.value {
            ExprVal::SymbolSet(val) => Ok(val),
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

    pub fn into_vector(
        self, err: &mut Error
    ) -> Result<((Distance, Distance), ast::Pos), Failed> {
        match self.value {
            ExprVal::Vector(val) => Ok((val, self.pos)),
            _ => {
                err.add(self.pos, "expected a vector");
                Err(Failed)
            }
        }
    }
}

impl<T: Theme> Default for Expression<T> {
    fn default() -> Self {
        Expression {
            value: Default::default(),
            pos: Default::default(),
        }
    }
}

impl<T: Theme> Clone for Expression<T> {
    fn clone(&self) -> Self {
        Expression {
            value: self.value.clone(),
            pos: self.pos
        }
    }
}


//------------ ExprVal -------------------------------------------------------

/// The value of a resolved expression.
///
/// This has a shorthand name because we are going to type it a lot.
#[derive(Clone)]
pub enum ExprVal<T: Theme> {
    Null,
    Color(Color),
    Distance(Distance),
    ImportPath(usize),
    List(Vec<Expression<T>>),
    Number(Number),
    Partial(Partial<T>),
    Trace(Trace),
    Position(Position),
    SymbolSet(SymbolSet),
    Text(String),
    Vector((Distance, Distance)),
    Custom(T::CustomExpr),
}

impl<T: Theme> Default for ExprVal<T> {
    fn default() -> Self {
        ExprVal::Null
    }
}

impl<T: Theme> ExprVal<T> {
    pub fn custom(src: T::CustomExpr) -> Self {
        ExprVal::Custom(src)
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


//------------ Partial -------------------------------------------------------

/// A partially applied function or procedure.
///
/// When a function is called in a let expression with an incomplete set of
/// arguments or a procedure is called, its execution is delayed and a partial
/// expression is retained instead. This can be called again supplying the
/// missing arguments or adding more in another let expression.
pub struct Partial<T: Theme> {
    /// The function or procedure to eventually execute.
    ///
    /// We abuse `Result` here as an either type.
    function: Result<T::Function, T::Procedure>,

    /// The arguments of the function.
    ///
    /// This is updated every time the partial function is evaluated again.
    args: ArgumentList<T>,

    /// The position of the function.
    pos: ast::Pos,
}

impl<T: Theme> Partial<T> {
    fn new(
        name: &str, args: ArgumentList<T>, pos: ast::Pos,
        scope: &Scope<T>, err: &mut Error
    ) -> Result<Self, Failed> {
        let function = scope.theme.lookup_function(name).map(Ok).or_else(|| {
            scope.theme.lookup_procedure(name).map(Err)
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

    fn extend(&mut self, args: ArgumentList<T>, pos: ast::Pos) {
        self.args.extend(args);
        self.pos = pos;
    }

    fn eval_expr(
        mut self, scope: &Scope<T>, err: &mut Error
    ) -> Result<ExprVal<T>, Failed> {
        if let Ok(function) = self.function.as_ref() {
            match scope.theme.eval_function(function, self.args, scope, err) {
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
        scope: &mut Scope<T>,
        features: &mut FeatureSet<T>,
        err: &mut Error
    ) -> Result<(), Failed> {
        match self.function.as_ref() {
            Ok(_) => {
                err.add(self.pos, "expected procedure");
                Err(Failed)
            }
            Err(procedure) => {
                scope.theme.eval_procedure(
                    procedure, self.pos, self.args, scope, features, err
                )
            }
        }
    }
}

impl<T: Theme> Clone for Partial<T> {
    fn clone(&self) -> Self {
        Partial {
            function: self.function.clone(),
            args: self.args.clone(),
            pos: self.pos
        }
    }
}


//------------ SymbolSet -----------------------------------------------------

/// A set of symbols.
#[derive(Clone, Debug, Default)]
pub struct SymbolSet {
    set: HashMap<ShortString, Option<ast::Pos>>,
    pos: ast::Pos,
}

impl SymbolSet {
    fn new(set: ast::SymbolSet) -> Self {
        SymbolSet {
            set: HashMap::from_iter(set.symbols.into_iter().map(|item| {
                (item.ident.ident, Some(item.pos))
            })),
            pos: set.pos
        }
    }

    pub fn insert(&mut self, symbol: impl Into<ShortString>) -> bool {
        // XXX This inserts the symbol as used.
        self.set.insert(symbol.into(), None).is_none()
    }

    pub fn contains(&self, symbol: impl AsRef<str>) -> bool {
        self.set.contains_key(symbol.as_ref())
    }

    pub fn take(&mut self, symbol: impl AsRef<str>) -> bool {
        match self.set.get_mut(symbol.as_ref()) {
            Some(item) => {
                *item = None;
                true
            }
            None => false
        }
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn pos(&self) -> ast::Pos {
        self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    pub fn check_exhausted(&self, err: &mut Error) -> Result<(), Failed> {
        for (key, &value) in &self.set {
            if let Some(pos) = value {
                err.add(pos, format!("unexpected symbol ':{}'", key));
                return Err(Failed)
            }
        }
        Ok(())
    }

    /// Returns the final member of the symbols set.
    ///
    /// If there is more than one item left, adds an error. If there are no
    /// items left, returns `Ok(None)`.
    pub fn take_final(
        self, err: &mut Error
    ) -> Result<Option<ShortString>, Failed> {
        let mut res = None;
        let mut many = false;

        for (key, value) in self.set.into_iter() {
            if let Some(pos) = value {
                if res.is_some() {
                    err.add(pos, format!("unexpected symbol ':{}'", key));
                    many = true;
                }
                else {
                    res = Some(key)
                }
            }
        }
        if many {
            return Err(Failed);
        }
        Ok(res)
    }

    pub fn into_iter(self) -> impl Iterator<Item = ShortString> {
        self.set.into_iter().map(|item| item.0)
    }
}


//------------ ArgumentList --------------------------------------------------

/// Evaluated arguments of a function.
pub struct ArgumentList<T: Theme> {
    /// The positional arguments.
    positional: Vec<Expression<T>>,

    /// The keyword arguments
    keyword: HashMap<ast::Identifier, Expression<T>>,

    /// The start of this argument list in its source.
    pos: ast::Pos,
}

impl<T: Theme> ArgumentList<T> {
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

    pub fn positional(&self) -> &[Expression<T>] {
        &self.positional
    }

    pub fn into_var_positionals(
        self,
        err: &mut Error,
        test: impl FnOnce(&Self, &mut Error) -> Result<bool, Failed>
    ) -> Result<Vec<Expression<T>>, Result<Self, Failed>> {
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

    pub fn into_positionals<const N: usize>(
        self, err: &mut Error
    ) -> Result<[Expression<T>; N], Failed>
    where [Expression<T>; N]: Default {
        if !self.keyword.is_empty() {
            err.add(self.pos, "expected positional arguments only");
            return Err(Failed)
        }
        if self.positional.len() != N {
            err.add(
                self.pos(),
                format!("expected exactly {} positional arguments", N)
            );
            return Err(Failed)
        }

        // XXX This could be more efficient unsing MaybeUninit but
        //     mem::transmute doesn’t seem to like const generics just yet.
        //     Or I am doing something wrong.

        let mut res: [Expression<T>; N] = Default::default();

        for (pos, src) in self.positional.into_iter().enumerate() {
            res[pos] = src;
        }

        Ok(res)
    }

    /// Returns exactly n positional arguments.
    ///
    /// Fails if there are keyword arguments or more than n positional
    /// arguments. Returns `Ok(None)` if there are less than n positional
    /// arguments.
    pub fn into_n_positionals(
        self, n: usize, err: &mut Error
    ) -> Result<Vec<Expression<T>>, Result<ArgumentList<T>, Failed>> {
        self.into_var_positionals(err, |args, err| {
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
    ) -> Result<Expression<T>, Result<ArgumentList<T>, Failed>> {
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
    pub fn get_keyword(&self, key: &str) -> Option<&Expression<T>> {
        self.keyword.get(key)
    }
}

impl<T: Theme> Clone for ArgumentList<T> {
    fn clone(&self) -> Self {
        ArgumentList {
            positional: self.positional.clone(),
            keyword: self.keyword.clone(),
            pos: self.pos
        }
    }
}


//------------ PositionOffset ------------------------------------------------

/// The combined offset of a position.
#[derive(Clone, Debug, Default)]
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

impl ops::AddAssign for PositionOffset {
    fn add_assign(&mut self, other: Self) {
        self.sideways += other.sideways;
        self.shift.0 += other.shift.0;
        self.shift.1 += other.shift.1;
        self.rotation = match (self.rotation, other.rotation) {
            (Some(l), Some(r)) => Some(l + r),
            (Some(l), None) => Some(l),
            (None, Some(r)) => Some(r),
            (None, None) => None
        }
    }
}


//============ Evaluations for AST Types =====================================

//------------ Statements ----------------------------------------------------

impl ast::StatementList {
    pub fn eval_all<T: Theme>(
        self, scope: &mut Scope<T>, features: &mut FeatureSet<T>
    ) -> Result<(), Error> {
        let mut err = Error::default();
        self.eval(scope, features, &mut err);
        err.check()
    }

    pub fn eval<T: Theme>(
        self,
        scope: &mut Scope<T>,
        features: &mut FeatureSet<T>,
        err: &mut Error
    ) {
        for statement in self.statements {
            statement.eval(scope, features, err)
        }
    }
}

impl ast::Statement {
    pub fn eval<T: Theme>(
        self,
        scope: &mut Scope<T>,
        features: &mut FeatureSet<T>,
        err: &mut Error
    ) {
        match self {
            ast::Statement::Let(stm) => stm.eval(scope, err),
            ast::Statement::NoOp(_) => { },
            ast::Statement::Procedure(stm) => {
                let _ = stm.eval(scope, features, err);
            }
            ast::Statement::With(stm) => stm.eval(scope, features, err),
            ast::Statement::Block(stm) => {
                stm.eval(&mut scope.clone(), features, err)
            }
        }
    }
}

impl ast::Let {
    fn eval<T: Theme>(self, scope: &mut Scope<T>, err: &mut Error) {
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
    fn eval<T: Theme>(
        self, 
        scope: &mut Scope<T>,
        features: &mut FeatureSet<T>,
        err: &mut Error
    ) -> Result<(), Failed> {
        let args = self.args.eval(scope, err)?;
        
        match scope.get_var_cloned(self.ident.as_ref()) {
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
                match scope.theme.lookup_procedure(self.ident.as_ref()) {
                    Some(procedure) => {
                        scope.theme.eval_procedure(
                            &procedure, self.pos, args, scope, features, err
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
    pub fn eval<T: Theme>(
        self,
        scope: &mut Scope<T>,
        features: &mut FeatureSet<T>,
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
    fn eval_params<T: Theme>(
        self,
        params: &mut T::RenderParams,
        scope: &Scope<T>,
        err: &mut Error
    ) {
        for item in self.assignments {
            let target = item.target.eval();
            let expression = match item.expression.eval(&scope, err) {
                Ok(expression) => expression,
                Err(_) => continue,
            };
            let _ = scope.theme.update_render_params(
                params, &target, expression, item.pos, err
            );
        }
    }
}

impl ast::ArgumentList {
    fn eval<T: Theme>(
        self, scope: &Scope<T>, err: &mut Error
    ) -> Result<ArgumentList<T>, Failed> {
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
    fn eval<T: Theme>(
        self, scope: &Scope<T>, err: &mut Error
    ) -> Result<Expression<T>, Failed> {
        if self.fragments.len() == 1 {
            let first = self.fragments.into_iter().next().unwrap();
            Ok(Expression::new(first.1.eval_simple(scope, err)?, self.pos))
        }
        else {
            let mut path = Trace::new();
            let mut fragments = self.fragments.into_iter();
            while let Some((conn, frag)) = fragments.next() {
                let (post, pre) = conn.tension();
                match frag.eval_path_component(scope, err)? {
                    PathComponent::Subpath(subpath) => {
                        path.push_subpath(post, pre, subpath)
                    }
                    PathComponent::Position(pos) => {
                        let (end_conn, end_frag) = match fragments.next() {
                            Some(stuff) => stuff,
                            None => {
                                err.add(
                                    self.pos,
                                    "path ends after sole position"
                                );
                                return Err(Failed)
                            }
                        };
                        if end_conn != ast::Connector::Straight {
                            err.add(
                                self.pos,
                                "smooth connector in position pair"
                            );
                            return Err(Failed)
                        }
                        let end_pos = match end_frag.eval_path_component(
                            scope, err
                        )? {
                            PathComponent::Position(pos) => pos,
                            _ => {
                                err.add(
                                    self.pos,
                                    "lone position in path definition"
                                );
                                return Err(Failed)
                            }
                        };
                        path.push_edge(post, pre, Edge::new(pos, end_pos));
                    }
                    PathComponent::Trace(val) => {
                        path.push_trace(post, pre, val)
                    }
                }
            }
            Ok(Expression::new(ExprVal::Trace(path), self.pos))
        }
    }
}

impl ast::Fragment {
    fn eval_simple<T: Theme>(
        self, scope: &Scope<T>, err: &mut Error
    ) -> Result<ExprVal<T>, Failed> {
        match self {
            ast::Fragment::Complex(frag) => frag.eval_expr(scope, err),
            ast::Fragment::List(frag) => frag.eval_expr(scope, err),
            ast::Fragment::Vector(frag) => frag.eval_expr(scope, err),
            ast::Fragment::Atom(frag) => frag.eval(scope, err),
        }
    }

    fn eval_path_component<'s, T: Theme>(
        self, scope: &'s Scope<T>, err: &mut Error
    ) -> Result<PathComponent<'s>, Failed> {
        match self {
            ast::Fragment::Complex(frag) => {
                frag.eval_path_component(scope, err)
            }
            _ => {
                err.add(self.pos(), "expected path component");
                Err(Failed)
            }
        }
    }
}

impl ast::Complex {
    fn eval_expr<T: Theme>(
        self, scope: &Scope<T>, err: &mut Error
    ) -> Result<ExprVal<T>, Failed> {
        match self.section {
            Some(section) => {
                let base = self.external.eval_import_path(scope, err)?;
                section.eval_expr(base, scope, err)
            }
            None => self.external.eval_expr(scope, err),
        }
    }

    fn eval_path_component<'s, T: Theme>(
        self, scope: &'s Scope<T>, err: &mut Error
    ) -> Result<PathComponent<'s>, Failed> {
        match self.section {
            Some(section) => {
                let base = self.external.eval_import_path(scope, err);
                section.eval_either(base?, scope, err)
            }
            None => {
                self.external.eval_path_component(scope, err)
            }
        }
    }
}

impl ast::External {
    fn eval_expr<T: Theme>(
        self, scope: &Scope<T>, err: &mut Error
    ) -> Result<ExprVal<T>, Failed> {
        match self.args {
            Some(args) => {
                let args = args.eval(scope, err)?;
                let func = match scope.get_var_cloned(&self.ident.ident) {
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
                            &self.ident.ident, args, self.pos, scope, err
                        )
                    }
                }?;

                func.eval_expr(scope, err)
            }
            None => {
                match scope.get_var_cloned(&self.ident.ident) {
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

    fn eval_import_path<'s, T: Theme>(
        self, scope: &'s Scope<T>, err: &mut Error
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

    fn eval_path_component<'s, T: Theme>(
        self, scope: &'s Scope<T>, err: &mut Error
    ) -> Result<PathComponent<'s>, Failed> {
        if self.args.is_some() {
            err.add(self.pos, "expected path variable");
            return Err(Failed)
        }
        match scope.get_var(self.ident.as_ref()) {
            Some(&ExprVal::Trace(ref path)) => Ok(PathComponent::Trace(path)),
            _ => {
                err.add(self.pos, "expected path variable");
                Err(Failed)
            }
        }
    }
}

impl ast::Section {
    fn eval_subpath<T: Theme>(
        self, base: &ImportPath, scope: &Scope<T>, err: &mut Error
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
                if let (Ok(mut res), Ok(item)) = (res, item) {
                    res += item;
                    Ok(res)
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

    fn eval_position<T: Theme>(
        self, base: &ImportPath, scope: &Scope<T>, err: &mut Error
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
                if let (Ok(mut res), Ok(item)) = (res, item) {
                    res += item;
                    Ok(res)
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

    fn eval_either<'s, T: Theme>(
        self, base: &ImportPath, scope: &'s Scope<T>, err: &mut Error
    ) -> Result<PathComponent<'s>, Failed> {
        if self.end.is_some() {
            self.eval_subpath(base, scope, err).map(PathComponent::Subpath)
        }
        else {
            self.eval_position(base, scope, err).map(PathComponent::Position)
        }
    }

    fn eval_expr<T: Theme>(
        self, base: &ImportPath, scope: &Scope<T>, err: &mut Error
    ) -> Result<ExprVal<T>, Failed> {
        if self.end.is_some() {
            self.eval_subpath(base, scope, err).map(|subpath| {
                let mut path = Trace::new();
                path.push_subpath(1., 1., subpath);
                ExprVal::Trace(path)
            })
        }
        else {
            self.eval_position(base, scope, err).map(ExprVal::Position)
        }
    }
}

impl ast::Location {
    fn eval<T: Theme>(
        self, base: &ImportPath, scope: &Scope<T>, err: &mut Error
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
                    (Ok(mut res), Ok(item)) => {
                        res += item;
                        Ok(res)
                    }
                    _ => Err(Failed),
                }
            }
        )?;
        let node = node?;
        Ok((node, distance))
    }
}

impl ast::Distance {
    fn eval<T: Theme>(
        self, scope: &Scope<T>, err: &mut Error
    ) -> Result<Distance, Failed> {
        let value = self.value.eval(scope, err)?;
        match self.op {
            ast::AddSub::Add => Ok(value),
            ast::AddSub::Sub => Ok(-value),
        }
    }
}

impl ast::Offset {
    fn eval_subpath<T: Theme>(
        self, scope: &Scope<T>, err: &mut Error
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

    fn eval_position<T: Theme>(
        self, scope: &Scope<T>, err: &mut Error
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
    fn eval_expr<T: Theme>(
        self, scope: &Scope<T>, err: &mut Error
    ) -> Result<ExprVal<T>, Failed> {
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
    fn eval_expr<T: Theme>(
        self, scope: &Scope<T>, err: &mut Error
    ) -> Result<ExprVal<T>, Failed> {
        self.eval(scope, err).map(|res| ExprVal::Vector(res))
    }

    fn eval<T: Theme>(
        self, scope: &Scope<T>, err: &mut Error
    ) -> Result<(Distance, Distance), Failed> {
        let x = self.x.eval(scope, err);
        let y = self.y.eval(scope, err)?;
        Ok((x?, y))
    }
}

impl ast::Atom {
    fn eval<T: Theme>(
        self, scope: &Scope<T>, err: &mut Error
    ) -> Result<ExprVal<T>, Failed> {
        match self {
            ast::Atom::Number(atom) => atom.eval_expr(scope, err),
            ast::Atom::SymbolSet(atom) => atom.eval_expr(scope, err),
            ast::Atom::Text(atom) => atom.eval_expr(scope, err),
            ast::Atom::UnitNumber(atom) => atom.eval_expr(scope, err),
        }
    }
}

impl ast::Number {
    fn eval_expr<T: Theme>(
        self, _scope: &Scope<T>, _err: &mut Error
    ) -> Result<ExprVal<T>, Failed> {
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
    fn eval_expr<T: Theme>(
        self, _scope: &Scope<T>, _err: &mut Error
    ) -> Result<ExprVal<T>, Failed> {
        Ok(ExprVal::SymbolSet(SymbolSet::new(self)))
    }
}

impl ast::Text {
    fn eval_expr<T: Theme>(
        self, _scope: &Scope<T>, _err: &mut Error
    ) -> Result<ExprVal<T>, Failed> {
        let mut res = self.first.content;
        self.others.into_iter().for_each(|val| res.push_str(&val.content));
        Ok(ExprVal::Text(res))
    }
}

impl ast::UnitNumber {
    fn eval_expr<T: Theme>(
        self, scope: &Scope<T>, err: &mut Error
    ) -> Result<ExprVal<T>, Failed> {
        self.eval(scope, err).map(|res| ExprVal::Distance(res))
    }

    fn eval<T: Theme>(
        self, scope: &Scope<T>, err: &mut Error
    ) -> Result<Distance, Failed> {
        scope.theme.eval_distance(
            self.number.eval_float(), self.unit.as_ref(), scope,
            self.pos, err
        )
    }
}

impl ast::Identifier {
    fn eval(self) -> ShortString {
        self.ident
    }
}


//------------ PathComponent -------------------------------------------------

enum PathComponent<'s> {
    Subpath(Subpath),
    Position(Position),
    Trace(&'s Trace),
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

