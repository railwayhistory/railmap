//! Classes of features.
//!
//! These help determine how to draw something. They apply to all types of
//! features.

use crate::features::color::Color;
use crate::import::eval::SymbolSet;

//------------ Class ---------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Class {
    status: Status,
    electrification: Electrification,
    pax: bool,
    tram: bool,
}

impl Class {
    pub fn from_symbols(symbols: &SymbolSet) -> Self {
        Class {
            status: Status::from_symbols(symbols),
            electrification: Electrification::from_symbols(symbols),
            pax: symbols.contains("pax"),
            tram: symbols.contains("tram"),
        }
    }
}

#[cfg(not(feature = "proof"))]
impl Class {
    pub fn standard_color(self) -> Color {
        self.status.standard_color()
    }

    pub fn shade_color(self) -> Color {
        if self.tram {
            self.status.tram_color()
        }
        else if self.pax {
            self.electrification.pax_shade_color()
        }
        else {
            self.electrification.non_pax_shade_color()
        }
    }

    pub fn removed_color(self) -> Color {
        if self.tram {
            Status::Removed.tram_color()
        }
        else {
            Status::Removed.important_color().unwrap()
        }
    }

    pub fn label_color(self) -> Color {
        self.standard_color()
    }

    pub fn marker_color(self) -> Color {
        self.standard_color()
    }
}

#[cfg(feature = "proof")]
impl Class {
    pub fn standard_color(self) -> Color {
        if self.tram {
            self.status.tram_color()
        }
        else if let Some(color) = self.status.important_color() {
            color
        }
        else if self.pax {
            self.electrification.pax_color()
        }
        else {
            self.electrification.non_pax_color()
        }
    }

    pub fn shade_color(self) -> Color {
        Color::TRANSPARENT
    }

    pub fn removed_color(self) -> Color {
        if self.tram {
            Status::Removed.tram_color()
        }
        else {
            Status::Removed.important_color().unwrap()
        }
    }

    pub fn label_color(self) -> Color {
        self.standard_color()
    }

    pub fn marker_color(self) -> Color {
        self.standard_color()
    }
}

//------------ OptClass ------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct OptClass {
    status: Option<Status>,
    electrification: Option<Electrification>,
    pax: bool,
    tram: bool,
}

impl OptClass {
    pub fn from_symbols(symbols: &SymbolSet) -> Self {
        if symbols.contains("tram") {
            OptClass {
                status: Some(Status::from_symbols(symbols)),
                electrification: None,
                pax: false,
                tram: true,
            }
        }
        else if symbols.contains("pax") {
            OptClass {
                status: Some(Status::from_symbols(symbols)),
                electrification: Some(Electrification::from_symbols(symbols)),
                pax: true,
                tram: false,
            }
        }
        else {
            OptClass {
                status: Status::opt_from_symbols(symbols),
                electrification: Electrification::opt_from_symbols(symbols),
                pax: false,
                tram: false,
            }
        }
    }
}

#[cfg(not(feature = "proof"))]
impl OptClass {
    pub fn label_color(self) -> Option<Color> {
        if let Some(status) = self.status {
            Some(status.standard_color())
        }
        else {
            None
        }
    }
}

#[cfg(feature = "proof")]
impl OptClass {
    pub fn label_color(self) -> Option<Color> {
        if let Some(status) = self.status {
            if self.tram {
                Some(status.tram_color())
            }
            else if matches!(status, Status::Open) {
                if let Some(el) = self.electrification {
                    if self.pax {
                        Some(el.pax_color())
                    }
                    else {
                        Some(el.non_pax_color())
                    }
                }
                else {
                    Some(status.standard_color())
                }
            }
            else {
                Some(status.standard_color())
            }
        }
        else if let Some(el) = self.electrification {
            if self.pax {
                Some(el.pax_color())
            }
            else {
                Some(el.non_pax_color())
            }
        }
        else {
            None
        }
    }
}


