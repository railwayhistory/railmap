//! Path segments.
//!
//! This is a private module. Its public types are re-exported by the parent.

use kurbo::{
    CubicBez, Line, Rect, ParamCurve, ParamCurveArclen, ParamCurveExtrema,
    Point, Vec2
};
use crate::canvas::Canvas;
use crate::import::mp_path::velocity;
use super::{Distance, STORAGE_ACCURACY, CANVAS_ACCURACY};


//------------ Segment -------------------------------------------------------

/// A path segment.
///
/// A segment connects exactly two points either in a straight line or via a
/// cubic bezier curve.
#[derive(Clone, Copy, Debug)]
pub struct Segment {
    /// The start point of the segment.
    start: Point,

    /// The optional control points of the segment.
    ///
    /// If this is `None`, the segment is a straight line.
    control: Option<(Point, Point)>,

    /// The end point of the segment.
    end: Point,

    /// The optional precomputed arc length of the segment.
    arclen: Option<f64>,
}

impl Segment {
    /// Creates a new straight segment.
    pub fn line(start: Point, end: Point, arclen: Option<f64>) -> Self {
        Segment {
            start,
            control: None,
            end,
            arclen,
        }
    }

    /// Creates a new cubic bezier segment.
    pub fn curve(
        start: Point, c0: Point, c1: Point, end: Point, arclen: Option<f64>
    ) -> Self {
        Segment {
            start,
            control: Some((c0, c1)),
            end,
            arclen,
        }
    }

    /// Creates a new segment connecting two other segments.
    pub fn connect(
        before: Segment, post: f64, pre: f64, after: Segment
    ) -> Segment {
        // Shortcut: if both tensions are infinite, we can just make a
        // straight line.
        if post.is_infinite() && pre.is_infinite() {
            return Segment::line(before.end, after.start, None)
        }

        let r = before.end;
        let s = after.start;

        let d = s - r;
        let aa = d.atan2();
        let theta = before.exit_dir().atan2() - aa;
        let phi = after.entry_dir().atan2() - aa;
        let (st, ct) = (theta.sin(), theta.cos());
        let (sf, cf) = (-phi.sin(), phi.cos());
        let rr = velocity(st, ct, sf, cf, post);
        let ss = velocity(sf, cf, st, ct, pre);

        // XXX We are ignoring negative tension ("at least") here because
        //     we don’t have that in our path expressions (yet).

        let u = Point::new(
            r.x + (d.x * ct - d.y * st) * rr,
            r.y + (d.y * ct + d.x * st) * rr
        );
        let v = Point::new(
            s.x - (d.x * cf + d.y * sf) * ss,
            s.y - (d.y * cf - d.x * sf) * ss
        );

        // If both control points are at the end points: straight line.
        if r == u && v == s {
            Segment::line(r, s, None)
        }
        else {
            Segment::curve(r, u, v, s, None)
        }

    }

    /// Converts the segment into a kurbo segment.
    fn into_kurbo(self) -> Result<CubicBez, Line> {
        match self.control {
            Some((c0, c1)) => Ok(CubicBez::new(self.start, c0, c1, self.end)),
            None => Err(Line::new(self.start, self.end))
        }
    }

    /// Returns the start point of the segment.
    pub fn p0(self) -> Point {
        self.start
    }

    /// Returns the first control point of the segment.
    ///
    /// If the segment is a straight line, this is the start point.
    pub fn p1(self) -> Point {
        self.control.map(|c| c.0).unwrap_or_else(|| self.start)
    }

    /// Returns the second control point of the segment.
    ///
    /// If the segment is a straight line, this is the end point.
    pub fn p2(self) -> Point {
        self.control.map(|c| c.1).unwrap_or_else(|| self.end)
    }

    /// Returns the end point of the segment.
    pub fn p3(self) -> Point {
        self.end
    }

    /// Returns the point at the given times value.
    pub fn point(self, at: f64) -> Point {
        match self.into_kurbo() {
            Ok(seg) => seg.eval(at),
            Err(seg) => seg.eval(at)
        }
    }

    /// Returns a tangent vector at the given times value.
    ///
    /// The tangent vector will point into the direction of the path. It will
    /// _not_ have been normalized.
    pub fn dir(self, at: f64) -> Vec2 {
        match self.control {
            Some((c0, c1)) => {
                let ta = 1. - at;
                3. * ta * ta * (c0 - self.start)
                + 6. * ta * at * (c1 - c0)
                + 3. * at * at * (self.end - c1)
            }
            None => {
                self.end - self.start
            }
        }
    }

