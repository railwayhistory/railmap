//! Procedures provided to the map language.

use femtomap::import::ast::Pos;
use femtomap::import::eval::{EvalErrors, Failed, SymbolSet};
use femtomap::layout::Align;
use femtomap::path::{Distance, Edge, Position, Trace};
use crate::railway::class::Railway;
use crate::railway::feature::{label, marker};
use crate::railway::feature::{FeatureSetBuilder, StoreBuilder};
use crate::railway::feature::border::BorderContour;
use crate::railway::feature::area::{AreaContour, PlatformContour};
use crate::railway::feature::dot::DotMarker;
use crate::railway::feature::guide::GuideContour;
use crate::railway::feature::label::{
    Anchor, FontSize, Label, Layout, LayoutProperties, TextAnchor,
};
use crate::railway::feature::track::{TrackCasing, TrackClass, TrackContour};
use super::units;
use super::eval::{ArgumentList, Scope, ScopeExt};

//------------ eval ----------------------------------------------------------

/// Evaluates a procedure call.
pub fn eval<'s>(
    name: &str,
    args: ArgumentList,
    scope: &Scope<'s>,
    pos: Pos,
    err: &mut EvalErrors,
) -> Result<(), Failed> {
    let procedure = match PROCEDURES.iter().find(|f| f.0 == name) {
        Some(procedure) => procedure,
        None => {
            err.add(pos, format!("undefined function '{}'", name));
            return Err(Failed)
        }
    };
    (procedure.1)(pos, args, scope, err)
}


//------------ Procedures ----------------------------------------------------

