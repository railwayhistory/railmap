//! Classes of features.
//!
//! In order to be able to draw features differently for different styles, we
//! need to describe them in an abstract fashion. This is what classes do.
//!
//! There is one big [`Class`] type that aggregates various sub-classes the
//! describing various more specific things. In the map source, the class of
//! a feature is typically given through symbol set.

use crate::import::eval;
use crate::import::Failed;
use crate::import::eval::{Expression, SymbolSet};
use super::theme::Overnight;


//------------ Class ---------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Class {
    mode: Option<Mode>,
    color: Option<RouteColor>,
}

impl Class {
    pub fn from_arg(
        arg: Expression<Overnight>,
        err: &mut eval::Error,
    ) -> Result<Self, Failed> {
        let mut symbols = arg.into_symbol_set(err)?;
        let class = Self::from_symbols(&mut symbols);
        symbols.check_exhausted(err)?;
        Ok(class)
    }

    pub fn from_symbols(symbols: &mut SymbolSet) -> Self {
        Class {
            mode: Mode::from_symbols(symbols),
            color: RouteColor::from_symbols(symbols),
        }
    }

    pub fn update(&self, class: &Class) -> Class {
        Class {
            mode: {
                if let Some(mode) = class.mode {
                    Some(mode)
                }
                else {
                    self.mode
                }
            },
            color: {
                if let Some(color) = class.color {
                    Some(color)
                }
                else {
                    self.color
                }
            },
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode.unwrap_or_default()
    }

    pub fn color(&self) -> RouteColor {
        self.color.unwrap_or_default()
    }
}


//------------ Mode ----------------------------------------------------------

/// The mode of service.
#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Train,
    Ferry,
}

impl Mode {
    fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
        if symbols.take("train") {
            Some(Mode::Train)
        }
        else if symbols.take("ferry") {
            Some(Mode::Ferry)
        }
        else {
            None
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Train
    }
}


//------------ RouteColor ----------------------------------------------------

/// The color used for the service.
#[derive(Clone, Copy, Debug)]
pub enum RouteColor {
    Azure,
    Black,
    Blue,
    Brown,
    Cyan,
    Green,
    Orange,
    Pine,
    Pink,
    Purple,
    Red,
    Scarlet,
    Yellow,
}

impl RouteColor {
    fn from_symbols(symbols: &mut SymbolSet) -> Option<Self> {
        if symbols.take("azure") {
            Some(RouteColor::Azure)
        }
        else if symbols.take("black") {
            Some(RouteColor::Black)
        }
        else if symbols.take("blue") {
            Some(RouteColor::Blue)
        }
        else if symbols.take("brown") {
            Some(RouteColor::Brown)
        }
        else if symbols.take("cyan") {
            Some(RouteColor::Cyan)
        }
        else if symbols.take("green") {
            Some(RouteColor::Green)
        }
        else if symbols.take("orange") {
            Some(RouteColor::Orange)
        }
        else if symbols.take("pine") {
            Some(RouteColor::Pine)
        }
        else if symbols.take("pink") {
            Some(RouteColor::Pink)
        }
        else if symbols.take("purple") {
            Some(RouteColor::Purple)
        }
        else if symbols.take("red") {
            Some(RouteColor::Red)
        }
        else if symbols.take("scarlet") {
            Some(RouteColor::Scarlet)
        }
        else if symbols.take("yellow") {
            Some(RouteColor::Yellow)
        }
        else {
            None
        }
    }
}

impl Default for RouteColor {
    fn default() -> Self {
        RouteColor::Black
    }
}