    /// Returns the direction when entering this segment.
    fn entry_dir(self) -> Vec2 {
        match self.control {
            Some((c0, c1)) => {
                if self.start == c0 {
                    if self.start == c1 {
                        self.end - self.start
                    }
                    else {
                        c1 - self.start
                    }
                }
                else {
                    c0 - self.start
                }
            }
            None => self.end - self.start
        }
    }

    /// Returns the direction when leaving the segment.
    fn exit_dir(self) -> Vec2 {
        match self.control {
            Some((c0, c1)) => {
                if self.end == c1 {
                    if self.end == c0 {
                        self.end - self.start
                    }
                    else {
                        self.end - c0
                    }
                }
                else {
                    self.end - c1
                }
            }
            None => self.end - self.start
        }
    }

    /// Returns whether the segment is straight.
    pub fn is_straight(self) -> bool {
        self.control.is_some()
    }

    /// Returns the bounding box of the segment.
    pub fn bounds(self) -> Rect {
        match self.into_kurbo() {
            Ok(seg) => seg.bounding_box(),
            Err(seg) => seg.bounding_box()
        }
    }

    /// Returns the arc length of the segment.
    pub fn arclen(self) -> f64 {
        match self.arclen {
            Some(arclen) => arclen,
            None => {
                match self.into_kurbo() {
                    Ok(seg) => seg.arclen(CANVAS_ACCURACY),
                    Err(seg) => seg.arclen(CANVAS_ACCURACY)
                }
            }
        }
    }

    /// Returns the arc length of the segment.
    pub fn arclen_storage(self) -> f64 {
        match self.arclen {
            Some(arclen) => arclen,
            None => {
                match self.into_kurbo() {
                    Ok(seg) => seg.arclen(STORAGE_ACCURACY),
                    Err(seg) => seg.arclen(STORAGE_ACCURACY)
                }
            }
        }
    }

    /// Returns the time of the point a given arc length away from the start.
    ///
    /// The result is accurate for use in canvas coordinates.
    pub fn arctime(self, arclen: f64) -> f64 {
        match self.into_kurbo() {
            Ok(seg) => seg.inv_arclen(arclen, CANVAS_ACCURACY),
            Err(seg) => seg.inv_arclen(arclen, CANVAS_ACCURACY),
        }
    }

    /// Returns the time of the point a given arc length away from the start.
    ///
    /// The result is accurate for use in storage coordinates.
    pub fn arctime_storage(self, arclen: f64) -> f64 {
        match self.into_kurbo() {
            Ok(seg) => seg.inv_arclen(arclen, STORAGE_ACCURACY),
            Err(seg) => seg.inv_arclen(arclen, STORAGE_ACCURACY),
        }
    }

    /// Reverses the segment.
    pub fn rev(self) -> Self {
        Segment {
            start: self.end,
            control: self.control.map(|(c0, c1)| (c1, c0)),
            end: self.start,
            arclen: self.arclen
        }
    }

    /// Transforms the segment for use with a canvas.
    pub fn transform(self, canvas: &Canvas) -> Self {
        Segment {
            start: canvas.transform() * self.start,
            control: self.control.map(|(c0, c1)| {
                (canvas.transform() * c0, canvas.transform() * c1)
            }),
            end: canvas.transform() * self.end,
            arclen: self.arclen.map(|arclen| {
                canvas.transform().as_tuple().1 * arclen
            }),
        }
    }

    /// Scales the segment and then offsets it if necessary.
    pub fn transf_off(
        self, canvas: &Canvas, offset: Option<Distance>
    ) -> Self {
        let res = self.transform(canvas);
        match offset {
            Some(offset) => {
                res.offset(offset.resolve(self.p0(), canvas))
            }
            None => res
        }
    }

    /// Returns the part of the segment between the two times.
    pub fn sub(self, start: f64, end: f64) -> Self {
        match self.into_kurbo() {
            Ok(bez) => {
                let bez = bez.subsegment(start..end);
                Segment {
                    start: bez.p0,
                    control: Some((bez.p1, bez.p2)),
                    end: bez.p3,
                    arclen: None
                }
            }
            Err(line) => {
                let line = line.subsegment(start..end);
                Segment {
                    start: line.p0,
                    control: None,
                    end: line.p1,
                    arclen: None
                }
            }
        }
    }

