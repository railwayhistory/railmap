//! All our procedures.

use crate::features;
use crate::features::contour::RenderContour;
use crate::features::marker::RenderMarker;
use crate::features::path::Position;
use crate::import::{ast, eval};
use crate::import::Failed;
use super::class::Class;
use super::feature::area::AreaContour;
use super::feature::border::BorderContour;
use super::feature::markers::StandardMarker;
use super::feature::track::{TrackCasing, TrackClass, TrackContour};
use super::feature::guide::GuideContour;
use super::feature::label;


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
    //
    ("area", &|pos, args, scope, features, err| {
        let [class, path] = args.into_positionals(err)?;
        let class = Class::from_arg(class, err)?;
        let path = path.into_path(err)?.0;
        features.insert(
            features::Contour::new(path, AreaContour::new(class).into_rule()),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),

    // Draws a badge.
    //
    // ```text
    // badge([properties: symbol-set,] position: position, layout: layout)
    // ```
    ("badge", &|pos, args, scope, features, err| {
        let (properties, position, layout) = label_args(
            args, err, Default::default()
        )?;
        features.insert(
            layout.into_label(position, true, properties.into()),
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
        let [class, path] = args.into_positionals(err)?;
        let rule = BorderContour::from_arg(class, err)?.into_rule();
        let path = path.into_path(err)?.0;
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
        let [class, path] = args.into_positionals(err)?;
        let class = TrackClass::from_arg(class, err)?;
        let layer_offset = class.class().layer_offset();
        let path = path.into_path(err)?.0;
        let track_rule = TrackCasing::new(class).into_rule();
        features.insert(
            features::Contour::new(path.clone(), track_rule),
            scope.params().detail(pos, err)?,
            scope.params().layer() - 0.1 + layer_offset,
        );
        Ok(())
    }),

    // Draws a line attaching a label to something.
    //
    // ```text
    // guide(class: symbol-set, path: path)
    // ```
    ("guide", &|pos, args, scope, features, err| {
        let [class, path] = args.into_positionals(err)?;
        let path = path.into_path(err)?.0;
        let rule = GuideContour::from_arg(class, err)?.into_rule();
        features.insert(
            features::Contour::new(path, rule),
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
        let [class, position] = args.into_positionals(err)?;
        let rule = StandardMarker::from_arg(class, err);
        let position = position.into_position(err)?.0;
        features.insert(
            features::Marker::new(position, rule?.into_rule()),
            scope.params().detail(pos, err)?,
            scope.params().layer() - 0.2,
        );
        Ok(())
    }),

    // Draws a label.
    //
    // ```text
    // label([properties: symbol-set,] position: position, layout: layout)
    // ```
    ("label", &|pos, args, scope, features, err| {
        let (properties, position, layout) = label_args(
            args, err, Default::default(),
        )?;
        features.insert(
            layout.into_label( position, false, properties.into()),
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
        let (properties, position, layout) = label_args(
            args, err, label::Properties::with_size(label::FontSize::Badge)
        )?;
        let layout = label::LayoutBuilder::hbox(
            label::Align::Center, label::Align::Center, Default::default(),
            vec![layout]
        );
        features.insert(
            layout.into_label(position, true, properties),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),

    // Draws a platform.
    //
    ("platform", &|pos, args, scope, features, err| {
        let [class, path] = args.into_positionals(err)?;
        let class = Class::from_arg(class, err)?;
        let path = path.into_path(err)?.0;
        features.insert(
            features::Contour::new(path, AreaContour::new(class).into_rule()),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
        );
        Ok(())
    }),

    // Renders a one-line label with small text.
    //
    // ```text
    // slabel(class: symbol-set, position, text: text|layout)
    // ```
    ("slabel", &|pos, args, scope, features, err| {
        let [class, position, layout] = args.into_positionals(err)?;

        let mut class = class.into_symbol_set(err)?.0;
        let properties = label::PropertiesBuilder::from_symbols(&mut class);
        let (halign, valign) = if class.take("top") {
            (label::Align::Center, label::Align::End)
        }
        else if class.take("left") {
            (label::Align::End, label::Align::Ref)
        }
        else if class.take("bottom") {
            (label::Align::Center, label::Align::Start)
        }
        else if class.take("right") {
            (label::Align::Start, label::Align::Ref)
        }
        else {
            // XXX DROP THIS AND MAKE IT AN ERROR.
            (label::Align::Start, label::Align::Ref)
        };

        let position = position.into_position(err);
        let text = layout.into_layout(err);

        class.check_exhausted(err)?;
        let position = position?.0;
        let text = text?.0;

        features.insert(
            label::LayoutBuilder::hbox(
                halign, valign, properties, vec![text]
            ).into_label(
                position, false,
                label::Properties::with_size(label::FontSize::Small),
            ),
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
    ("station", &|_pos, args, scope, features, err| {
        let [class, position, name, km] = args.into_positionals(err)?;

        let symbols = class.into_symbol_set(err);
        let position = position.into_position(err);
        let name = name.into_layout(err);
        let km = km.into_layout(err);

        let (mut symbols, pos) = symbols?;
        let position = position?.0;
        let mut name = name?.0;
        let mut km = km?.0;

        name.rebase_properties(
            &label::PropertiesBuilder::with_size(label::FontSize::Medium)
        );
        km.rebase_properties(
            &label::PropertiesBuilder::with_size(label::FontSize::Xsmall)
        );

        let properties = label::Properties::with_class(
            Class::from_symbols(&mut symbols)
        );

        let halign = if symbols.take("left_align") {
            label::Align::Start
        }
        else if symbols.take("right_align") {
            label::Align::End
        }
        else {
            label::Align::Center
        };

        let layout = if symbols.take("top") {
            label::LayoutBuilder::vbox(
                halign, label::Align::End, Default::default(), vec![name, km]
            )
        }
        else if symbols.take("left") {
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
        else if symbols.take("bottom") {
            label::LayoutBuilder::vbox(
                halign, label::Align::Start, Default::default(), vec![name, km]
            )
        }
        else if symbols.take("right") {
            label::LayoutBuilder::hbox(
                label::Align::Start, label::Align::Ref, Default::default(),
                vec![
                    label::LayoutBuilder::vbox(
                        halign, label::Align::Ref, Default::default(),
                        vec![name, km]
                    )
                ]
            )
        }
        else {
            err.add(pos, "missing attachment direction");
            return Err(Failed)
        };
        symbols.check_exhausted(err)?;

        features.insert(
            layout.into_label(position, false, properties),
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
        let [class, path] = args.into_positionals(err)?;
        let class = TrackClass::from_arg(class, err)?;
        let layer_offset = class.class().layer_offset();
        let path = path.into_path(err)?.0;
        let track_rule = TrackContour::new(class).into_rule();
        features.insert(
            features::Contour::new(path, track_rule),
            scope.params().detail(pos, err)?,
            scope.params().layer() - 0.1 + layer_offset,
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


//------------ HelperFunctions ----------------------------------------------

/// Converts an argument list into the standard 2/3 label args.
///
/// This is basically:
///
/// ```text
/// ([class: symbol-set,] position: position, layout: Layout)
/// ```
fn label_args(
    args: eval::ArgumentList,
    err: &mut eval::Error,
    mut base_properties: label::Properties
) -> Result<(label::Properties, Position, label::LayoutBuilder), Failed> {
    let args = args.into_var_positionals(err,  |args, err| {
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
    let properties = if three {
        let mut symbols = args.next().unwrap().into_symbol_set(err)?.0;
        let properties = label::PropertiesBuilder::from_symbols(&mut symbols);
        if symbols.take("linenum") {
            base_properties.enable_linenum()
        }
        symbols.check_exhausted(err)?;
        base_properties.update(&properties)
    }
    else {
        base_properties
    };
    let position = args.next().unwrap().into_position(err)?.0;
    let layout = args.next().unwrap().into_layout(err)?.0;
    Ok((properties, position, layout))
}