const PROCEDURES: &[(
    &str,
    &dyn for<'a> Fn(
        Pos,
        ArgumentList<'a>,
        &Scope<'a>,
        &mut EvalErrors,
    ) -> Result<(), Failed>
)] = &[
    // Draws an area.
    //
    ("area", &|pos, args, scope, err| {
        let [class, trace] = args.into_array(err)?;
        let class = Railway::from_arg(class, scope, err)?;
        let trace = trace.eval(err)?;
        scope.builtin().with_store(|store| {
            store.railway.insert(
                AreaContour::new(class, trace),
                scope.detail(pos, err)?,
                scope.layer(),
                //-100,
            );
            Ok(())
        })
    }),

    // Draws a badge.
    //
    // ```text
    // badge([properties: symbol-set,] position: position, layout: layout)
    // ```
    ("badge", &|pos, args, scope, err| {
        let args = BadgeArgs::from_args(args, scope, err)?;
        scope.builtin().with_store(|store| {
            args.features(store).insert(
                label::Label::new(
                    args.layout,
                    args.position,
                    true,
                    args.properties
                ),
                scope.detail(pos, err)?,
                scope.layer(),
            );
            Ok(())
        })
    }),

    // Draws a border.
    //
    // ```text
    // border(class: symbol-set, path: path)
    // ```
    ("border", &|pos, args, scope, err| {
        let [class, trace] = args.into_array(err)?;
        let trace = trace.eval(err)?;
        scope.builtin().with_store(|store| {
            store.borders.insert(
                BorderContour::from_arg(class, trace, err)?,
                scope.detail(pos, err)?,
                scope.layer(),
            );
            Ok(())
        })
    }),

    // Renders casing for a track.
    //
    // ```text
    // casing(class: symbol-set, path: path)
    // ```
    ("casing", &|pos, args, scope, err| {
        let [class, trace] = args.into_array(err)?;
        let class = TrackClass::from_arg(class, scope, err)?;
        let trace = trace.eval(err)?;
        scope.builtin().with_store(|store| {
            store.railway.insert(
                TrackCasing::new(class, trace),
                scope.detail(pos, err)?,
                scope.layer(),
            );
            Ok(())
        })
    }),

    // Draws a line attaching a label to something.
    //
    // ```text
    // guide(class: symbol-set, path: path)
    // ```
    ("guide", &|pos, args, scope, err| {
        let [class, trace] = args.into_array(err)?;
        let class = class.eval::<SymbolSet>(err);
        let trace = trace.eval(err)?;
        let mut class = class?;

        let linenum = class.take("linenum");
        let contour = GuideContour::from_symbols(class, trace, scope, err)?;

        scope.builtin().with_store(move |store| {
            if linenum {
                &mut store.line_labels
            }
            else {
                &mut store.railway
            }.insert(
                contour,
                scope.detail(pos, err)?,
                scope.layer(),
            );
            Ok(())
        })
    }),

    // Draws a label.
    //
    // ```text
    // label([properties: symbol-set,] position: position, layout: layout)
    // ```
    ("label", &|pos, args, scope, err| {
        let args = BadgeArgs::from_args(args, scope, err)?;
        scope.builtin().with_store(|store| {
            args.features(store).insert(
                label::Label::new(
                    args.layout,
                    args.position,
                    false,
                    args.properties
                ),
                scope.detail(pos, err)?,
                scope.layer(),
            );
            Ok(())
        })
    }),

    // Renders a badge containing a line label.
    //
    // ```text
    // line_badge(class: symbol-set, position: position, text: Text)
    // ```
    ("line_badge", &|pos, args, scope, err| {
        let label = line_badge(args, scope, err)?;
        scope.builtin().with_store(|store| {
            store.line_labels.insert(
                label,
                scope.detail(pos, err)?,
                scope.layer(),
            );
            Ok(())
        })
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
    ("line_box", &|pos, args, scope, err| {
        let (args, layout) = TextBoxArgs::from_args(args, scope, err)?;

        // Get the positions for things.
        let (pos0, pos1) = if args.double {
            (
                Some(args.position.sideways(
                    units::dt(if args.left { -0.5 } else { 0.5 })
                )),
                args.position.sideways(
                    units::dt(if args.left { 0.5 } else { -0.5 })
                )
            )
        }
        else {
            (None, args.position.clone())
        };
        let pos2 = args.position.sideways(
            units::dt(
                if args.double {
                    if args.left { 5.5 } else { -5.5 }
                }
                else {
                    if args.left { 5.0 } else { -5.0 }
                }
            )
        );
        let pos3 = pos2.clone().shift(args.shift);

        let (halign, valign) = {
            use self::Anchor::*;
            use self::Align::*;

            match args.anchor {
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

        scope.builtin().with_store(|store| {
            // Build the dots
            if let Some(pos0) = pos0.as_ref() {
                store.line_labels.insert(
                    DotMarker::guide(
                        args.properties.class().clone(),
                        pos0.clone(),
                    ),
                    scope.detail(pos, err)?,
                    scope.layer(),
                );
            }
            store.line_labels.insert(
                DotMarker::guide(
                    args.properties.class().clone(),
                    pos1.clone(),
                ),
                scope.detail(pos, err)?,
                scope.layer(),
            );

            // Build the guide
            let mut trace = Trace::new();
            if let Some(pos0) = pos0 {
                trace.push_edge(1., 1., Edge::new(pos0, pos1.clone()));
            }
            trace.push_edge(1., 1., Edge::new(pos1, pos2));
            store.line_labels.insert(
                GuideContour::new(
                    args.properties.class().clone(), true, trace
                ),
                scope.detail(pos, err)?,
                scope.layer(),
            );

            // Build the label.
            store.line_labels.insert(
                Label::new(
                    Layout::hbox(
                        halign, valign, args.properties, vec![layout],
                    ),
                    pos3, false,
                    Default::default(),
                ),
                scope.detail(pos, err)?,
                scope.layer(),
            );

            Ok(())
        })
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
    ("line_label", &|pos, args, scope, err| {
        let (args, layout) = TextBoxArgs::from_args(args, scope, err)?;
        let (contour, label) = line_label(args, layout, scope);
        scope.builtin().with_store(|store| {
            store.line_labels.insert(
                contour,
                scope.detail(pos, err)?,
                scope.layer(),
            );
            store.line_labels.insert(
                label,
                scope.detail(pos, err)?,
                scope.layer(),
            );
            Ok(())
        })
    }),

    ("line_tt_label", &|pos, args, scope, err| {
        let (args, line_layout, tt_layout) = TextBoxArgs::double_from_args(
            args, scope, err
        )?;
        let (l_contour, l_label) = line_label(args.clone(), line_layout, scope);
        let (tt_contour, tt_label) = line_label(args, tt_layout, scope);
        scope.builtin().with_store(|store| {
            store.line_labels.insert(
                l_contour,
                scope.detail(pos, err)?,
                scope.layer(),
            );
            store.line_labels.insert(
                l_label,
                scope.detail(pos, err)?,
                scope.layer(),
            );
            store.tt_labels.insert(
                tt_contour,
                scope.detail(pos, err)?,
                scope.layer(),
            );
            store.tt_labels.insert(
                tt_label,
                scope.detail(pos, err)?,
                scope.layer(),
            );
            Ok(())
        })
    }),

    // Draw a symbol.
    //
    // ```text
    // marker(marker: symbol-set, position: position)
    // ```
    ("marker", &|pos, args, scope, err| {
        let [class, position] = args.into_array(err)?;
        let class = class.eval::<SymbolSet>(err);
        let position = position.eval::<Position>(err)?;
        let mut class = class?;

        scope.builtin().with_store(|store| {
            match DotMarker::try_from_arg(
                &mut class, position.clone(), scope, err
            )? {
                Some(marker) => {
                    store.railway.insert(
                        marker,
                        scope.detail(pos, err)?,
                        scope.layer(),
                    );
                }
                None => {
                    let marker = marker::from_args(
                        class, position, scope, err
                    )?;
                    store.railway.insert(
                        marker,
                        scope.detail(pos, err)?,
                        scope.layer(),
                    );
                }
            }
            Ok(())
        })
    }),

    // Draws a platform.
    //
    ("platform", &|pos, mut args, scope, err| {
        let mut class_symbols = args.take_first_if_matches(
            err
        )?.unwrap_or_default();
        let [trace] = args.into_array(err)?;
        let class = Railway::from_symbols(&mut class_symbols, scope);
        class_symbols.check_exhausted(err)?;
        let trace = trace.eval(err)?;
        scope.builtin().with_store(|store| {
            store.railway.insert(
                PlatformContour::new(class, trace),
                scope.detail(pos, err)?,
                scope.layer(),
            );
            Ok(())
        })
    }),

    // Renders a label with small text.
    //
    // ```text
    // slabel([class: symbol-set,] position, text: text|layout)
    // ```
    ("slabel", &|pos, args, scope, err| {
        let (mut args, layout) = LabelArgs::from_args(args, scope, err)?;
        args.properties.update_size(FontSize::Small);
        let layout = Layout::hbox(
            args.anchor.h, args.anchor.v, args.properties, vec![layout]
        );

        scope.builtin().with_store(|store| {
            if args.linenum {
                &mut store.line_labels
            }
            else {
                &mut store.railway
            }.insert(
                Label::new(layout, args.position, false, Default::default()),
                scope.detail(pos, err)?,
                scope.layer(),
            );
            Ok(())
        })
    }),

    // Renders a station dot.
    //
    // ```text
    // statdot([marker: symbol-set,] position: position)
    // ```
    ("statdot", &|pos, args, scope, err| {
        let marker = match args.try_into_array() {
            Ok([class, position]) => {
                let class = class.eval(err)?;
                let position = position.eval(err)?;
                DotMarker::from_arg(
                    class, position, scope, err
                )?
            }
            Err(args) => match args.try_into_array() {
                Ok([position]) => {
                    let position = position.eval(err)?;
                    DotMarker::from_position(position, scope)?
                }
                Err(_) => {
                    err.add(pos, "expected 1 or 2 arguments");
                    return Err(Failed)
                }
            }
        };
        scope.builtin().with_store(|store| {
            store.railway.insert(
                marker,
                scope.detail(pos, err)?,
                scope.layer(),
            );
            Ok(())
        })
    }),

    // Renders a station label.
    //
    // ```text
    // station(class: symbol-set, position, name: text|layout, km: text|layout)
    // ```
    ("station", &|pos, args, scope, err| {
        let [class, position, name, km] = args.into_array(err)?;

        let class = class.eval::<SymbolSet>(err);
        let position = position.eval(err);
        let name = label::layout_from_expr(name, err);
        let km = label::layout_from_expr(km, err);

        let mut class = class?;
        let position = position?;
        let mut name = name?;
        let mut km = km?;

        name.properties_mut().update(
            &label::LayoutProperties::with_size(label::FontSize::Medium)
        );
        km.properties_mut().update(
            &label::LayoutProperties::with_size(label::FontSize::Xsmall)
        );
        let properties = label::LayoutProperties::with_class(
            Railway::from_symbols(&mut class, scope)
        );

        let halign = if class.take("left_align") {
            Align::Start
        }
        else if class.take("right_align") {
            Align::End
        }
        else {
            Align::Center
        };

        let layout = if class.take("top") {
            label::Layout::vbox(
                halign, Align::End, Default::default(),
                vec![name, km]
            )
        }
        else if class.take("left") {
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
        else if class.take("bottom") {
            label::Layout::vbox(
                halign, Align::Start, Default::default(),
                vec![name, km]
            )
        }
        else if class.take("right") {
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
            err.add(pos, "missing attachment direction");
            return Err(Failed)
        };
        class.check_exhausted(err)?;

        scope.builtin().with_store(|store| {
            store.railway.insert(
                label::Label::new(
                    layout, position, false, properties
                ),
                scope.detail(pos, err)?,
                scope.layer(),
            );
            Ok(())
        })
    }),

    // Renders a track.
    //
    // ```text
    // track([class: symbol-set, ]path: path)
    // ```
    ("track", &|pos, mut args, scope, err| {
        let mut class_symbols = args.take_first_if_matches(
            err
        )?.unwrap_or_default();
        let [trace] = args.into_array(err)?;
        let trace = trace.eval(err)?;
        let class = TrackClass::from_symbols(&mut class_symbols, scope);
        let casing = class_symbols.take("casing");
        class_symbols.check_exhausted(err)?;

        scope.builtin().with_store(|store| {
            store.railway.insert(
                TrackContour::new(class, casing, trace),
                scope.detail(pos, err)?,
                scope.layer(),
            );
            Ok(())
        })
    }),

    // Renders a badge containing a timetable label.
    //
    // ```text
    // tt_badge(class: symbol-set, position: position, text: Text)
    // ```
    ("tt_badge", &|pos, args, scope, err| {
        let label = line_badge(args, scope, err)?;
        scope.builtin().with_store(|store| {
            store.tt_labels.insert(
                label,
                scope.detail(pos, err)?,
                scope.layer(),
            );
            Ok(())
        })
    }),

    // Draws a timetable label connected with a guide.
    //
    // ```text
    // tt_label(
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
    ("tt_label", &|pos, args, scope, err| {
        let (args, layout) = TextBoxArgs::from_args(args, scope, err)?;
        let (contour, label) = line_label(args, layout, scope);
        scope.builtin().with_store(|store| {
            store.tt_labels.insert(
                contour,
                scope.detail(pos, err)?,
                scope.layer(),
            );
            store.tt_labels.insert(
                label,
                scope.detail(pos, err)?,
                scope.layer(),
            );
            Ok(())
        })
    }),
];


//============ Helpers =======================================================

//------------ line_badge, line_label ----------------------------------------

fn line_badge(
    args: ArgumentList,
    scope: &Scope,
    err: &mut EvalErrors,
) -> Result<label::Label, Failed> {
    let mut args = BadgeArgs::from_args(args, scope, err)?;
    args.properties.set_layout_type(
        label::LayoutType::BadgeFrame
    );
    args.properties.update_size(FontSize::Badge);
    args.properties.set_packed(true);
    args.layout.properties_mut().set_layout_type(label::LayoutType::Framed);

    let mut layout = label::Layout::hbox(
        Align::Center, Align::Center, args.properties,
        vec![args.layout]
    );
    layout.properties_mut().set_layout_type(label::LayoutType::TextFrame);
    
    Ok(label::Label::new(
        layout, 
        args.position,
        true,
        Default::default(),
    ))
}

fn line_label(
    mut args: TextBoxArgs,
    mut layout: Layout,
    scope: &Scope,
) -> (GuideContour, label::Label) {
    // Get the positions for things.
    let pos1 = args.position.sideways(
        units::dt(
            if args.left {
                //if double { 1.0 } else { 0.8 }
                if args.double { 0.2 } else { 0. }
            } else {
                //if double { -1.0 } else { -0.8 }
                if args.double { -0.2 } else { 0. }
            },
        )
    );
    let distance = if scope.min_detail() >= 3 { 4. }
    else { 3. };
    let pos2 = args.position.sideways(
        units::dt(if args.left { distance } else { -distance })
    );
    let pos3 = pos2.clone().shift(args.shift);

    let (halign, valign) = args.anchor.into_aligns();

    args.properties.set_packed(true);
    args.properties.set_layout_type(label::LayoutType::TextFrame);
    layout.properties_mut().set_layout_type(
        label::LayoutType::Framed
    );

    let mut trace = Trace::new();
    trace.push_edge(1., 1., Edge::new(pos1, pos2));

    (
        GuideContour::new(
            args.properties.class().clone(), true, trace
        ),
        label::Label::new(
            label::Layout::hbox(
                halign, valign, args.properties, vec![layout],
            ),
            pos3, false, LayoutProperties::with_size(FontSize::Badge),
        ),
    )
}


//------------ BadgeArgs -----------------------------------------------------

/// The arguments for the various label-making procedures.
struct BadgeArgs {
    /// Is this a line num label?
    ///
    /// This is legacy and should disappear.
    linenum: bool,

    /// The layout properties for the label.
    properties: label::LayoutProperties,

    /// The position of the label.
    position: Position,

    /// The actual layout.
    layout: label::Layout
}

impl BadgeArgs {
    /// Converts an argument list into the standard 2/3 label args.
    ///
    /// This is basically:
    ///
    /// ```text
    /// ([class: symbol-set,] position: position, layout: Layout)
    /// ```
    fn from_args(
        args: ArgumentList, scope: &Scope, err: &mut EvalErrors
    ) -> Result<Self, Failed> {
        let args = match args.try_into_array() {
            Ok([class, position, layout]) => {
                let class = class.eval::<SymbolSet>(err);
                let position = position.eval(err);
                let layout = label::layout_from_expr(layout, err);
                
                let mut class = class?;
                let position = position?;
                let layout = layout?;

                let linenum = class.take("linenum");
                let mut properties = label::LayoutProperties::from_symbols(
                    &mut class, scope
                );
                class.check_exhausted(err)?;

                if linenum {
                    properties.update_size(label::FontSize::Badge);
                }

                return Ok(Self { linenum, properties, position, layout })
            }
            Err(args) => args,
        };

        let args = match args.try_into_array() {
            Ok([position, layout]) => {
                let position = position.eval(err);
                let layout = label::layout_from_expr(layout, err);
                return Ok(Self {
                    linenum: false,
                    properties: label::LayoutProperties::from_scope(scope),
                    position: position?,
                    layout: layout?,
                })
            }
            Err(args) => args,
        };

        err.add(args.pos(), "expected 2 or 3 arguments");
        Err(Failed)
    }

    fn features<'s>(
        &self, store: &'s mut StoreBuilder
    ) -> &'s mut FeatureSetBuilder {
        if self.linenum {
            &mut store.line_labels
        }
        else {
            &mut store.railway
        }
    }
}


//------------ LabelArgs -----------------------------------------------------

/// The arguments for the various label-making procedures.
struct LabelArgs {
    /// Is this a line num label?
    ///
    /// This is legacy and should disappear.
    linenum: bool,

    /// Anchor of the box.
    anchor: TextAnchor,

    /// The layout properties for the label.
    properties: LayoutProperties,

    /// The position of the label.
    position: Position,
}

impl LabelArgs {
    /// Converts an argument list into the standard 2/3 label args.
    ///
    /// This is basically:
    ///
    /// ```text
    /// (class: symbol-set, position: position, layout: Layout)
    /// ```
    fn from_args(
        args: ArgumentList, scope: &Scope, err: &mut EvalErrors
    ) -> Result<(Self, Layout), Failed> {
        let [class, position, layout] = args.into_array(err)?;
        let class = class.eval::<SymbolSet>(err);
        let position = position.eval::<Position>(err);
        let layout = label::layout_from_expr(layout, err)?;
        let mut class = class?;
        let position = position?;

        let linenum = class.take("linenum");
        let anchor = match TextAnchor::from_symbols(&mut class) {
            Some(anchor) => anchor,
            None => {
                err.add(class.pos(), "missing text anchor");
                return Err(Failed)
            }
        };
        let properties = LayoutProperties::from_symbols(&mut class, scope);
        class.check_exhausted(err)?;

        Ok((Self { linenum, anchor, properties, position }, layout))
    }
}


//------------ TextBoxArgs ---------------------------------------------------

/// The arguments for the various text box-making procedures.
#[derive(Clone)]
struct TextBoxArgs {
    /// Is the box attached to the left?
    left: bool,

    /// Anchor of the box.
    anchor: Anchor,

    /// The layout properties.
    properties: LayoutProperties,

    /// Is this for a double track line?
    double: bool,

    /// The position of the label.
    position: Position,

    /// The text shift distance.
    shift: (Distance, Distance),
}

impl TextBoxArgs {
    /// Converts an argument list into the standard 2/3 label args.
    ///
    /// ```text
    /// line_box(
    ///     class: symbol-set, position: position,
    ///     [text-shift: distance,]
    ///     text: Text
    /// )
    /// ```
    ///
    /// Classes:
    ///
    /// * `:left`, `:right` for the direction of the guide.
    /// * `:n`, `:e`, `:s`, `:w`: the compass direction where the label is
    ///   anchored.
    /// 
    fn from_args(
        args: ArgumentList, scope: &Scope, err: &mut EvalErrors
    ) -> Result<(Self, Layout), Failed> {
        let args = match args.try_into_array() {
            Ok([class, position, shift, layout]) => {
                let class = class.eval(err);
                let position = position.eval(err);
                let shift = shift.eval(err);
                return Ok((
                    Self::from_all_args(
                        class?, position?, shift?, scope, err
                    )?,
                    label::layout_from_expr(layout, err)?,
                ))
            },
            Err(args) => args
        };

        let args = match args.try_into_array() {
            Ok([class, position, layout]) => {
                let class = class.eval(err);
                let position = position.eval(err);
                return Ok((
                    Self::from_all_args(
                        class?, position?, Default::default(), scope, err
                    )?,
                    label::layout_from_expr(layout, err)?,
                ))
            },
            Err(args) => args
        };

        err.add(args.pos(), "expected 3 or 4 arguments");
        Err(Failed)
    }

    fn double_from_args(
        args: ArgumentList, scope: &Scope, err: &mut EvalErrors
    ) -> Result<(Self, Layout, Layout), Failed> {
        let args = match args.try_into_array() {
            Ok([class, position, shift, l1, l2]) => {
                let class = class.eval(err);
                let position = position.eval(err);
                let shift = shift.eval(err);
                return Ok((
                    Self::from_all_args(
                        class?, position?, shift?, scope, err
                    )?,
                    label::layout_from_expr(l1, err)?,
                    label::layout_from_expr(l2, err)?,
                ))
            },
            Err(args) => args
        };

        let args = match args.try_into_array() {
            Ok([class, position, l1, l2]) => {
                let class = class.eval(err);
                let position = position.eval(err);
                return Ok((
                    Self::from_all_args(
                        class?, position?, Default::default(), scope, err
                    )?,
                    label::layout_from_expr(l1, err)?,
                    label::layout_from_expr(l2, err)?,
                ))
            },
            Err(args) => args
        };

        err.add(args.pos(), "expected 4 or 5 arguments");
        Err(Failed)
    }

    fn from_all_args(
        mut class: SymbolSet,
        position: Position,
        shift: (Distance, Distance),
        scope: &Scope,
        err: &mut EvalErrors,
    ) -> Result<Self, Failed> {
        // :left or :right
        let left = if class.take("left") {
            true
        }
        else if class.take("right") {
            false
        }
        else {
            err.add(class.pos(), "missing direction ':left' or ':right'");
            return Err(Failed)
        };

        // box anchor
        let anchor = match Anchor::from_symbols(&mut class) {
            Some(anchor) => anchor,
            None => {
                err.add(class.pos(), "missing label anchor direction");
                return Err(Failed)
            }
        };

        // every else from class
        let mut properties = LayoutProperties::from_symbols(&mut class, scope);
        properties.set_layout_type(label::LayoutType::TextFrame);

        // :double and then we should be empty.
        let double = class.take("double");
        class.check_exhausted(err)?;

        Ok(Self { left, anchor, properties, double, position, shift })
    }

}