    /// Returns a path that is offset to the left by the given value.
    ///
    /// This uses the Tiller-Hanson method by just shifting the four points
    /// in the given direction and will break with tight curves. For now, we
    /// assume we don’t have those with railways and can get away with this
    /// approach.
    pub fn offset(self, offset: f64) -> Segment {
        // Let’s change the naming slighly. r and s are the end points, u and
        // v the control points, if we have them.
        //
        // To avoid weirdness, we treat control points that are at their
        // end points as non-existing. This may or may not be necessary
        // anymore.
        let (r, s) = (self.start, self.end);
        let (u, v) = match self.control {
            Some((p1, p2)) => {
                (
                    if p1 == r { None } else { Some(p1) },
                    if p2 == s { None } else { Some(p2) }
                )
            }
            None => (None, None)
        };

        // Since control points can be identical (too close to?) to their
        // nearest endpoint, we end up with four cases.
        match (u, v) {
            (Some(u), Some(v)) => {
                // Four unique points.
                // 
                // Get direction vectors out for the connecting lines:
                let wru = u - r; // direction from r to u.
                let wuv = v - u; // direction from u to v.
                let wvs = s - v; // direction from v to s.

                // The start and end points are just out along wru and wvs
                // rotated 90° and scaled accordingly.
                let rr = r + rot90(wru).normalize() * offset;
                let ss = s + rot90(wvs).normalize() * offset;

                // The control points are where the connecting lines meet
                // after they have been moved out. To construct these we
                // need a start point for the middle line which we get by
                // just moving u out along wuv:
                let uv = u + rot90(wuv).normalize() * offset;

                // Now we can interset the lines.
                let uu = line_intersect(rr, wru, uv, wuv);
                let vv = line_intersect(uv, wuv, ss, wvs);
                Segment::curve(rr, uu, vv, ss, None)
            }
            (None, Some(v)) => {
                // r and u are the same.
                //
                // We skip calculating uu and just set it rr.
                let wrv = v - r;
                let wvs = s - v;
                let rr = r + rot90(wrv).normalize() * offset;
                let ss = s + rot90(wvs).normalize() * offset;
                let vs = v + rot90(wvs).normalize() * offset;
                let vv = line_intersect(rr, wrv, vs, wvs);
                Segment::curve(rr, rr, vv, ss, None)
            }
            (Some(u), None) => {
                // v and s are the same.
                let wru = u - r;
                let wus = s - u;
                let rr = r + rot90(wru).normalize() * offset;
                let ss = s + rot90(wus).normalize() * offset;
                let us = u + rot90(wus).normalize() * offset;
                let uu = line_intersect(rr, wru, us, wus);
                Segment::curve(rr, uu, ss, ss, None)
            }
            (None, None) => {
                // Straight line.
                let d = rot90(s - r).normalize() * offset;
                Segment::line(r + d, s + d, None)
            }
        }
    }

    /// Applies the start of the segment to the canvas.
    pub fn apply_start(&self, canvas: &Canvas) {
        canvas.move_to(self.start.x, self.start.y);
    }

    /// Applies the tail of the segment to the canvas.
    pub fn apply_tail(&self, canvas: &Canvas) {
        /*
        canvas.stroke();
        canvas.save();
        canvas.set_source_rgb(1.0, 0., 0.);
        canvas.set_line_width(0.5 * canvas.canvas_bp());
        for p in &[self.p1(), self.p2()] {
            canvas.arc(
                p.x, p.y, 1. * canvas.canvas_bp(),
                0., 2. * std::f64::consts::PI
            );
            canvas.stroke();
        }
        canvas.set_source_rgb(0., 0., 1.);
        canvas.arc(
            self.p3().x, self.p3().y, 1. * canvas.canvas_bp(),
            0., 2. * std::f64::consts::PI
        );
        canvas.stroke();
        canvas.restore();
        canvas.move_to(self.0.p0.x, self.0.p0.y);
        */
        match self.control {
            Some((c0, c1)) => {
                canvas.curve_to(
                    c0.x, c0.y,
                    c1.x, c1.y,
                    self.end.x, self.end.y,
                )
            }
            None => {
                canvas.line_to(self.end.x, self.end.y)
            }
        }
    }

}


//------------ Helper Functions ----------------------------------------------

/// Rotates a vector by 90°.
fn rot90(vec: Vec2) -> Vec2 {
    Vec2::new(vec.y, -vec.x)
}

/// Determines the intersection point between two lines.
///
/// Each line is given by a point and a direction vector.
fn line_intersect(p1: Point, d1: Vec2, p2: Point, d2: Vec2) -> Point {
    let t = (-(p1.x - p2.x) * d2.y + (p1.y - p2.y) * d2.x)
          / (d1.x * d2.y - d1.y * d2.x);

    p1 + t * d1
}