//------------ Status --------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum Status {
    Open,
    Closed,
    Removed,
    Gone
}

impl Status {
    fn from_symbols(symbols: &SymbolSet) -> Self {
        if symbols.contains("closed") {
            Status::Closed
        }
        else if symbols.contains("removed") {
            Status::Removed
        }
        else if symbols.contains("gone") {
            Status::Gone
        }
        else {
            Status::Open
        }
    }

    fn opt_from_symbols(symbols: &SymbolSet) -> Option<Self> {
        if symbols.contains("closed") {
            Some(Status::Closed)
        }
        else if symbols.contains("removed") {
            Some(Status::Removed)
        }
        else if symbols.contains("gone") {
            Some(Status::Gone)
        }
        else if symbols.contains("open") {
            Some(Status::Open)
        }
        else {
            None
        }
    }

    fn standard_color(self) -> Color {
        match self {
            Status::Open => BLACK,
            Status::Closed => DARK_GREY,
            Status::Removed => MEDIUM_GREY,
            Status::Gone => LIGHT_GREY
        }
    }

    fn important_color(self) -> Option<Color> {
        match self {
            Status::Open => None,
            Status::Closed => Some(DARK_GREY),
            Status::Removed => Some(MEDIUM_GREY),
            Status::Gone => Some(LIGHT_GREY)
        }
    }

    fn tram_color(self) -> Color {
        match self {
            Status::Open => BLUE_OPEN,
            Status::Closed => BLUE_CLOSED,
            Status::Removed => BLUE_REMOVED,
            Status::Gone => BLUE_GONE,
        }
    }
}


//------------ Electrification -----------------------------------------------

#[derive(Clone, Copy, Debug)]
enum Electrification {
    None,
    OleAcHigh,
    OleAcLow,
    OleDcHigh,
    OleDcLow,
    OleUnknown,
    RailHigh,
    RailLow,
    RailUnknown,
}

impl Electrification {
    fn from_symbols(symbols: &SymbolSet) -> Self {
        if symbols.contains("cat") {
            if symbols.contains("ac6k6") { // 6600 V AC 25 Hz
                Electrification::OleAcLow
            }
            else if symbols.contains("ac15") {
                Electrification::OleAcLow
            }
            else if symbols.contains("ac25") {
                Electrification::OleAcHigh
            }
            else if symbols.contains("dc30") {
                Electrification::OleDcHigh
            }
            else if symbols.contains("dc15") {
                Electrification::OleDcLow
            }
            else if symbols.contains("dc75") {
                Electrification::OleDcLow
            }
            else {
                Electrification::OleUnknown
            }
        }
        else if symbols.contains("rail") {
            if symbols.contains("dc12") {
                Electrification::RailHigh
            }
            else if symbols.contains("dc75") {
                Electrification::RailLow
            }
            else {
                Electrification::RailUnknown
            }
        }
        else {
            Electrification::None
        }
    }

    fn opt_from_symbols(symbols: &SymbolSet) -> Option<Self> {
        match Self::from_symbols(symbols) {
            Electrification::None => None,
            other => Some(other)
        }
    }

    fn pax_color(self) -> Color {
        match self {
            Electrification::None => YELLOW_HIGH,
            Electrification::OleAcHigh => PURPLE_HIGH,
            Electrification::OleAcLow => PINK_HIGH,
            Electrification::OleDcHigh => RED_HIGH,
            Electrification::OleDcLow => ORANGE_HIGH,
            Electrification::RailHigh => CYAN_HIGH,
            Electrification::RailLow => GREEN_HIGH,
            _ => TOXIC_HIGH, 
        }
    }

    fn non_pax_color(self) -> Color {
        match self {
            Electrification::None => BLACK,
            Electrification::OleAcHigh => PURPLE_LOW,
            Electrification::OleAcLow => PINK_LOW,
            Electrification::OleDcHigh => RED_LOW,
            Electrification::OleDcLow => ORANGE_LOW,
            Electrification::RailHigh => CYAN_LOW,
            Electrification::RailLow => GREEN_LOW,
            _ => TOXIC_LOW,
        }
    }
}

