/// The function we support during import.

//use crate::features::label;
use crate::import::Failed;
use crate::import::eval;
use crate::import::eval::ExprVal;


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
/*

    // A contour rendering rule for a simple dashed line.
    //
    // ```text
    // dashed_line(
    //     color = :color,
    //     width = :distance,
    //     on = :distance,
    //     off = :distance,
    //     [offset = :distance]
    // )
    // ```
    //
    // All distances can be canvas units only.
    ("dashed_line", &|args, _, err| {
        args.keyword_only(err)?;
        let color = match args.get_keyword("color") {
            Some(color) => color.to_color(err)?,
            None => return Ok(None)
        };
        let width = match args.get_keyword("width") {
            Some(width) => width.to_canvas_distance(err)?,
            None => return Ok(None)
        };
        let on = match args.get_keyword("on") {
            Some(on) => on.to_canvas_distance(err)?,
            None => return Ok(None)
        };
        let off = match args.get_keyword("off") {
            Some(off) => off.to_canvas_distance(err)?,
            None => return Ok(None)
        };
        let offset = match args.get_keyword("offset") {
            Some(offset) => Some(offset.to_canvas_distance(err)?),
            None => None
        };
        Ok(Some(ExprVal::ContourRule(
            contour::dashed_line(color, width, on, off, offset)
        )))
    }),

    // A font for rendering a label.
    //
    // ```text
    // font(
    //     color = color,
    //     size = distance,
    // )
    // ```
    ("font", &|args, _, err| {
        args.keyword_only(err)?;
        let color = match args.get_keyword("color") {
            Some(color) => color.to_color(err)?,
            None => return Ok(None)
        };
        let size = match args.get_keyword("size") {
            Some(on) => on.to_canvas_distance(err)?,
            None => return Ok(None)
        };
        Ok(Some(ExprVal::Font(
            label::FontInfo::new(color, size).into_font()
        )))
    }),

    // Produces a horizontal box for a label layout.
    //
    // ```text
    // hbox(
    //     alignment, layout *[, layout]
    // )
    // ```
    ("hbox", &|args, _, err| {
        let args = args.positional_only(err)?;
        if args.len() < 2 {
            return Ok(None)
        }
        let mut args = args.into_iter();

        let align = args.next().unwrap().to_align(err)?;
        let mut lines = Vec::new();
        for expr in args {
            lines.push(expr.to_layout(err)?);
        }
        Ok(Some(ExprVal::Layout(label::Layout::Hbox(
            label::Hbox::new(align, lines)
        ))))
    }),

    // A contour rendering rule for a continous line.
    //
    // ```text
    // line(color: color, width: distance) -> contour_rule
    // ```
    ("line", &|args, _, err| {
        let mut args = match args.n_positional_only(2, err)? {
            Some(args) => args.into_iter(),
            None => return Ok(None)
        };
        let color = args.next().unwrap().to_color(err)?;
        let width = args.next().unwrap().to_canvas_distance(err)?;
        Ok(Some(ExprVal::ContourRule(contour::simple(color, width))))
    }),

    // A symbol rendering rule for a monochrome symbol.
    //
    // ```text
    // mono_symbol(
    //     symbol:text, color:color, rotation:distance
    // )
    // ```
    ("mono_symbol", &|args, _, err| {
        let mut args = match args.n_positional_only(3, err)? {
            Some(args) => args.into_iter(),
            None => return Ok(None)
        };
        let symbol = args.next().unwrap();
        let pos = symbol.pos;
        let symbol = symbol.to_text(err)?;
        let color = args.next().unwrap().to_color(err)?;
        let rotation = args.next().unwrap().to_number(err)?.into_f64();
        match symbol::monochrome(symbol, color, rotation) {
            Some(rule) => Ok(Some(ExprVal::SymbolRule(rule))),
            None => {
                err.add(
                    pos,
                    format!("unresolved symbol name '{}'", symbol)
                );
                Err(Failed)
            }
        }
    }),
    */

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

    /*
    // Produces an opaque color.
    //
    // ```text
    // rgb(red: number, green: number, blue: number) -> color
    // ```
    //
    // The color values must be between 0 and 1.
    ("rgb", &|args, _, err| {
        let mut args = match args.n_positional_only(3, err)? {
            Some(args) => args.into_iter(),
            None => return Ok(None),
        };
        let red = args.next().unwrap().to_number(err)?.into_f64();
        let green = args.next().unwrap().to_number(err)?.into_f64();
        let blue = args.next().unwrap().to_number(err)?.into_f64();
        Ok(Some(ExprVal::Color(Color::rgb(red, green, blue))))
    }),

    // Produces a color with an alpha value.
    //
    // ```text
    // rgba(
    //     red: number, green: number, blue: number, alpha: number
    // ) -> Color
    // ```
    //
    // The color values must be between 0 and 1.
    ("rgba", &|args, _, err| {
        let mut args = match args.n_positional_only(4, err)? {
            Some(args) => args.into_iter(),
            None => return Ok(None)
        };
        let red = args.next().unwrap().to_number(err)?.into_f64();
        let green = args.next().unwrap().to_number(err)?.into_f64();
        let blue = args.next().unwrap().to_number(err)?.into_f64();
        let alpha = args.next().unwrap().to_number(err)?.into_f64();
        Ok(Some(ExprVal::Color(Color::rgba(red, green, blue, alpha))))
    }),

    // Produces a span of text.
    //
    // ```text
    // span(font, text)
    // ```
    ("span", &|args, _, err| {
        let mut args = match args.n_positional_only(2, err)? {
            Some(args) =>  args.into_iter(),
            None => return Ok(None),
        };
        let font = args.next().unwrap().to_font(err)?;
        let text = args.next().unwrap().to_text(err)?;
        Ok(Some(ExprVal::Layout(label::Layout::Span(
            label::Span::new(font, text.to_string())
        ))))
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

        let (align, pos) = args.next().unwrap().into_symbol(err)?;
        let align = match align.as_str() {
            "left" => label::Align::Start,
            "center" => label::Align::Center,
            "base" => label::Align::Ref,
            "right" => label::Align::End,
            _ => {
                err.add(pos, "expected horizonal alignment");
                return Err(Err(Failed))
            }
        };
        let mut lines = Vec::new();
        for expr in args {
            lines.push(expr.into_layout(err)?.0);
        }
        Ok(ExprVal::Layout(label::Layout::Vbox(
            label::Vbox::new(align, lines)
        )))
    }),
    */
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

