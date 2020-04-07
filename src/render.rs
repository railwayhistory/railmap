//! Tools for rendering.
//!
use kurbo::{BezPath, PathEl, Point, Rect, Shape};
use crate::canvas::Canvas;

#[derive(Clone, Debug)]
pub struct Curve(BezPath);

impl Curve {
    pub fn bounding_box(&self) -> Rect {
        self.0.bounding_box()
    }

    pub fn apply(&self, canvas: &Canvas) {
        for el in self.0.iter() {
            match el {
                PathEl::MoveTo(pt) => {
                    let pt = canvas.transform() * pt;
                    canvas.move_to(pt.x, pt.y)
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    let p1 = canvas.transform() * p1;
                    let p2 = canvas.transform() * p2;
                    let p3 = canvas.transform() * p3;
                    canvas.curve_to(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y)
                }
                _ => unreachable!()
            }
        }
    }
}


pub struct CurveBuilder(BezPath);

impl CurveBuilder {
    pub fn new(start: Point) -> Self {
        let mut res = BezPath::new();
        res.move_to(start);
        CurveBuilder(res)
    }

    pub fn curve_to(&mut self, p1: Point, p2: Point, p3: Point) {
        self.0.curve_to(p1, p2, p3)
    }

    pub fn finish(self) -> Curve {
        Curve(self.0)
    }
}

