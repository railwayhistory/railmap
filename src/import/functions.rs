/// The function we support during import.

use crate::import::eval;

pub fn eval(
    _name: String,
    _args: eval::ArgumentList,
    _err: &mut eval::Error,
) -> Option<eval::ExprVal> {
    None
}


/// Resolve a base path.
///
/// ```ignore
/// fn path(name: string) -> path::Segment
/// ```
fn path(
    args: eval::ArgumentList, err: &mut eval::Error
) -> Option<path::ExprValue> {
}

