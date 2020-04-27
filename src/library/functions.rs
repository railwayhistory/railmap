/// The function we support during import.

use crate::features::label;
use crate::import::Failed;
use crate::import::eval;
use crate::import::eval::ExprVal;
use super::fonts::font_from_symbols;


/// All known functions.
///
/// The first element is the name of the function. The second element is the
/// closure evaluating the arguments and producing the function’s result.
/// Since functions can be partially applied, there are three possible return
/// values: `Ok(Ok(_))` is returned if the function succeeds and a result is
/// available. `Ok(Err(_))` is returned if the function doesn’t have all
/// arguments yet. In this case, it can return a modified argument list if it
/// wants to. If it already found the arguments to be lacking, it reports an
/// error and returns `Err(Failed)`.
const FUNCTIONS: &[(
    &str,
    &dyn Fn(
        eval::ArgumentList, &eval::Scope, &mut eval::Error
    ) -> Result<ExprVal, Result<eval::ArgumentList, Failed>>
)] = &[
    // Produces a layout containing a horizontal bar.
    //
    // ```text
    // hbar(width: unit-number)
    // ```
    ("hbar", &|args, _, err| {
        let width =
            args.into_sole_positional(err)?
            .into_number(err)?.0.into_f64();

        Ok(ExprVal::Layout(label::Layout::hbar(width)))
    }),

    // Produces a horizontal box for a label layout.
    //
    // ```text
    // hbox(
    //     alignment, layout *[, layout]
    // )
    // ```
    ("hbox", &|args, _, err| {
        let args = args.into_positionals(err, |args, _| {
            if args.positional().len() < 2 {
                Ok(false)
            }
            else {
                Ok(true)
            }
        })?;
        let mut args = args.into_iter();

        let (align, pos) = args.next().unwrap().into_symbol_set(err)?;
        let halign = label::Align::h_from_symbols(
            &align
        ).unwrap_or(label::Align::Start);
        let valign = match label::Align::v_from_symbols(&align) {
            Some(align) => align,
            None => {
                err.add(pos, "vertical alignment required");
                return Err(Err(Failed))
            }
        };
        let font = font_from_symbols(&align);

        let mut lines = label::Stack::new();
        for expr in args {
            lines.push(expr.into_layout(err)?.0);
        }
        Ok(ExprVal::Layout(label::Layout::hbox(halign, valign, font, lines)))
    }),

    // Resolve a base path.
    //
    // ```text
    // path(name: string) -> stored_path
    // ```
    ("path", &|args, scope, err| {
        let name_expr = args.into_sole_positional(err)?;
        let (name, pos) = name_expr.into_text(err)?;

        match scope.paths().lookup(&name) {
            Some(path) => Ok(ExprVal::ImportPath(path)),
            None => {
                err.add(
                    pos,
                    format!("unresolved path \"{}\"", name)
                );
                Err(Err(Failed))
            }
        }
    }),

    // Produces a span of text.
    //
    // ```text
    // span(font: symbol-set, text)
    // ```
    ("span", &|args, _, err| {
        let mut args = args.into_n_positionals(2, err)?.into_iter();
        let font = args.next().unwrap().into_symbol_set(err);
        let text = args.next().unwrap().into_text(err)?.0;

        let font = font_from_symbols(&font?.0);
        Ok(ExprVal::Layout(label::Layout::span(font, text)))
    }),

    // Produces a vertical box for a label layout.
    //
    // ```text
    // vbox(
    //     alignment, layout *[, layout]
    // )
    // ```
    ("vbox", &|args, _, err| {
        let args = args.into_positionals(err, |args, _| {
            if args.positional().len() < 2 {
                Ok(false)
            }
            else {
                Ok(true)
            }
        })?;
        let mut args = args.into_iter();

        let (align, pos) = args.next().unwrap().into_symbol_set(err)?;
        let halign = match label::Align::h_from_symbols(&align) {
            Some(align) => align,
            None => {
                err.add(pos, "horizonal alignment required");
                return Err(Err(Failed))
            }
        };
        let valign = label::Align::v_from_symbols(
            &align
        ).unwrap_or(label::Align::Start);
        let font = font_from_symbols(&align);

        let mut lines = label::Stack::new();
        for expr in args {
            lines.push(expr.into_layout(err)?.0);
        }
        Ok(ExprVal::Layout(label::Layout::vbox(halign, valign, font, lines)))
    }),
];


//------------ Function ------------------------------------------------------

/// A reference to a function.
#[derive(Clone, Copy, Debug)]
pub struct Function(usize);

impl Function {
    pub fn lookup(name: &str) -> Option<Self> {
        FUNCTIONS.iter().enumerate().find_map(|(i, item)| {
            if item.0 == name {
                Some(Function(i))
            }
            else {
                None
            }
        })
    }

    pub fn eval(
        self,
        args: eval::ArgumentList,
        scope: &eval::Scope,
        err: &mut eval::Error,
    ) -> Result<ExprVal, Result<eval::ArgumentList, Failed>> {
        (*FUNCTIONS[self.0].1)(args, scope, err)
    }
}

