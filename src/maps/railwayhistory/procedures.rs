//! All our procedures.

use crate::import::{ast, eval};
use crate::import::Failed;
use crate::render::feature::FeatureSet;
use crate::render::label::Align;
use crate::render::path::Position;
use super::class::Class;
use super::feature::Feature;
use super::feature::area::AreaContour;
use super::feature::border::BorderContour;
use super::feature::markers::StandardMarker;
use super::feature::track::{TrackCasing, TrackClass, TrackContour};
use super::feature::guide::GuideContour;
use super::feature::label;
use super::theme::Railwayhistory;


const PROCEDURES: &[(
    &str,
    &dyn Fn(
        ast::Pos,
        eval::ArgumentList<Railwayhistory>,
        &eval::Scope<Railwayhistory>,
        &mut FeatureSet<Railwayhistory>,
        &mut eval::Error
    ) -> Result<(), Failed>
)] = &[
    // Draws an area.
    //
    ("area", &|pos, args, scope, features, err| {
        let [class, trace] = args.into_positionals(err)?;
        let class = Class::from_arg(class, err)?;
        let trace = trace.into_path(err)?.0;
        features.insert(
            Feature::Area(AreaContour::new(class, trace)),
            scope.params().detail(pos, err)?,
            scope.params().layer(), 1,
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
            layout.into_feature(position, true, properties.into()),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
            1,
        );
        Ok(())
    }),

    // Draws a border.
    //
    // ```text
    // border(class: symbol-set, path: path)
    // ```
    ("border", &|pos, args, scope, features, err| {
        let [class, trace] = args.into_positionals(err)?;
        let trace = trace.into_path(err)?.0;
        features.insert(
            Feature::Border(BorderContour::from_arg(class, trace, err)?),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
            1,
        );
        Ok(())
    }),

    // Renders casing for a track.
    //
    // ```text
    // casing(class: symbol-set, path: path)
    // ```
    ("casing", &|pos, args, scope, features, err| {
        let [class, trace] = args.into_positionals(err)?;
        let class = TrackClass::from_arg(class, err)?;
        let layer_offset = class.class().layer_offset();
        let trace = trace.into_path(err)?.0;
        features.insert(
            Feature::Casing(TrackCasing::new(class, trace)),
            scope.params().detail(pos, err)?,
            scope.params().layer() - 0.1 + layer_offset,
            1,
        );
        Ok(())
    }),

    // Draws a line attaching a label to something.
    //
    // ```text
    // guide(class: symbol-set, path: path)
    // ```
    ("guide", &|pos, args, scope, features, err| {
        let [class, trace] = args.into_positionals(err)?;
        let trace = trace.into_path(err)?.0;
        features.insert(
            Feature::Guide(GuideContour::from_arg(class, trace, err)?),
            scope.params().detail(pos, err)?,
            scope.params().layer(), 1,
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
        let position = position.into_position(err)?.0;
        let marker = StandardMarker::from_arg(class, position, err)?;
        let layer_offset = marker.class().layer_offset();
        features.insert(
            Feature::Marker(marker),
            scope.params().detail(pos, err)?,
            scope.params().layer() - 0.3 + layer_offset,
            1,
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
            layout.into_feature(position, false, properties.into()),
            scope.params().detail(pos, err)?,
            scope.params().layer(), 1,
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
        let layout = label::LayoutBuilder::badge_frame(
            label::PropertiesBuilder::packed(),
            label::LayoutBuilder::hbox(
                Align::Center, Align::Center, Default::default(),
                vec![layout]
            ),
        );
        features.insert(
            layout.into_feature(position, true, properties),
            scope.params().detail(pos, err)?,
            scope.params().layer(), 1,
        );
        Ok(())
    }),

    // Draws a platform.
    //
    ("platform", &|pos, args, scope, features, err| {
        let [class, trace] = args.into_positionals(err)?;
        let class = Class::from_arg(class, err)?;
        let layer_offset = class.layer_offset();
        let trace = trace.into_path(err)?.0;
        features.insert(
            Feature::Area(AreaContour::new(class, trace)),
            scope.params().detail(pos, err)?,
            scope.params().layer() - 0.2 + layer_offset,
            1,
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
            (Align::Center, Align::End)
        }
        else if class.take("left") {
            (Align::End, Align::Ref)
        }
        else if class.take("bottom") {
            (Align::Center, Align::Start)
        }
        else if class.take("right") {
            (Align::Start, Align::Ref)
        }
        else {
            // XXX DROP THIS AND MAKE IT AN ERROR.
            (Align::Start, Align::Ref)
        };

        let position = position.into_position(err);
        let text = label::LayoutBuilder::from_expr(layout, err);

        class.check_exhausted(err)?;
        let position = position?.0;
        let text = text?;

        features.insert(
            label::LayoutBuilder::hbox(
                halign, valign, properties, vec![text]
            ).into_feature(
                position, false,
                label::Properties::with_size(label::FontSize::Small),
            ),
            scope.params().detail(pos, err)?,
            scope.params().layer(), 1,
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
        let name = label::LayoutBuilder::from_expr(name, err);
        let km = label::LayoutBuilder::from_expr(km, err);

        let (mut symbols, pos) = symbols?;
        let position = position?.0;
        let mut name = name?;
        let mut km = km?;

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
            Align::Start
        }
        else if symbols.take("right_align") {
            Align::End
        }
        else {
            Align::Center
        };

        let layout = if symbols.take("top") {
            label::LayoutBuilder::vbox(
                halign, Align::End, Default::default(),
                vec![name, km]
            )
        }
        else if symbols.take("left") {
            label::LayoutBuilder::hbox(
                Align::End, Align::Ref, Default::default(),
                vec![
                    label::LayoutBuilder::vbox(
                        halign, Align::Ref, Default::default(),
                        vec![name, km]
                    )
                ]
            )
        }
        else if symbols.take("bottom") {
            label::LayoutBuilder::vbox(
                halign, Align::Start, Default::default(),
                vec![name, km]
            )
        }
        else if symbols.take("right") {
            label::LayoutBuilder::hbox(
                Align::Start, Align::Ref, Default::default(),
                vec![
                    label::LayoutBuilder::vbox(
                        halign, Align::Ref, Default::default(),
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
            layout.into_feature(position, false, properties),
            scope.params().detail(pos, err)?,
            scope.params().layer(), 1,
        );
        Ok(())
    }),

    // Renders a bit of track.
    //
    // ```text
    // track(class: symbol-set, path: path)
    // ```
    ("track", &|pos, args, scope, features, err| {
        let [class, trace] = args.into_positionals(err)?;
        let class = TrackClass::from_arg(class, err)?;
        //let is_tunnel = class.class().surface().is_tunnel();
        let layer_offset = class.class().layer_offset();
        let trace = trace.into_path(err)?.0;
        features.insert(
            Feature::Track(TrackContour::new(class, trace)),
            scope.params().detail(pos, err)?,
            scope.params().layer() - 0.1 + layer_offset,
            1, //if is_tunnel { 2 } else { 1 },
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
        &self,
        pos: ast::Pos,
        args: eval::ArgumentList<Railwayhistory>,
        scope: &eval::Scope<Railwayhistory>,
        features: &mut FeatureSet<Railwayhistory>,
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
    args: eval::ArgumentList<Railwayhistory>,
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
    let layout = label::LayoutBuilder::from_expr(args.next().unwrap(), err)?;
    Ok((properties, position, layout))
}

