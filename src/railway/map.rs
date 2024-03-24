//! Binding it all together.

use std::str::FromStr;
use femtomap::render::Canvas;
use kurbo::{Point, Rect};
use crate::tile;
use crate::tile::{Surface, TileId, TileIdError};
use super::colors::ColorSet;
use super::feature::{FeatureSet, Stage, Store};
use super::style::{Style, StyleId};


//------------ Map -----------------------------------------------------------

/// All information necessary to render the railway layers.
pub struct Map {
    /// All the features to render.
    features: Store,

    /// The color set for rendering.
    colors: ColorSet,
}

impl Map {
    /// Creates a new map.
    pub fn new(features: Store) -> Self {
        Self { features, colors: Default::default() }
    }

    /// Renders a map tile.
    pub fn render(
        &self, tile_id: TileId, surface: &Surface
    ) -> Result<(), TileIdError> {
        let layer_id = LayerId::try_from(tile_id.layer)?;
        let style = Style::new(layer_id, &tile_id, &self.colors);
        let mut canvas = Canvas::new(surface);
        let size = tile_id.format.size();
        canvas.set_clip(Rect::new(0., 0., size, size));
        let shapes = layer_id.features(&self.features).shape(
            style.store_scale(),
            Self::feature_bounds(tile_id, &style).into(),
            &style, &canvas,
        );

        for group in shapes.layer_groups() {
            for stage in Stage::default() {
                group.iter().for_each(|shape| {
                    shape.shape().render(stage, &style, &mut canvas)
                });
            }
        }

        Ok(())
    }

    fn feature_bounds(id: TileId, style: &Style) -> Rect {
        let size = id.format.size();
        let scale = size * id.n();
        let feature_size = Point::new(size / scale, size / scale);
        let nw = id.nw();

        let correct = style.bounds_correction();
        let correct = Point::new(
            feature_size.x * correct,
            feature_size.y * correct,
        );

        Rect::new(
            nw.x - correct.x,
            nw.y - correct.y,
            nw.x + feature_size.x + correct.x,
            nw.y + feature_size.y + correct.y,
        )
    }
}


//------------ LayerId -------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LayerId {
    /// Electrification base map.
    El(ScriptId),

    /// Electrification line number map.
    ElNum,

    /// Passenger base map.
    Pax(ScriptId),

    /// Passenger timetable number map.
    PaxNum,

    /// Border map.
    Border,
}

impl LayerId {
    pub fn style_id(self) -> StyleId {
        use self::LayerId::*;

        match self {
            El(_) | ElNum | Border => StyleId::El,
            Pax(_) | PaxNum => StyleId::Pax
        }
    }

    pub fn features(self, store: &Store) -> &FeatureSet {
        use self::LayerId::*;

        match self {
            El(_) | Pax(_) => &store.railway,
            ElNum => &store.line_labels,
            PaxNum => &store.tt_labels,
            Border => &store.borders,
        }
    }

    pub fn latin_text(self) -> bool {
        match self {
            LayerId::El(id) | LayerId::Pax(id) => id.latin_text(),
            _ => false
        }
    }
}

impl TryFrom<tile::LayerId> for LayerId {
    type Error = TileIdError;

    fn try_from(id: tile::LayerId) -> Result<Self, Self::Error> {
        match id {
            tile::LayerId::Railway(id) => Ok(id),
        }
    }
}

impl FromStr for LayerId {
    type Err = TileIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "el" => Ok(LayerId::El(ScriptId::Original)),
            "el-lat" => Ok(LayerId::El(ScriptId::Latin)),
            "el-num" => Ok(LayerId::ElNum),
            "pax" => Ok(LayerId::Pax(ScriptId::Original)),
            "pax-lat" => Ok(LayerId::Pax(ScriptId::Latin)),
            "pax-num" => Ok(LayerId::PaxNum),
            "border" => Ok(LayerId::Border),
            _ => Err(TileIdError)
        }
    }
}


//------------ ScriptId ------------------------------------------------------

/// Which script should we prefer for labels?
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ScriptId {
    /// Use the script used by the locals.
    Original,

    /// Use latin or latin transliteration.
    Latin,
}

impl ScriptId {
    pub fn latin_text(self) -> bool {
        matches!(self, ScriptId::Latin)
    }
}

