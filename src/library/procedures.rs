//! All our procedures.

use crate::features;
use crate::features::label;
use crate::features::contour::RenderContour;
use crate::features::marker::RenderMarker;
use crate::import::{ast, eval};
use crate::import::Failed;
use super::colors::Palette;
use super::fonts;
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
    // Draws an area.
    ("area", &|pos, args, scope, features, err| {
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

        let class = args.next().unwrap().into_symbol_set(err)?.0;
        let palette = Palette::from_symbols(&class);
        let path = args.next().unwrap().into_path(err)?.0;
        features.insert(
            features::Contour::new(
                path, features::contour::fill(palette.fill)
            ),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),

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
            features::Label::new(position, true, true, layout),
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
            features::Label::new(position, false, false, layout),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),

    // Renders a badge containing a line label.
    //
    // ```text
    // line_badge(class: symbol-set, position: position, text: Text)
    // ```
    ("line_badge", &|pos, args, scope, features, err| {
        let mut args = match args.into_n_positionals(3, err) {
            Ok(args) => args.into_iter(),
            Err(Ok(args)) => {
                err.add(
                    args.pos(),
                    "expected exactly three positional arguments"
                );
                return Err(Failed);
            }
            Err(Err(_)) => return Err(Failed)
        };
        let class = args.next().unwrap().into_symbol_set(err);
        let position = args.next().unwrap().into_position(err);
        let text = args.next().unwrap().into_text(err)?.0;
        let position = position?.0;
        let palette = Palette::from_symbols(&class?.0);

        let layout = label::Layout::hbox(
            label::Align::Center, label::Align::Center, Default::default(),
            vec![
                label::Layout::span(
                    label::Font::normal(
                        palette.stroke, fonts::SIZE_LINE_BADGE
                    ), text
                )
            ]
        );

        features.insert(
            features::Label::new(position, true, true, layout),
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

    // Renders a one-label with small text.
    //
    // ```text
    // slabel(class: symbol-set, position, text: text|layout)
    // ```
    ("slabel", &|pos, args, scope, features, err| {
        let mut args = match args.into_n_positionals(3, err) {
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
        let class = args.next().unwrap().into_symbol_set(err)?.0;
        let palette = Palette::from_symbols(&class);
        let font = label::Font::normal(palette.stroke, fonts::SIZE_S);

        let position = args.next().unwrap().into_position(err);
        let text = args.next().unwrap().into_based_layout(font, err);
        let position = position?.0;
        let text = text?.0;

        let (halign, valign) = if class.contains("top") {
            (label::Align::Center, label::Align::End)
        }
        else if class.contains("left") {
            (label::Align::End, label::Align::Ref)
        }
        else if class.contains("bottom") {
            (label::Align::Center, label::Align::Start)
        }
        else {
            (label::Align::Start, label::Align::Ref)
        };
        features.insert(
            features::Label::new(
                position, false, false,
                label::Layout::hbox(
                    halign, valign, Default::default(), vec![text]
                )
            ),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),

    // Renders a station label.
    //
    // ```text
    // station(class: symbol-set, position, name: text|layout, km: text|layout)
    // ```
    ("station", &|pos, args, scope, features, err| {
        let mut args = match args.into_n_positionals(4, err) {
            Ok(args) => args.into_iter(),
            Err(Ok(args)) => {
                err.add(
                    args.pos(),
                    "expected exactly four positional arguments"
                );
                return Err(Failed);
            }
            Err(Err(_)) => return Err(Failed)
        };
        let class = args.next().unwrap().into_symbol_set(err)?.0;
        let palette = Palette::from_symbols(&class);
        let name_font = label::Font::normal(palette.stroke, fonts::SIZE_M);
        let km_font = label::Font::normal(palette.stroke, fonts::SIZE_XS);

        let position = args.next().unwrap().into_position(err);
        let name = args.next().unwrap().into_based_layout(name_font, err);
        let km = args.next().unwrap().into_based_layout(km_font, err);
        let position = position?.0;
        let name = name?.0;
        let km = km?.0;

        let halign = if class.contains("left_align") {
            label::Align::Start
        }
        else if class.contains("right_align") {
            label::Align::End
        }
        else {
            label::Align::Center
        };

        let layout = if class.contains("top") {
            label::Layout::vbox(
                halign, label::Align::End, Default::default(), vec![name, km]
            )
        }
        else if class.contains("left") {
            label::Layout::hbox(
                label::Align::End, label::Align::Ref, Default::default(),
                vec![
                    label::Layout::vbox(
                        halign, label::Align::Ref, Default::default(),
                        vec![name, km]
                    )
                ]
            )
        }
        else if class.contains("bottom") {
            label::Layout::vbox(
                halign, label::Align::Start, Default::default(), vec![name, km]
            )
        }
        else /* "right" */ {
            label::Layout::hbox(
                label::Align::Start, label::Align::Ref, Default::default(),
                vec![
                    label::Layout::vbox(
                        halign, label::Align::Ref, Default::default(),
                        vec![name, km]
                    )
                ]
            )
        };

        features.insert(
            features::Label::new(position, false, false, layout),
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

