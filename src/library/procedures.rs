//! All our procedures.

use crate::features;
use crate::features::label;
use crate::features::contour::RenderContour;
use crate::features::marker::RenderMarker;
use crate::import::{ast, eval};
use crate::import::Failed;
use super::fonts;
use super::border::BorderContour;
use super::colors::Palette;
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
            layout.into_label(position, true, label::Background::Clear.into()),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),

    // Draws a border.
    //
    // ```text
    // border(class: symbol-set, path: path)
    // ```
    ("border", &|pos, args, scope, features, err| {
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
        let rule = BorderContour::new(
            args.next().unwrap().into_symbol_set(err)?.0
        ).into_rule();
        let path = args.next().unwrap().into_path(err)?.0;
        features.insert(
            features::Contour::new(path, rule),
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
            layout.into_label(
                position, false,
                Default::default(),
            ),
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
        let mut properties = label::PropertiesBuilder::from(
            label::FontBuilder::normal(palette.text, fonts::SIZE_LINE_BADGE) 
        );
        properties.set_background(label::Background::Clear);

        let layout = label::LayoutBuilder::hbox(
            label::Align::Center, label::Align::Center, Default::default(),
            vec![
                label::LayoutBuilder::span(text, properties),
            ]
        );

        features.insert(
            layout.into_label(position, true, Default::default()),
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

    // Draws a platform.
    ("platform", &|pos, args, scope, features, err| {
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
                    "expected exactly three positional arguments"
                );
                return Err(Failed);
            }
            Err(Err(_)) => return Err(Failed)
        };
        let class = args.next().unwrap().into_symbol_set(err)?.0;
        let palette = Palette::from_symbols(&class);
        let font = label::FontBuilder::normal(palette.text, fonts::SIZE_S);

        let position = args.next().unwrap().into_position(err);
        let text = args.next().unwrap().into_layout(err);
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
            label::LayoutBuilder::hbox(
                halign, valign, font.into(), vec![text]
            ).into_label(position, false, Default::default()), 
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),

    // Renders a station label.
    //
    // This procedure is deprecated. Use statlabel instead.
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
        let name_font = label::FontBuilder::normal(palette.text, fonts::SIZE_M);
        let km_font = label::FontBuilder::normal(palette.text, fonts::SIZE_XS);

        let position = args.next().unwrap().into_position(err);
        let name = args.next().unwrap().into_layout(err);
        let km = args.next().unwrap().into_layout(err);
        let position = position?.0;
        let mut name = name?.0;
        let mut km = km?.0;

        name.properties_mut().font_mut().defaults(&name_font);
        km.properties_mut().font_mut().defaults(&km_font);

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
            label::LayoutBuilder::vbox(
                halign, label::Align::End, Default::default(), vec![name, km]
            )
        }
        else if class.contains("left") {
            label::LayoutBuilder::hbox(
                label::Align::End, label::Align::Ref, Default::default(),
                vec![
                    label::LayoutBuilder::vbox(
                        halign, label::Align::Ref, Default::default(),
                        vec![name, km]
                    )
                ]
            )
        }
        else if class.contains("bottom") {
            label::LayoutBuilder::vbox(
                halign, label::Align::Start, Default::default(), vec![name, km]
            )
        }
        else /* "right" */ {
            label::LayoutBuilder::hbox(
                label::Align::Start, label::Align::Ref, Default::default(),
                vec![
                    label::LayoutBuilder::vbox(
                        halign, label::Align::Ref, Default::default(),
                        vec![name, km]
                    )
                ]
            )
        };

        features.insert(
            layout.into_label(position, false, Default::default()),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),

    /*
    // Renders a station label.
    //
    // ```text
    // statlabel(class: symbol-set, position, name: layout [, ...], km: layout)
    // ```
    //
    // The class defines where on the stack of name layouts the position is to
    // be located via the symbols `:n`, `:nw`, `:w`, `:sw`, `:s`, `:se`, `:e`,
    // and `:ne` which represent compass directions. One of these is
    // mandatory.
    //
    // The class also defines how the
    // various name boxes are aligned via the `:left`, `:center`, `:right`
    // symbols. The default if neither of them is present is `:center`. The
    // _km_ layout will always be centered below the last `name` layout.
    //
    // Finally, the class provides defaults for the font properties used by
    // layouts. Colour properties (such as :removed or :gone) applied to all
    // layouts. Font variant properties (such as :bold or :designation) are
    // only applied to the name layouts. Font sizes are ignored.
    ("statlabel", &|pos, args, scope, features, err| {
        let args = args.into_positionals(err, |args, err| {
            if args.positional().len() < 4 {
                err.add(self.pos, "expected at least 4 positional arguments");
                Err(Failed)
            }
            else {
                Ok(true)
            }
        }).map_err(|_| Failed)?;
        let mut args = args.into_iter();

        let class = args.next().unwrap().into_symbol_set(err);
        let position = args.next().unwrap().into_position(err);
    */

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

