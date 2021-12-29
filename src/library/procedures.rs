//! All our procedures.

use crate::features;
use crate::features::label;
use crate::features::contour::RenderContour;
use crate::features::marker::RenderMarker;
use crate::import::{ast, eval};
use crate::import::Failed;
use super::fonts;
use super::border::BorderContour;
use super::class::Class;
use super::colors::Style;
use super::markers::StandardMarker;
use super::track::{TrackContour, TrackShading};


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

        let class = Class::from_symbols(
            &args.next().unwrap().into_symbol_set(err)?.0
        );
        let path = args.next().unwrap().into_path(err)?.0;
        features.insert(
            features::Contour::new(
                path, features::contour::fill(class.standard_color())
            ),
            scope.params().detail(pos, err)?,
            scope.params().layer() - 0.1 + class.layer_offset(),
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
        let style = Style::from_name(scope.params().style());
        let rule = TrackContour::new(
            style, true, &args.next().unwrap().into_symbol_set(err)?.0
        ).into_rule();
        let path = args.next().unwrap().into_path(err)?.0;
        features.insert(
            features::Contour::new(path, rule),
            scope.params().detail(pos, err)?,
            scope.params().layer() - 0.3,
        );
        Ok(())
    }),

    // Draws a line attaching a label to something.
    //
    // ```text
    // guide(class: symbol-set, path: path)
    // ```
    ("guide", &|pos, args, scope, features, err| {
        // Currently, this is a track with the "guide" class added to the
        // classes.
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
        let style = Style::from_name(scope.params().style());
        let mut classes = args.next().unwrap().into_symbol_set(err)?.0;
        classes.insert("guide".into());
        let rule = TrackContour::new(
            style, false, &classes,
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
    // label(class: symbol-set, position: position, layout: layout)
    // ```
    ("label", &|pos, args, scope, features, err| {
        let args = args.into_positionals(err, |args, err| {
            if args.positional().len() == 2 || args.positional().len() == 3 {
                Ok(true)
            }
            else {
                err.add(
                    args.pos(),
                    "expected 2 or 3 positional arguments"
                );
                Err(Failed)
            }
        }).map_err(|_| Failed)?;
        let three = args.len() > 2;
        let mut args = args.into_iter();
        let class = if three {
            Some(args.next().unwrap().into_symbol_set(err))
        }
        else {
            None
        };
        let position = args.next().unwrap().into_position(err);
        let layout = args.next().unwrap().into_layout(err)?.0;
        let position = position?.0;
        let class = match class {
            Some(class) => class?.0,
            None => Default::default()
        };

        features.insert(
            layout.into_label(
                position, false,
                label::Properties::new(
                    label::Font::default().update(
                        &fonts::base_font_from_symbols(
                            &class,
                            Style::from_name(scope.params().style()),
                        )
                    ),
                    Default::default(),
                ),
            ),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())

        /*
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
        */
    }),

    // Renders a badge containing a line label.
    //
    // ```text
    // line_badge(class: symbol-set, position: position, text: Text)
    // ```
    /*
    ("line_badge", &|_, _, _, _, _| {
        Ok(())
    }),
    */
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
        /*
        let palette = Style::from_name(
            scope.params().style()
        ).palette(&class?.0);
        */
        let mut properties = label::PropertiesBuilder::from(
            label::FontBuilder::normal(
                Class::from_symbols(&class?.0).label_color(),
                fonts::SIZE_LINE_BADGE
            ) 
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
            pos,
            Style::from_name(scope.params().style()),
            args.next().unwrap().into_symbol_set(err)?.0,
            err,
        );
        let position = args.next().unwrap().into_position(err)?.0;
        features.insert(
            features::Marker::new(position, rule?.into_rule()),
            scope.params().detail(pos, err)?,
            scope.params().layer() - 0.2,
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

        let class = Class::from_symbols(
            &args.next().unwrap().into_symbol_set(err)?.0
        );
        let path = args.next().unwrap().into_path(err)?.0;
        features.insert(
            features::Contour::new(
                path, features::contour::fill(class.standard_color())
            ),
            scope.params().detail(pos, err)?,
            scope.params().layer() - 0.1 + class.layer_offset(),
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
        //let palette = Style::from_name(scope.params().style()).palette(&class);
        let font = label::FontBuilder::normal(
            Class::from_symbols(&class).label_color(),
            fonts::SIZE_S
        );

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
        //let palette = Style::from_name(scope.params().style()).palette(&class);
        let color = Class::from_symbols(&class).standard_color();
        let name_font = label::FontBuilder::normal(color, fonts::SIZE_M);
        let km_font = label::FontBuilder::normal(color, fonts::SIZE_XS);

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

    // Renders a stroke.
    //
    // ```text
    // stroke(width: distance, color: color, path: path)
    // ```
    ("stroke", &|pos, args, scope, features, err| {
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
        let (width, wpos) = args.next().unwrap().into_distance(err)?;
        if width.world.is_some() {
            err.add(
                wpos, "stroke width cannot have a world component"
            );
            return Err(Failed)
        }
        let width = width.canvas.unwrap_or(0.);
        let color = args.next().unwrap().into_color(err)?.0;
        let path = args.next().unwrap().into_path(err)?.0;
        features.insert(
            features::Contour::new(
                path, features::contour::simple(color, width)
            ),
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
        let style = Style::from_name(scope.params().style());
        let symbols = args.next().unwrap().into_symbol_set(err)?.0;
        let path = args.next().unwrap().into_path(err)?.0;

        let track_rule = TrackContour::new(style, false, &symbols,);
        let class = track_rule.class();
        let track_rule = track_rule.into_rule();
        let shade_rule = TrackShading::new(&symbols).into_rule();
        features.insert(
            features::Contour::new(path.clone(), track_rule),
            scope.params().detail(pos, err)?,
            scope.params().layer() - 0.1 + class.layer_offset(),
        );
        features.insert(
            features::Contour::new(path, shade_rule),
            scope.params().detail(pos, err)?,
            scope.params().layer() - 100. + class.layer_offset(),
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