#[cfg(not(feature = "proof"))]
impl Electrification {
    fn pax_shade_color(self) -> Color {
        match self {
            Electrification::None => YELLOW_HIGH,
            Electrification::OleAcHigh => PURPLE_HIGH,
            Electrification::OleAcLow => PINK_HIGH,
            Electrification::OleDcHigh => RED_HIGH,
            Electrification::OleDcLow => ORANGE_HIGH,
            Electrification::RailHigh => CYAN_HIGH,
            Electrification::RailLow => GREEN_HIGH,
            _ => TOXIC_HIGH, 
        }
    }

    fn non_pax_shade_color(self) -> Color {
        match self {
            Electrification::None => Color::rgba(0., 0., 0., 0.),
            Electrification::OleAcHigh => PURPLE_LOW,
            Electrification::OleAcLow => PINK_LOW,
            Electrification::OleDcHigh => RED_LOW,
            Electrification::OleDcLow => ORANGE_LOW,
            Electrification::RailHigh => CYAN_LOW,
            Electrification::RailLow => GREEN_LOW,
            _ => TOXIC_LOW,
        }
    }

}


//------------ The Color Palette ---------------------------------------------
//
// Currently, we use eight distinct colors, each in a ‘high’ and ‘low’
// variant. Plus three greys.

const RED_HIGH: Color = Color::rgb(0.926, 0.156, 0.156);
const RED_LOW:  Color = Color::rgb(0.645, 0.055, 0.055);
const ORANGE_HIGH: Color = Color::rgb(0.926, 0.668, 0.156);
const ORANGE_LOW: Color = Color::rgb(0.644, 0.445, 0.055);
const YELLOW_HIGH: Color = Color::rgb(0.647, 0.447, 0.055);
//const YELLOW_LOW: Color = Color::rgb(0.424, 0.294, 0.035);
//const GREEN_HIGH: Color = Color::rgb(0.063, 0.737, 0.063);
const GREEN_HIGH: Color = Color::rgb(0.055, 0.645, 0.055);
const GREEN_LOW: Color = Color::rgb(0.035, 0.424, 0.035);
//const CYAN_HIGH: Color = Color::rgb(0.071, 0.835, 0.706);
const CYAN_HIGH: Color = Color::rgb(0.055, 0.644, 0.547);
const CYAN_LOW: Color = Color::rgb(0.035, 0.424, 0.361);
//const BLUE_HIGH: Color = Color::rgb(0.156, 0.156, 0.926);
//const BLUE_LOW: Color = Color::rgb(0.055, 0.055, 0.645);
const PURPLE_HIGH: Color = Color::rgb(0.668, 0.156, 0.926);
const PURPLE_LOW: Color = Color::rgb(0.445, 0.055, 0.645);
const PINK_HIGH: Color = Color::rgb(0.926, 0.156, 0.668);
const PINK_LOW: Color = Color::rgb(0.645, 0.055, 0.445);

const TOXIC_HIGH: Color = Color::rgb(0.824, 0.824, 0.0);
const TOXIC_LOW: Color = Color::rgb(0.824, 0.824, 0.0);

//const WHITE: Color = Color::grey(1.);
const BLACK: Color = Color::grey(0.109);

const DARK_GREY:   Color = Color::grey(0.450);
const MEDIUM_GREY: Color = Color::grey(0.550);
const LIGHT_GREY:  Color = Color::grey(0.750);

const BLUE_OPEN: Color = Color::rgb(0.109, 0.387, 0.668); // #1C63AB
const BLUE_CLOSED: Color = Color::rgb(0.367, 0.555, 0.723); // # 5E8EB9
const BLUE_REMOVED: Color = Color::rgb(0.559, 0.686, 0.816); // #8FB0D1
const BLUE_GONE: Color = Color::rgb(0.742, 0.820, 0.890); // #BED2E4
