//! Functions provided to the map language.

use femtomap::import::ast::Pos;
use femtomap::import::eval::{EvalErrors, Failed};
use femtomap::import::path::ImportPathSet;
use femtomap::render::Color;
use crate::feature::label;
use super::eval::{ArgumentList, Scope, Value};

//------------ eval ----------------------------------------------------------

/// Evaluates a function call.
pub fn eval<'s>(
    name: &str,
    args: ArgumentList<'s>,
    scope: &Scope<'s>,
    paths: &'s ImportPathSet,
    pos: Pos, err: &mut EvalErrors,
) -> Result<Value<'s>, Failed> {
    let function = match FUNCTIONS.iter().find(|f| f.0 == name) {
        Some(function) => function,
        None => {
            err.add(pos, format!("undefined function '{}'", name));
            return Err(Failed)
        }
    };
    (function.1)(args, scope, paths, err)
}


//------------ Functions -----------------------------------------------------

/// All defined functions.
///
/// The first element is the name of the function. The second element is the
/// closure evaluating the arguments and producing the function’s result.
const FUNCTIONS: &[(
    &str,
    &dyn for<'a> Fn(
        ArgumentList<'a>,
        &Scope<'a>,
        &'a ImportPathSet,
        &mut EvalErrors
    ) -> Result<Value<'a>, Failed>
)] = &[
    // Produces a layout containing a horizontal bar.
    //
    // ```text
    // hbar(width: unit-number)
    // ```
    ("hbar", &|args, _, _, err| {
        let [width] = args.into_array::<1>(err)?;
        let _width = width.eval::<f64>(err)?;
        let mut props = label::LayoutProperties::default();
        props.set_layout_type(label::LayoutType::Rule);

        Ok(Value::Custom(label::Layout::hrule(props).into()))
    }),

    // Produces a horizontal box for a label layout.
    //
    // ```text
    // hbox(
    //     alignment, layout *[, layout]
    // )
    // ```
    ("hbox", &|args, _, _,  err| {
        let ([align], layouts) = args.into_var_array::<1>(err)?;
        let mut align = align.eval(err)?;
        let halign = match label::halign_from_symbols(&mut align) {
            Some(align) => align,
            None => {
                err.add(align.pos(), "horizontal alignment required");
                return Err(Failed)
            }
        };
        let valign = match label::valign_from_symbols(&mut align) {
            Some(align) => align,
            None => {
                err.add(align.pos(), "vertical alignment required");
                return Err(Failed)
            }
        };
        let mut properties = label::LayoutProperties::from_symbols(&mut align);
        if align.take("frame") {
            properties.set_layout_type(label::LayoutType::TextFrame);
        }
        align.check_exhausted(err)?;
        Ok(Value::Custom(
            label::Layout::hbox(
                halign, valign, properties,
                label::layouts_from_args(layouts, err)?,
            ).into()
        ))
    }),

    // Produces a horizontal rule.
    //
    // ```text
    // hrule([style])
    // ```
    ("hrule", &|args, _, _, err| {
        let mut props = if args.is_empty() {
            label::LayoutProperties::default()
        }
        else {
            let [props] = args.into_array(err)?;
            label::LayoutProperties::from_arg(props, err)?
        };
        props.set_layout_type(label::LayoutType::Rule);
        Ok(Value::Custom(label::Layout::hrule(props).into()))
    }),

    // Resolves a color from a string of hex digits.
    //
    // ```text
    // hexcolor(code: string) -> color
    // ```
    ("hexcolor", &|args, _, _, err| {
        let [color] = args.into_array(err)?;
        let (color, pos) = color.eval::<(String, _)>(err)?;
        Color::try_from(color).map(Value::Color).map_err(|_| {
            err.add(pos, "expected color code");
            Failed
        })
    }),

    // Resolve a base path.
    //
    // ```text
    // path(name: string) -> stored_path
    // ```
    ("path", &|args, _, paths, err| {
        let [name] = args.into_array(err)?;
        let (name, pos) = name.eval::<(String, _)>(err)?;
        match paths.lookup(&name) {
            Some(path) => Ok(Value::ImportPath(path)),
            None => {
                err.add(
                    pos,
                    format!("unresolved path \"{}\"", name)
                );
                Err(Failed)
            }
        }
    }),

    // Produces a span of text.
    //
    // ```text
    // span(font: symbol-set, text)
    // ```
    ("span", &|args, _, _, err| {
        let [properties, text] = args.into_array(err)?;
        let properties = label::LayoutProperties::from_arg(properties, err)?;
        Ok(Value::Custom(
            label::Layout::span(text.eval(err)?, properties).into()
        ))
    }),

    // Produces a vertical box for a label layout.
    //
    // ```text
    // vbox(
    //     alignment, layout *[, layout]
    // )
    // ```
    ("vbox", &|args, _, _,  err| {
        let ([align], layouts) = args.into_var_array::<1>(err)?;
        let mut align = align.eval(err)?;
        let halign = match label::halign_from_symbols(&mut align) {
            Some(align) => align,
            None => {
                err.add(align.pos(), "horizontal alignment required");
                return Err(Failed)
            }
        };
        let valign = match label::valign_from_symbols(&mut align) {
            Some(align) => align,
            None => {
                err.add(align.pos(), "vertical alignment required");
                return Err(Failed)
            }
        };
        let mut properties = label::LayoutProperties::from_symbols(&mut align);
        if align.take("frame") {
            properties.set_layout_type(label::LayoutType::TextFrame);
        }
        align.check_exhausted(err)?;
        Ok(Value::Custom(
            label::Layout::vbox(
                halign, valign, properties,
                label::layouts_from_args(layouts, err)?,
            ).into()
        ))
    }),

    // Produces a vertical rule.
    //
    // ```text
    // vrule([style])
    // ```
    ("vrule", &|args, _, _, err| {
        let mut props = if args.is_empty() {
            label::LayoutProperties::default()
        }
        else {
            let [props] = args.into_array(err)?;
            label::LayoutProperties::from_arg(props, err)?
        };
        props.set_layout_type(label::LayoutType::Rule);
        Ok(Value::Custom(label::Layout::vrule(props).into()))
    }),
];

