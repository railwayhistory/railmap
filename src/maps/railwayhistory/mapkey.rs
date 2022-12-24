use hyper::Body;
use kurbo::Point;
use crate::render::canvas::Canvas;
use crate::theme::Style as _;
use crate::tile::{Surface, TileFormat};
use super::style::Style;


pub fn map_key(style: Style, format: TileFormat) -> Body {
    let size = Point::new(4. * style.dimensions().sw, 1000.);
    let surface = Surface::new_map_key(format, size);
    {
        let canvas = Canvas::new(&surface);

        match style.detail() {
            0 => map_key_0(&style, &canvas),
            1 => map_key_0(&style, &canvas),
            2 => map_key_0(&style, &canvas),
            3 => map_key_0(&style, &canvas),
            _ => map_key_0(&style, &canvas),
        }
    }
    surface.finalize().into()
}


pub fn map_key_0(_style: &Style, _canvas: &Canvas) {
}

