//! All our procedures.

use crate::features;
use crate::features::label;
use crate::features::contour::RenderContour;
use crate::features::marker::RenderMarker;
use crate::import::{ast, eval};
use crate::import::Failed;
use super::markers::StandardMarker;
use super::track::TrackContour;


const PROCEDURES: &[(
    &str,
    &dyn Fn(
        ast::Pos,
        eval::ArgumentList,
        &eval::Scope,
        &mut features::FeatureSet,
        &mut eval::Error
    ) -> Result<(), Failed>
)] = &[
    // Draws a badge.
    //
    // ```text
    // badge(position: position, layout: layout)
    // ```
    ("badge", &|pos, args, scope, features, err| {
        let mut args = match args.into_n_positionals(2, err) {
            Ok(args) => args.into_iter(),
            Err(Ok(args)) => {
                err.add(
                    args.pos(),
                    "expected exactly two positional arguments"
                );
                return Err(Failed);
            }
            Err(Err(_)) => return Err(Failed)
        };
        let position = args.next().unwrap().into_position(err);
        let layout = args.next().unwrap().into_layout(err)?.0;
        let position = position?.0;
        features.insert(
            features::Label::new(position, true, layout),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),

    // Renders casing for a track.
    //
    // ```text
    // casing(class: symbol-set, path: path)
    // ```
    ("casing", &|pos, args, scope, features, err| {
        let mut args = match args.into_n_positionals(2, err) {
            Ok(args) => args.into_iter(),
            Err(Ok(args)) => {
                err.add(
                    args.pos(),
                    "expected exactly two positional arguments"
                );
                return Err(Failed);
            }
            Err(Err(_)) => return Err(Failed)
        };
        let rule = TrackContour::new(
            true, args.next().unwrap().into_symbol_set(err)?.0
        ).into_rule();
        let path = args.next().unwrap().into_path(err)?.0;
        features.insert(
            features::Contour::new(path, rule),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),

    // Draws a label.
    //
    // ```text
    // label(position: position, layout: layout)
    // ```
    ("label", &|pos, args, scope, features, err| {
        let mut args = match args.into_n_positionals(2, err) {
            Ok(args) => args.into_iter(),
            Err(Ok(args)) => {
                err.add(
                    args.pos(),
                    "expected exactly two positional arguments"
                );
                return Err(Failed);
            }
            Err(Err(_)) => return Err(Failed)
        };
        let position = args.next().unwrap().into_position(err);
        let layout = args.next().unwrap().into_layout(err)?.0;
        let position = position?.0;
        features.insert(
            features::Label::new(position, false, layout),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),

    // Renders a badge containing a line label.
    //
    // ```text
    // line_badge(position: position, text: Text)
    // ```
    ("line_badge", &|pos, args, scope, features, err| {
        let mut args = match args.into_n_positionals(2, err) {
            Ok(args) => args.into_iter(),
            Err(Ok(args)) => {
                err.add(
                    args.pos(),
                    "expected exactly two positional arguments"
                );
                return Err(Failed);
            }
            Err(Err(_)) => return Err(Failed)
        };
        let position = args.next().unwrap().into_position(err);
        let text = args.next().unwrap().into_text(err)?.0;
        let position = position?.0;

        let layout = label::Layout::hbox(
            label::Align::Center, label::Align::Center,
            vec![
                label::Layout::span(
                    label::FontInfo::new(
                        features::Color::BLACK,
                        6.
                    ).into_font(),
                    text
                )
            ]
        );

        features.insert(
            features::Label::new(position, true, layout),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),


    // Draw a symbol onto the map.
    //
    // ```text
    // marker(marker: symbol-set, position: position)
    // ```
    ("marker", &|pos, args, scope, features, err| {
        let mut args = match args.into_n_positionals(2, err) {
            Ok(args) => args.into_iter(),
            Err(Ok(args)) => {
                err.add(
                    args.pos(),
                    "expected exactly two positional arguments"
                );
                return Err(Failed);
            }
            Err(Err(_)) => return Err(Failed)
        };
        let rule = StandardMarker::create(
            pos, args.next().unwrap().into_symbol_set(err)?.0, err,
        );
        let position = args.next().unwrap().into_position(err)?.0;
        features.insert(
            features::Marker::new(position, rule?.into_rule()),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),

    // Renders a station label.
    //
    // ```text
    // station(position, class: symbol-set, name: text|layout, km: text|layout)
    // ```
    ("station", &|pos, args, scope, features, err| {
        let mut args = match args.into_n_positionals(4, err) {
            Ok(args) => args.into_iter(),
            Err(Ok(args)) => {
                err.add(
                    args.pos(),
                    "expected exactly four, positional arguments"
                );
                return Err(Failed);
            }
            Err(Err(_)) => return Err(Failed)
        };
        let position = args.next().unwrap().into_position(err);
        let class = args.next().unwrap().into_symbol_set(err);
        let name = args.next().unwrap().into_layout_or_text(err);
        let km = args.next().unwrap().into_layout_or_text(err);
        let class = class?.0;
        let position = position?.0;
        let name = name?.0;
        let km = km?.0;

        let name = name.unwrap_or_else(|name| {
            label::Layout::span(label::FontInfo::black(8.).into_font(), name)
        });
        let km = km.unwrap_or_else(|km| {
            label::Layout::span(label::FontInfo::black(6.).into_font(), km)
        });

        let layout = if class.contains("top") {
            label::Layout::vbox(
                label::Align::Center, label::Align::End, vec![name, km]
            )
        }
        else if class.contains("left") {
            label::Layout::hbox(
                label::Align::End, label::Align::Ref, vec![
                    label::Layout::vbox(
                        label::Align::Center, label::Align::Ref, vec![name, km]
                    )
                ]
            )
        }
        else if class.contains("bottom") {
            label::Layout::vbox(
                label::Align::Center, label::Align::Start, vec![name, km]
            )
        }
        else /* "right" */ {
            label::Layout::hbox(
                label::Align::Start, label::Align::Ref, vec![
                    label::Layout::vbox(
                        label::Align::Center, label::Align::Ref, vec![name, km]
                    )
                ]
            )
        };

        features.insert(
            features::Label::new(position, false, layout),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),


    // Renders a bit of track.
    //
    // ```text
    // track(class: symbol-set, path: path)
    // ```
    ("track", &|pos, args, scope, features, err| {
        let mut args = match args.into_n_positionals(2, err) {
            Ok(args) => args.into_iter(),
            Err(Ok(args)) => {
                err.add(
                    args.pos(),
                    "expected exactly two positional arguments"
                );
                return Err(Failed);
            }
            Err(Err(_)) => return Err(Failed)
        };
        let rule = TrackContour::new(
            false, args.next().unwrap().into_symbol_set(err)?.0
        ).into_rule();
        let path = args.next().unwrap().into_path(err)?.0;
        features.insert(
            features::Contour::new(path, rule),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),
];


//------------ Procedure -----------------------------------------------------

/// A reference to a function.
#[derive(Clone, Copy, Debug)]
pub struct Procedure(usize);

impl Procedure {
    pub fn lookup(name: &str) -> Option<Self> {
        PROCEDURES.iter().enumerate().find_map(|(i, item)| {
            if item.0 == name {
                Some(Procedure(i))
            }
            else {
                None
            }
        })
    }

    pub fn eval(
        self,
        pos: ast::Pos,
        args: eval::ArgumentList,
        scope: &eval::Scope,
        features: &mut features::FeatureSet,
        err: &mut eval::Error,
    ) -> Result<(), Failed> {
        (*PROCEDURES[self.0].1)(pos, args, scope, features, err)
    }
}


/*
//------------ Label ---------------------------------------------------------

/// The label statement renders text onto the map.
///
/// ```text
/// label ::= "label"
///           [ "with" params:assignment-list ]
///           position
///           layout:expression ";"
/// ```
#[derive(Clone, Debug)]
pub struct Label {
    /// Optional updates to the rendering parameters for this contour.
    pub params: Option<AssignmentList>,

    /// The position of the symbol.
    pub position: Position,

    /// The layout to render.
    pub layout: Expression,

    /// The start of the text statement in the source.
    pub pos: Pos
}

impl Label {
    fn parse(input: Span) -> IResult<Span, Self> {
        let pos = Pos::capture(&input);
        let (input, _) = opt_ws(tag("label"))(input)?;
        let (input, _) = skip_ws(input)?;
        let (input, params) = opt(
            preceded(
                opt_ws(tag("with")),
                opt_ws(AssignmentList::parse)
            )
        )(input)?;
        let (input, position) = opt_ws(Position::parse)(input)?;
        let (input, layout) = opt_ws(Expression::parse)(input)?;
        let (input, _) = opt_ws(tag_char(';'))(input)?;
        Ok((input, Label { params, position, layout, pos }))
    }

    fn eval(
        self,
        scope: &mut Scope,
        features: &mut FeatureSet,
        err: &mut Error
    ) {
        // Get the rendering parameters for this contour.
        let mut params = scope.params.clone();
        if let Some(value) = self.params {
            value.eval_params(&mut params, scope, err);
        }

        // Get all the parts.
        let layout = self.layout.eval_layout(scope, err);
        let position = self.position.eval(scope, err);

        // If we don’t have a detail, complain.
        let detail = match params.detail {
            Some(detail) => detail,
            None => {
                err.add(self.pos, "'detail' rendering parameter not yet set");
                return
            }
        };

        // If we have both a rule and a position, create the feature.
        if let (Some(layout), Some(position)) = (layout, position) {
            features.insert(
                Label::new(position, layout),
                detail,  params.layer   
            )
        }
    }
}



*/

