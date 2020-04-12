/// The function we support during import.

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
        //"subpath" => subpath(pos, args, scope, err),
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


/*
/// Resolve a subpath.
///
/// ```text
/// subpath(
///     name: string, start: distance, end: distance [, offset: distance]
/// ) -> path::Segment
/// ```
fn subpath(
    pos: ast::Pos, args: ArgumentList,
    scope: &eval::Scope, err: &mut eval::Error
) -> Option<ExprVal> {
    let mut args = args.into_pos_iter(err)?.fuse();
    let name = args.next();
    let start = args.next();
    let end = args.next();
    let offset = args.next();
    let extra = args.next();
    if end.is_none() || extra.is_some() {
        err.add(pos, "expected 3 or 4 positional arguments");
        return None
    }

    let name = name?.into_text(err);
    let start = start?.into_distance(err);
    let end = end?.into_distance(err);
    let offset = offset.and_then(|offset| {
        let pos = offset.pos;
        offset.into_distance(err).map(|dist| (pos, dist))
    }).and_then(|(pos, dist)| {
        if dist.world.is_some() {
            err.add(pos, "offset can only have canvas distances");
            None
        }
        else {
            dist.canvas
        }
    }).unwrap_or(0.);

    let path = match scope.get_path(&name) {
        Some(path) => path,
        None => {
            err.add(pos, format!("unresolved path \"{}\"", name));
            return None
        }
    };


    unimplemented!()
}
*/
