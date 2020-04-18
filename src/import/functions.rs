/// The function we support during import.

use crate::features::Color;
use crate::features::contour;
use crate::import::units;
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
        "hbib" => simple_line(true, true),
        "hbab" => simple_line(true, false),
        "nbib" => simple_line(false, true),
        "nbab" => simple_line(false, false),
        "elib" => Some(square_dash()),
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
    hauptbahn: bool, in_betrieb: bool,
) -> Option<ExprVal> {
    Some(ExprVal::ContourRule(contour::simple(
        if in_betrieb { Color::BLACK }
        else { Color::grey(0.5) },
        if hauptbahn { 0.8 }
        else { 0.6 }
    )))
}

/// A contour rendering rule producing square dashes.
fn square_dash() -> ExprVal {
    ExprVal::ContourRule(contour::square_dash(
        Color::BLACK,
        8. * units::DT, 3. * units::DT,
        (-0.5 * units::DT, 0.5 * units::DT),
    ))
}
