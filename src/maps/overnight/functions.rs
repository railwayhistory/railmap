/// The function we support during import.

use crate::import::Failed;
use crate::import::eval;
use crate::import::eval::ExprVal;
use crate::render::label::Align;
use super::feature::label;
use super::theme::Overnight;


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
        eval::ArgumentList<Overnight>,
        &eval::Scope<Overnight>,
        &mut eval::Error
    ) -> Result<
        ExprVal<Overnight>,
        Result<eval::ArgumentList<Overnight>, Failed>
    >
)] = &[
    // Produces a layout containing a horizontal bar.
    //
    // ```text
    // hbar(width: unit-number)
    // ```
    ("hbar", &|_, _, _| {
        Ok(label::LayoutBuilder::hrule(
            label::PropertiesBuilder::default()
        ).into())
    }),

    // Produces a layout containing a horizontal bar.
    //
    // ```text
    // hrule(class: symbol-set)
    // ```
    ("hrule", &|args, _, err| {
        let properties = label::PropertiesBuilder::from_arg(
            args.into_sole_positional(err)?,
            err
        )?;

        Ok(label::LayoutBuilder::hrule(properties).into())
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

        let (mut align, pos) = args.next().unwrap().into_symbol_set(err)?;
        let halign = Align::h_from_symbols(
            &mut align
        ).unwrap_or(Align::Start);
        let valign = match Align::v_from_symbols(&mut align) {
            Some(align) => align,
            None => {
                err.add(pos, "vertical alignment required");
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

    // Produces a frame.
    //
    // ```text
    // frame(class: symbol-set, content: layout)
    // ```
    ("frame", &|args, _scope, err| {
        let [class, content] = args.into_positionals(err)?;
        let properties = label::PropertiesBuilder::from_arg(class, err);
        let content = label::LayoutBuilder::from_expr(content, err)?;
        Ok(label::LayoutBuilder::frame(
            properties?, content
        ).into())
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

        let (mut align, pos) = args.next().unwrap().into_symbol_set(err)?;
        let halign = match Align::h_from_symbols(&mut align) {
            Some(align) => align,
            None => {
                err.add(pos, "horizonal alignment required");
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

    // Produces a layout containing a horizontal bar.
    //
    // ```text
    // vrule(width: distance)
    // ```
    ("vrule", &|args, _, err| {
        let properties = label::PropertiesBuilder::from_arg(
            args.into_sole_positional(err)?,
            err
        )?;

        Ok(label::LayoutBuilder::vrule(properties).into())
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
        args: eval::ArgumentList<Overnight>,
        scope: &eval::Scope<Overnight>,
        err: &mut eval::Error,
    ) -> Result<
        ExprVal<Overnight>,
        Result<eval::ArgumentList<Overnight>, Failed>
    > {
        (*FUNCTIONS[self.0].1)(args, scope, err)
    }
}

