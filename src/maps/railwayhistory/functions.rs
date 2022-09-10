/// The function we support during import.

use crate::import::Failed;
use crate::import::eval;
use crate::import::eval::ExprVal;
use crate::render::color::Color;
use crate::render::label::Align;
use crate::render::path::{Distance, MapDistance};
use super::feature::label;
use super::theme::Railwayhistory;


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
        eval::ArgumentList<Railwayhistory>,
        &eval::Scope<Railwayhistory>,
        &mut eval::Error
    ) -> Result<
        ExprVal<Railwayhistory>,
        Result<eval::ArgumentList<Railwayhistory>, Failed>
    >
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

        Ok(label::LayoutBuilder::hrule(
            Distance::new(None, vec![MapDistance::new(width, 0)]),
            label::PropertiesBuilder::default()
        ).into())
    }),

    // Produces a horizontal box for a label layout.
    //
    // ```text
    // hbox(
    //     alignment, layout *[, layout]
    // )
    // ```
    ("hbox", &|args, _scope, err| {
        let args = args.into_var_positionals(err, |args, _| {
            if args.positional().len() < 2 {
                Ok(false)
            }
            else {
                Ok(true)
            }
        })?;
        let mut args = args.into_iter();

        let mut align = args.next().unwrap().into_symbol_set(err)?;
        let halign = Align::h_from_symbols(
            &mut align
        ).unwrap_or(Align::Start);
        let valign = match Align::v_from_symbols(&mut align) {
            Some(align) => align,
            None => {
                err.add(align.pos(), "vertical alignment required");
                return Err(Err(Failed))
            }
        };
        let properties = label::PropertiesBuilder::from_symbols(&mut align);
        align.check_exhausted(err)?;
        Ok(label::LayoutBuilder::hbox(
            halign, valign, properties,
            label::StackBuilder::from_args(args, err)?,
        ).into())
    }),

    // Resolves a color from a string of hex digits.
    //
    // ```text
    // hexcolor(code: string) -> color
    // ```
    ("hexcolor", &|args, _, err| {
        let color = args.into_sole_positional(err)?;
        let (color, pos) = color.into_text(err)?;

        if color.len() != 6 {
            err.add(pos, "expected color code");
            return Err(Err(Failed))
        }
        if !color.is_ascii() {
            err.add(pos, "expected color code");
            return Err(Err(Failed))
        }
        let red = match u8::from_str_radix(&color[0..2], 16) {
            Ok(red) => f64::from(red) / 255.,
            Err(_) => {
                err.add(pos, "expected color code");
                return Err(Err(Failed))
            }
        };
        let green = match u8::from_str_radix(&color[2..4], 16) {
            Ok(red) => f64::from(red) / 255.,
            Err(_) => {
                err.add(pos, "expected color code");
                return Err(Err(Failed))
            }
        };
        let blue = match u8::from_str_radix(&color[4..6], 16) {
            Ok(red) => f64::from(red) / 255.,
            Err(_) => {
                err.add(pos, "expected color code");
                return Err(Err(Failed))
            }
        };
        Ok(ExprVal::Color(Color::rgb(red, green, blue)))
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

    // Returns a color expression.
    //
    // ```text
    // rgb(red, green, blue)
    // ```
    ("rgb", &|args, _, err| {
        let mut args = args.into_n_positionals(3, err)?.into_iter();
        Ok(ExprVal::Color(
            Color::rgb(
                args.next().unwrap().into_f64(err)?.0,
                args.next().unwrap().into_f64(err)?.0,
                args.next().unwrap().into_f64(err)?.0,
            )
        ))
    }),

    // Produces a span of text.
    //
    // ```text
    // span(font: symbol-set, text)
    // ```
    ("span", &|args, _scope, err| {
        let [properties, text] = args.into_positionals(err)?;
        let properties = label::PropertiesBuilder::from_arg(properties, err)?;
        let text = text.into_text(err)?.0;
        Ok(label::LayoutBuilder::span(text, properties).into())
    }),

    // Produces a vertical box for a label layout.
    //
    // ```text
    // vbox(
    //     alignment, layout *[, layout]
    // )
    // ```
    ("vbox", &|args, _scope, err| {
        let args = args.into_var_positionals(err, |args, _| {
            if args.positional().len() < 2 {
                Ok(false)
            }
            else {
                Ok(true)
            }
        })?;
        let mut args = args.into_iter();

        let mut align = args.next().unwrap().into_symbol_set(err)?;
        let halign = match Align::h_from_symbols(&mut align) {
            Some(align) => align,
            None => {
                err.add(align.pos(), "horizonal alignment required");
                return Err(Err(Failed))
            }
        };
        let valign = Align::v_from_symbols(
            &mut align
        ).unwrap_or(Align::Start);
        let properties = label::PropertiesBuilder::from_symbols(&mut align);
        align.check_exhausted(err)?;
        Ok(label::LayoutBuilder::vbox(
            halign, valign, properties,
            label::StackBuilder::from_args(args, err)?,
        ).into())
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
        &self,
        args: eval::ArgumentList<Railwayhistory>,
        scope: &eval::Scope<Railwayhistory>,
        err: &mut eval::Error,
    ) -> Result<
        ExprVal<Railwayhistory>,
        Result<eval::ArgumentList<Railwayhistory>, Failed>
    > {
        (*FUNCTIONS[self.0].1)(args, scope, err)
    }
}

