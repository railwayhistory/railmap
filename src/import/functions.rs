/// The function we support during import.

use crate::import::eval;

pub fn eval(
    _name: String,
    _args: eval::ArgumentList,
    _err: &mut eval::Error,
) -> Option<eval::Expression> {
    None
}

