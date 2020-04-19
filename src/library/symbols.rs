//! All the available symbols.

use std::f64::consts::PI;
use crate::canvas::Canvas;
use crate::import::units;


const SYMBOLS: &[(&'static str, &'static dyn Fn(&Canvas))] = &[
    ("de_bf", &|canvas| {
        let sw = SW * canvas.canvas_bp();
        let sh = SH * canvas.canvas_bp();
        let ds = DS * canvas.canvas_bp();

        canvas.move_to(-0.5 * sw, 0.);
        canvas.line_to(-0.5 * sw, sh - ds);
        canvas.curve_to(
            -0.5 * sw, sh - 1.5 * ds,
            -0.5 * sw + 0.5 * ds, sh,
            -0.5 * sw + ds, sh
        );
        canvas.line_to(0.5 * sw - ds, sh);
        canvas.curve_to(
            0.5 * sw - 0.5 * ds, sh,
            0.5 * sw, sh - 0.5 * ds,
            0.5 * sw, sh - ds
        );
        canvas.line_to(0.5 * sw, 0.);
        canvas.close_path();
        canvas.fill();
    }),

    ("de_hp", &|canvas| {
        let sw = SW * canvas.canvas_bp();
        let sh = SH * canvas.canvas_bp();
        let sp = SP * canvas.canvas_bp();

        canvas.move_to(-0.5 * sw + 0.5 * sp, 0.);
        canvas.line_to(-0.5 * sw + 0.5 * sp, sh - 0.5 * sp);
        canvas.line_to(0.5 * sw - 0.5 * sp, sh - 0.5 * sp);
        canvas.line_to(0.5 * sw - 0.5 * sp, 0.);
        canvas.set_line_width(sp);
        canvas.stroke();
    }),

    ("de_dirgr", &|canvas| {
        let sp = SP * canvas.canvas_bp();
        let dt = DT * canvas.canvas_bp();

        let u = 0.8;
        canvas.set_line_width(sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., 2. * u * dt);
        canvas.stroke();
        canvas.arc(0., 3. * u * dt, u * dt, 0., 2. * PI);
        canvas.stroke();
        canvas.arc(0., 3. * u * dt, 0.5 * u * dt, 0., 2. * PI);
        canvas.fill();
    }),

    ("statdt", &|canvas| {
        let sp = SP * canvas.canvas_bp();
        let dt = DT * canvas.canvas_bp();

        canvas.set_line_width(sp);
        canvas.move_to(0., 0.);
        canvas.line_to(0., dt);
        canvas.stroke();
    }),
];


#[derive(Clone, Copy, Debug)]
pub struct Symbol(usize);

impl Symbol {
    pub fn lookup(name: &str) -> Option<Self> {
        SYMBOLS.iter().enumerate().find_map(|(i, item)| {
            if item.0 == name {
                Some(Symbol(i))
            }
            else {
                None
            }
        })
    }

    pub fn render(self, canvas: &Canvas) {
        (*SYMBOLS[self.0].1)(canvas)
    }
}


const DT: f64 = units::DT;
const SW: f64 = 2.25 * units::MM;
const SH: f64 = 2.1 * units::MM;
const SP: f64 = 0.5 * units::PT;
const DS: f64 = 0.2 * units::PT;
