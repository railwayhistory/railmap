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
        "dashed_line" => dashed_line(pos, args, scope, err),
        "line" => line(pos, args, scope, err),
        "path" => path(pos, args, scope, err),
        "rgb" => rgb(pos, args, scope, err),
        "rgba" => rgba(pos, args, scope, err),

        _ => {
            err.add(pos, format!("unknown function '{}'", name));
            None
        }
    }
}

/// A contour rendering rule for a simple dashed line.
///
/// ```text
/// dashed_line(
///     color = :color,
///     width = :distance,
///     on = :distance,
///     off = :distance,
///     [offset = :distance]
/// )
/// ```
///
/// All distances can be canvas units only.
fn dashed_line(
    _pos: ast::Pos, args: ArgumentList,
    _scope: &eval::Scope, err: &mut eval::Error
) -> Option<ExprVal> {
    let mut args = args.into_keyword(err)?;
    let color = args.take("color", err)?.into_color(err)?;
    let width = args.take("width", err)?.into_canvas_distance(err)?;
    let on = args.take("on", err)?.into_canvas_distance(err)?;
    let off = args.take("off", err)?.into_canvas_distance(err)?;
    let offset = match args.take_opt("offset") {
        Some(offset) => Some(offset.into_canvas_distance(err)?),
        None => None
    };
        
    args.check_empty(err)?;
    Some(ExprVal::ContourRule(
        contour::dashed_line(color, width, on, off, offset)
    ))
}


/// A contour rendering rule for a continous line.
///
/// ```text
/// line(color: color, width: distance) -> contour_rule
/// ```
fn line(
    _pos: ast::Pos, args: ArgumentList,
    _scope: &eval::Scope, err: &mut eval::Error
) -> Option<ExprVal> {
    let mut args = args.into_n_positional(2, err)?;
    let color = args.next().unwrap().into_color(err)?;
    let width = args.next().unwrap().into_canvas_distance(err)?;
    Some(ExprVal::ContourRule(contour::simple(color, width)))
}


/// Resolve a base path.
///
/// ```text
/// path(name: string) -> stored_path
/// ```
fn path(
    pos: ast::Pos, args: ArgumentList,
    scope: &eval::Scope, err: &mut eval::Error
) -> Option<ExprVal> {
    let name = args.single_positional(err)?.into_text(err)?;

    match scope.paths().lookup(&name) {
        Some(path) => Some(ExprVal::Path(path)),
        None => {
            err.add(pos, format!("unresolved path \"{}\"", name));
            None
        }
    }
}


/// Produces an opaque color.
///
/// ```text
/// rgb(red: number, green: number, blue: number) -> color
/// ```
///
/// The color values must be between 0 and 1.
fn rgb(
    _pos: ast::Pos, args: ArgumentList,
    _scope: &eval::Scope, err: &mut eval::Error
) -> Option<ExprVal> {
    let mut args = args.into_n_positional(3, err)?;
    let red = args.next().unwrap().into_number(err)?.into_f64();
    let green = args.next().unwrap().into_number(err)?.into_f64();
    let blue = args.next().unwrap().into_number(err)?.into_f64();
    Some(ExprVal::Color(Color::rgb(red, green, blue)))
}


/// Produces a color with an alpha value.
///
/// ```text
/// rgba(
///     red: number, green: number, blue: number, alpha: number
/// ) -> Color
/// ```
///
/// The color values must be between 0 and 1.
fn rgba(
    _pos: ast::Pos, args: ArgumentList,
    _scope: &eval::Scope, err: &mut eval::Error
) -> Option<ExprVal> {
    let mut args = args.into_n_positional(4, err)?;
    let red = args.next().unwrap().into_number(err)?.into_f64();
    let green = args.next().unwrap().into_number(err)?.into_f64();
    let blue = args.next().unwrap().into_number(err)?.into_f64();
    let alpha = args.next().unwrap().into_number(err)?.into_f64();
    Some(ExprVal::Color(Color::rgba(red, green, blue, alpha)))
}

