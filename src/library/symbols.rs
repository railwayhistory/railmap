//! All the available symbols.

use crate::canvas::Canvas;
use crate::features::{Color, Position};
use crate::import::units;


const SW: f64 = 2.25 * units::MM;
const SH: f64 = 2.1 * units::MM;
const SP: f64 = 0.5 * units::PT;

const BLACK: Color = Color::rgb(0., 0., 0.);
const GREY: Color = Color::rgb(0.6, 0.6, 0.6);

pub fn de_bf_ib(canvas: &Canvas, position: &Position) {
    BLACK.apply(canvas);
    de_bf(canvas, position)
}

pub fn de_bf_ab(canvas: &Canvas, position: &Position) {
    GREY.apply(canvas);
    de_bf(canvas, position)
}

fn de_bf(canvas: &Canvas, position: &Position) {
    let (point, angle) = position.resolve(canvas);
    canvas.translate(point.x, point.y);
    canvas.rotate(angle);
    
    let sw = SW * canvas.canvas_bp();
    let sh = SH * canvas.canvas_bp();

    canvas.move_to(-0.5 * sw, 0.);
    canvas.line_to(-0.5 * sw, sh);
    canvas.line_to(0.5 * sw, sh);
    canvas.line_to(0.5 * sw, 0.);
    canvas.close_path();
    canvas.fill();
    canvas.identity_matrix();
}


pub fn de_hp_ib(canvas: &Canvas, position: &Position) {
    BLACK.apply(canvas);
    de_hp(canvas, position)
}

pub fn de_hp_ab(canvas: &Canvas, position: &Position) {
    GREY.apply(canvas);
    de_hp(canvas, position)
}

fn de_hp(canvas: &Canvas, position: &Position) {
    let (point, angle) = position.resolve(canvas);
    canvas.translate(point.x, point.y);
    canvas.rotate(angle);
    
    let sw = SW * canvas.canvas_bp();
    let sh = SH * canvas.canvas_bp();
    let sp = SP * canvas.canvas_bp();

    canvas.move_to(-0.5 * sw + 0.5 * sp, 0.);
    canvas.line_to(-0.5 * sw + 0.5 * sp, sh - 0.5 * sp);
    canvas.line_to(0.5 * sw - 0.5 * sp, sh - 0.5 * sp);
    canvas.line_to(0.5 * sw - 0.5 * sp, 0.);
    canvas.set_line_width(sp);
    canvas.stroke();
    canvas.identity_matrix();
}

