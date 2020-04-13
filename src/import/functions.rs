/// The function we support during import.

use crate::features::Color;
use crate::features::contour;
use super::{ast, eval};
use super::eval::{ArgumentList, ExprVal};

pub fn eval(
    pos: ast::Pos,
    name: String,
    args: eval::ArgumentList,
    scope: &eval::Scope,
    err: &mut eval::Error,
) -> Option<eval::ExprVal> {
    match name.as_str() {
        "path" => path(pos, args, scope, err),
        "simple_line" => simple_line(pos, args, scope, err),
        _ => {
            err.add(pos, format!("unknown function '{}'", name));
            None
        }
    }
}


/// Resolve a base path.
///
/// ```text
/// path(name: string) -> path::Segment
/// ```
fn path(
    pos: ast::Pos, args: ArgumentList,
    scope: &eval::Scope, err: &mut eval::Error
) -> Option<ExprVal> {
    let name = args.single_pos(err)?.into_text(err)?;

    match scope.paths().lookup(&name) {
        Some(path) => Some(ExprVal::Path(path)),
        None => {
            err.add(pos, format!("unresolved path \"{}\"", name));
            None
        }
    }
}


/// A contour rendering rule for a simple, continous line.
fn simple_line(
    _pos: ast::Pos, _args: ArgumentList,
    _scope: &eval::Scope, _err: &mut eval::Error
) -> Option<ExprVal> {
    Some(ExprVal::ContourRule(contour::simple(Color::BLACK, 1.0)))
}
