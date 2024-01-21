//! All our procedures.

#![allow(unused_imports)]

use femtomap::layout::Align;
use femtomap::path::{Edge, Position, Trace};
use femtomap::feature::FeatureSetBuilder;
use crate::oldimport::{ast, eval};
use crate::oldimport::Failed;
use super::class::Class;
use super::feature::Feature;
use super::feature::area::{AreaContour, PlatformContour};
use super::feature::border::BorderContour;
use super::feature::dot::DotMarker;
use super::feature::label::Anchor;
use super::feature::markers::StandardMarker;
use super::feature::track::{TrackCasing, TrackClass, TrackContour};
use super::feature::guide::GuideContour;
use super::feature::label;
use super::theme::Railwayhistory;
use super::units::resolve_unit;


const PROCEDURES: &[(
    &str,
    &dyn Fn(
        ast::Pos,
        eval::ArgumentList<Railwayhistory>,
        &eval::Scope<Railwayhistory>,
        &mut FeatureSetBuilder<Feature>,
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
            scope.params().layer(),
            -100,
        );
        Ok(())
    }),

    // Draws a badge.
    //
    // ```text
    // badge([properties: symbol-set,] position: position, layout: layout)
    // ```
    ("badge", &|pos, args, scope, features, err| {
        let (label_properties, properties, position, layout) = label_args(
            false, args, err,
        )?;
        features.insert(
            label::Feature::new(
                layout, label_properties,
                position, true,
                properties
            ).into(),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
            100,
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
            -500,
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
            scope.params().layer(),
            -2000 + layer_offset,
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
            scope.params().layer(),
            200,
        );
        Ok(())
    }),

    // Draws a label.
    //
    // ```text
    // label([properties: symbol-set,] position: position, layout: layout)
    // ```
    ("label", &|pos, args, scope, features, err| {
        let (label_properties, properties, position, layout) = label_args(
            false, args, err,
        )?;
        features.insert(
            label::Feature::new(
                layout, label_properties,
                position, false, properties.into()
            ).into(),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
            200,
        );
        Ok(())
    }),

    // Renders a badge containing a line label.
    //
    // ```text
    // line_badge(class: symbol-set, position: position, text: Text)
    // ```
    ("line_badge", &|pos, args, scope, features, err| {
        let (label_props, mut layout_props, position, mut layout) = label_args(
            true, args, err,
        )?;
        layout_props.set_layout_type(
            label::LayoutType::BadgeFrame
        );
        layout_props.set_packed(true);
        layout.properties_mut().set_layout_type(label::LayoutType::Framed);

        let mut layout = label::Layout::hbox(
            Align::Center, Align::Center, layout_props,
            vec![layout]
        );
        layout.properties_mut().set_layout_type(label::LayoutType::TextFrame);
        
        features.insert(
            label::Feature::new(
                layout, label_props,
                position, true,
                Default::default(),
            ).into(),
            scope.params().detail(pos, err)?,
            scope.params().layer() + 1,
            100,
        );
        Ok(())
    }),

    // Draws a text box connected to a line with a guide.
    //
    // ```text
    // line_box(
    //     class: symbol-set, position: position,
    //     [text-shift: distance,]
    //     text: Text
    // )
    // ```
    //
    // Classes:
    //
    // * `:left`, `:right` for the direction of the guide.
    // * `:n`, `:e`, `:s`, `:w`: the compass direction where the label is
    //   anchored.
    ("line_box", &|pos, args, scope, features, err| {
        // Get the arguments.
        let args = args.into_var_positionals(err,  |args, err| {
            if args.positional().len() == 3 || args.positional().len() == 4 {
                Ok(true)
            }
            else {
                err.add(
                    args.pos(),
                    "expected 3 or 4 positional arguments"
                );
                Err(Failed)
            }
        }).map_err(|_| Failed)?;

        let four = args.len() > 3;
        let mut args = args.into_iter();
        let mut symbols = args.next().unwrap().into_symbol_set(err)?;
        let position = args.next().unwrap().into_position(err)?.0;
        let shift = if four {
            Some(args.next().unwrap().into_vector(err)?.0)
        }
        else {
            None
        };
        let layout = label::layout_from_expr(
            args.next().unwrap(), err
        )?;

        // Tear symbols apart.
        let left = if symbols.take("left") {
            true
        }
        else if symbols.take("right") {
            false
        }
        else {
            err.add(symbols.pos(), "missing direction ':left' or ':right'");
            return Err(Failed)
        };
        let anchor = match Anchor::from_symbols(&mut symbols) {
            Some(anchor) => anchor,
            None => {
                err.add(symbols.pos(), "missing label anchor direction");
                return Err(Failed)
            }
        };
        let mut properties = label::LayoutProperties::from_symbols(
            &mut symbols
        );
        properties.set_layout_type(label::LayoutType::TextFrame);
        properties.set_size(label::FontSize::Small);
        let double = symbols.take("double");
        symbols.check_exhausted(err)?;

        // Get the positions for things.
        let (pos0, pos1) = if double {
            (
                Some(position.sideways(
                    resolve_unit(
                        if left { -0.5 } else { 0.5 },
                        "dt"
                    ).unwrap()
                )),
                position.sideways(
                    resolve_unit(
                        if left { 0.5 } else { -0.5 },
                        "dt"
                    ).unwrap()
                )
            )
        }
        else {
            (None, position.clone())
        };
        let pos2 = position.sideways(
            resolve_unit(
                if double {
                    if left { 5.5 } else { -5.5 }
                }
                else {
                    if left { 5.0 } else { -5.0 }
                },
                "dt"
            ).unwrap()
        );
        let mut pos3 = pos2.clone();
        if let Some(shift) = shift {
            pos3.shift_assign(shift);
        }

        let (halign, valign) = {
            use self::Anchor::*;
            use self::Align::*;

            match anchor {
                North => (Center, Start),
                NorthEast => (End, Start),
                East => (End, Center),
                SouthEast => (End, End),
                South => (Center, End),
                SouthWest => (Start, End),
                West => (Start, Center),
                NorthWest => (Start, Start)
            }
        };

        // Build the dots
        if let Some(pos0) = pos0.as_ref() {
            features.insert(
                Feature::Dot(DotMarker::guide(
                    properties.class().clone(),
                    pos0.clone(),
                )),
                scope.params().detail(pos, err)?,
                scope.params().layer(),
                200,
            );
        }
        features.insert(
            Feature::Dot(DotMarker::guide(
                properties.class().clone(),
                pos1.clone(),
            )),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
            200,
        );

        // Build the guide
        let mut trace = Trace::new();
        if let Some(pos0) = pos0 {
            trace.push_edge(1., 1., Edge::new(pos0, pos1.clone()));
        }
        trace.push_edge(1., 1., Edge::new(pos1, pos2));
        features.insert(
            Feature::Guide(GuideContour::new(
                properties.class().clone(), true, false, trace
            )),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
            200,
        );

        // Build the label.
        features.insert(
            label::Feature::new(
                label::Layout::hbox(
                    halign, valign, properties, vec![layout],
                ),
                label::LabelProperties::new_linenum(), pos3, false,
                Default::default(),
            ).into(),
            scope.params().detail(symbols.pos(), err)?,
            scope.params().layer(),
            200,
        );

        Ok(())
    }),

    // Draws a line label connected with a guide.
    //
    // ```text
    // line_label(
    //     class: symbol-set, position: position,
    //     [text-shift: distance,]
    //     text: Text
    // )
    // ```
    //
    // Classes:
    //
    // * `:left`, `:right` for the direction of the guide.
    // * `:n`, `:e`, `:s`, `:w`: the compass direction where the label is
    //   anchored.
    ("line_label", &|pos, args, scope, features, err| {
        // Get the arguments.
        let args = args.into_var_positionals(err,  |args, err| {
            if args.positional().len() == 3 || args.positional().len() == 4 {
                Ok(true)
            }
            else {
                err.add(
                    args.pos(),
                    "expected 3 or 4 positional arguments"
                );
                Err(Failed)
            }
        }).map_err(|_| Failed)?;

        let four = args.len() > 3;
        let mut args = args.into_iter();
        let mut symbols = args.next().unwrap().into_symbol_set(err)?;
        let position = args.next().unwrap().into_position(err)?.0;
        let shift = if four {
            Some(args.next().unwrap().into_vector(err)?.0)
        }
        else {
            None
        };
        let mut layout = label::layout_from_expr(
            args.next().unwrap(), err
        )?;
        layout.properties_mut().set_layout_type(label::LayoutType::Framed);

        // Tear symbols apart.
        let left = if symbols.take("left") {
            true
        }
        else if symbols.take("right") {
            false
        }
        else {
            err.add(symbols.pos(), "missing direction ':left' or ':right'");
            return Err(Failed)
        };
        let anchor = match Anchor::from_symbols(&mut symbols) {
            Some(anchor) => anchor,
            None => {
                err.add(symbols.pos(), "missing label anchor direction");
                return Err(Failed)
            }
        };
        let double = symbols.take("double");
        let mut properties = label::LayoutProperties::from_symbols(
            &mut symbols
        );
        symbols.check_exhausted(err)?;
        properties.set_layout_type(label::LayoutType::TextFrame);
        properties.set_packed(true);

        // Get the positions for things.
        let pos1 = position.sideways(
            resolve_unit(
                if left {
                    //if double { 1.0 } else { 0.8 }
                    if double { 0.2 } else { 0. }
                } else {
                    //if double { -1.0 } else { -0.8 }
                    if double { -0.2 } else { 0. }
                },
                "dt"
            ).unwrap()
        );
        let pos2 = position.sideways(
            resolve_unit(if left { 3.0 } else { -3.0 }, "dt").unwrap()
        );
        let pos3 = 3.0; /*{
            use self::Anchor::*;

            match anchor {
                North | South => 3.0,
                East | West => 3.5,
                _ => 3.8,
            }
        };*/
        let mut pos3 = position.sideways(
            resolve_unit(if left { pos3 } else { -pos3 }, "dt").unwrap()
        );
        if let Some(shift) = shift {
            pos3.shift_assign(shift)
        }
        let (halign, valign) = {
            use self::Anchor::*;
            use self::Align::*;

            match anchor {
            /*
                North => (Center, Start),
                NorthEast | East | SouthEast => (End, Center),
                South => (Center, End),
                SouthWest | West | NorthWest => (Start, Center)
            */
                North => (Center, Start),
                NorthEast => (End, Start),
                East => (End, Center),
                SouthEast => (End, End),
                South => (Center, End),
                SouthWest => (Start, End),
                West => (Start, Center),
                NorthWest => (Start, Start),
            }
        };

        // Build the guide.
        let mut trace = Trace::new();
        trace.push_edge(1., 1., Edge::new(pos1, pos2));
        features.insert(
            Feature::Guide(GuideContour::new(
                properties.class().clone(), true, false, trace
            )),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
            200,
        );

        // Build the label.
        let (lprop, bprop) = label::LabelProperties::default_pair(true);
        features.insert(
            label::Feature::new(
                label::Layout::hbox(
                    halign, valign, properties, vec![layout],
                ),
                lprop, pos3, false, bprop
            ).into(),
            scope.params().detail(symbols.pos(), err)?,
            scope.params().layer(),
            200,
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
        let mut class = class.into_symbol_set(err)?;
        let position = position.into_position(err)?.0;
        match DotMarker::try_from_arg(&mut class, position.clone(), err)? {
            Some(marker) => {
                let layer_offset = marker.class().layer_offset();
                features.insert(
                    Feature::Dot(marker),
                    scope.params().detail(pos, err)?,
                    scope.params().layer(),
                    100 + layer_offset,
                );
            }
            None => {
                let marker = StandardMarker::from_arg(class, position, err)?;
                let layer_offset = marker.class().layer_offset();
                features.insert(
                    Feature::Marker(marker),
                    scope.params().detail(pos, err)?,
                    scope.params().layer(),
                    -200 + layer_offset,
                );
            }
        }
        Ok(())
    }),

    // Renders a one-line label with small text.
    //
    // ```text
    // slabel(class: symbol-set, position, text: text|layout)
    // ```
    ("slabel", &|pos, args, scope, features, err| {
        let [class, position, layout] = args.into_positionals(err)?;

        let mut class = class.into_symbol_set(err)?;
        let properties = label::LayoutProperties::from_symbols(&mut class);
        let (h, v) = if let Some(anchor) = label::Anchor::from_symbols(
            &mut class
        ) {
            anchor.into_aligns()
        }
        else if class.take("top") {
            (Align::Center, Align::End)
        }
        else if class.take("left") {
            (Align::End, Align::Base)
        }
        else if class.take("bottom") {
            (Align::Center, Align::Start)
        }
        else if class.take("right") {
            (Align::Start, Align::Base)
        }
        else {
            err.add(pos, "missing anchor");
            return Err(Failed)
        };

        let position = position.into_position(err);
        let text = label::layout_from_expr(layout, err);

        class.check_exhausted(err)?;
        let position = position?.0;
        let text = text?;

        let layout = label::Layout::hbox(h, v, properties, vec![text]);

        features.insert(
            label::Feature::new(layout, Default::default(), position, false,
                label::LayoutProperties::with_size(label::FontSize::Small),
            ).into(),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
            1000,
        );
        Ok(())
    }),

    // Renders a station dot.
    //
    // ```text
    // statdot(marker: symbol-set, position: position)
    // ```
    ("statdot", &|pos, args, scope, features, err| {
        let [class, position] = args.into_positionals(err)?;
        let class = class.into_symbol_set(err)?;
        let position = position.into_position(err)?.0;
        let marker = DotMarker::from_arg(class, position, err)?;
        let layer_offset = marker.class().layer_offset();
        features.insert(
            Feature::Dot(marker),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
            layer_offset + 100,
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
        let name = label::layout_from_expr(name, err);
        let km = label::layout_from_expr(km, err);

        let mut symbols = symbols?;
        let position = position?.0;
        let mut name = name?;
        let mut km = km?;

        name.properties_mut().update(
            &label::LayoutProperties::with_size(label::FontSize::Medium)
        );
        km.properties_mut().update(
            &label::LayoutProperties::with_size(label::FontSize::Xsmall)
        );
        let properties = label::LayoutProperties::with_class(
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
            label::Layout::vbox(
                halign, Align::End, Default::default(),
                vec![name, km]
            )
        }
        else if symbols.take("left") {
            label::Layout::hbox(
                Align::End, Align::Base, Default::default(),
                vec![
                    label::Layout::vbox(
                        halign, Align::Base, Default::default(),
                        vec![name, km]
                    )
                ]
            )
        }
        else if symbols.take("bottom") {
            label::Layout::vbox(
                halign, Align::Start, Default::default(),
                vec![name, km]
            )
        }
        else if symbols.take("right") {
            label::Layout::hbox(
                Align::Start, Align::Base, Default::default(),
                vec![
                    label::Layout::vbox(
                        halign, Align::Base, Default::default(),
                        vec![name, km]
                    )
                ]
            )
        }
        else {
            err.add(symbols.pos(), "missing attachment direction");
            return Err(Failed)
        };
        symbols.check_exhausted(err)?;

        features.insert(
            label::Feature::new(
                layout, Default::default(), position, false, properties
            ).into(),
            scope.params().detail(symbols.pos(), err)?,
            scope.params().layer(),
            200,
        );
        Ok(())
    }),

    // Renders a bit of track.
    //
    // ```text
    // track(class: symbol-set, path: path)
    // ```
    ("track", &|pos, args, scope, features, err| {
        let [class_symbols, trace] = args.into_positionals(err)?;
        let mut class_symbols = class_symbols.into_symbol_set(err)?;
        let class = TrackClass::from_symbols(&mut class_symbols);
        let casing = class_symbols.take("casing");
        class_symbols.check_exhausted(err)?;
        let layer_offset = class.class().layer_offset();
        let trace = trace.into_path(err)?.0;
        features.insert(
            Feature::Track(TrackContour::new(class, casing, trace)),
            scope.params().detail(pos, err)?,
            scope.params().layer(),
            layer_offset,
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
        features: &mut FeatureSetBuilder<Feature>,
        err: &mut eval::Error,
    ) -> Result<(), Failed> {
        (*PROCEDURES[self.0].1)(pos, args, scope, features, err)
    }
}


//------------ Helper Functions ----------------------------------------------

/// Converts an argument list into the standard 2/3 label args.
///
/// This is basically:
///
/// ```text
/// ([class: symbol-set,] position: position, layout: Layout)
/// ```
fn label_args(
    linenum: bool,
    args: eval::ArgumentList<Railwayhistory>,
    err: &mut eval::Error,
) -> Result<
    (
        label::LabelProperties, label::LayoutProperties,
        Position, label::Layout,
    ),
    Failed
>
{
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
    let (label_properties, properties) = if three {
        label::LabelProperties::from_arg(linenum, args.next().unwrap(), err)?
    }
    else {
        label::LabelProperties::default_pair(linenum)
    };
    let position = args.next().unwrap().into_position(err)?.0;
    let layout = label::layout_from_expr(args.next().unwrap(), err)?;
    Ok((label_properties, properties, position, layout))
}

