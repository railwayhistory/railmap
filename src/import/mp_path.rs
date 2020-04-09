use std::f64::consts::SQRT_2;
use kurbo::{BezPath, Vec2};



pub struct Knot {
    point: Vec2,
    left_tension: f64,
    right_tension: f64,
}

impl Knot {
    pub fn new(point: Vec2, left_tension: f64, right_tension: f64) -> Self {
        Knot {
            point,
            left_tension, right_tension,
        }
    }
}


pub struct Segment {
    knots: Vec<Knot>,
}

impl Segment {
    pub fn from_vec(knots: Vec<Knot>) -> Self {
        Segment { knots }
    }

    pub fn to_bez_path(&self) -> BezPath {
        if self.knots.len() < 3 {
            let mut res = BezPath::new();
            res.move_to(self.knots[0].point.to_point());
            if self.knots.len() == 2 {
                res.curve_to(
                    self.knots[0].point.to_point(),
                    self.knots[1].point.to_point(),
                    self.knots[1].point.to_point(),
                )
            }
            return res
        }

        // This is a special case of the generic Hobby’s Spline algorithm:
        // We don’t do cycles and we have exactly one segment with all open
        // knots. The start and end points are of both of type curl with a
        // curl of 1.

        // Calculate the deltas from a knot to the next.
        let delta: Vec<_> = self.knots.windows(2).map(Delta::calc).collect();

        // Calculate the angle between the incoming and outgoing delta.
        // There is none for the first point, so we start with a dummy.
        // Similarly, we end with a dummy, too.
        let mut psi = vec![0.];
        psi.extend(delta.windows(2).map(|slice| {
            let sine = slice[0].y() / slice[0].len();
            let cosine = slice[0].x() / slice[0].len();
            Vec2::new(
                slice[1].x() * cosine + slice[1].y() * sine,
                slice[1].y() * cosine - slice[1].x() * sine
            ).atan2()
        }));
        psi.push(0.);

        // State for solving equations.
        let len = delta.len() + 1;
        let mut theta = vec![0.; len];
        let mut uu = vec![0.; len];
        let mut vv = vec![0.; len];
        let mut ww = vec![0.; len];

        // Start values for equations, assuming that the right knot type for
        // the first knot is curl with 1. and the left knot type for the
        // second knot is open.
        let cc = 1.; // right curl of first knot.
        let lt = self.knots[1].left_tension;
        let rt = self.knots[0].right_tension;
        uu[0] = curl_ratio(cc, rt, lt);
        vv[0] = -psi[1] * uu[0];
        ww[0] = 0.;

        // Middle values. These are all open on their left.
        for k in 1..self.knots.len() - 1 {
            let r = &self.knots[k - 1];
            let s = &self.knots[k];
            let t = &self.knots[k + 1];

            let aa = 1. / (3. * r.right_tension.abs() - 1.);
            let bb = 1. / (3. * t.left_tension.abs() - 1.);
            let cc = 1. - uu[k - 1] * aa;
            let dd = delta[k].len() * (3. - 1. / r.right_tension.abs());
            let mut ee = delta[k - 1].len() * (3. - 1. / t.left_tension.abs());

            let mut dd = dd * cc;
            let lt = s.left_tension.abs();
            let rt = s.right_tension.abs();

            if lt < rt {
                dd *= (lt/rt) * (lt/rt);
            }
            else if lt > rt {
                ee *= (rt/lt) * (rt/lt);
            }
            let ff = ee / (ee + dd);
            uu[k] = ff * bb;

            let acc = -psi[k + 1] * uu[k];
            if k == 1 {
                // In which case r’s right type is curl.
                ww[k] = 0.;
                vv[k] = acc - psi[1] * (1. - ff);
            }
            else {
                let ff = (1. - ff) / cc;
                let acc = acc - psi[k] * ff;
                let ff = ff * aa;
                vv[k] = acc - vv[k - 1] * ff;
                ww[k] = -ww[k - 1] * ff;
            }
        }

        // Last value: left type is curl with 1.
        let n = self.knots.len() - 1;
        let r = &self.knots[n - 2];
        let s = &self.knots[n - 1];
        let cc = 1.; // s.left_curl
        let lt = s.left_tension.abs();
        let rt = r.right_tension.abs();
        let ff = curl_ratio(cc, lt, rt);
        theta[n] = -(vv[n - 1] * ff) / (1. - ff * uu[n - 1]);

        for k in (0..n).rev() {
            theta[k] = vv[k] - theta[k+1] * uu[k];
        }

        let mut res = BezPath::new();
        res.move_to(self.knots[0].point.to_point());
        for k in 0..n {
            let s = &self.knots[k];
            let t = &self.knots[k + 1];
            let (ct, st) = cos_sin(theta[k]);
            let (cf, sf) = cos_sin(-psi[k + 1] - theta[k + 1]);
            let (left, right) = Self::get_controls(
                s, t, &delta[k], st, ct, sf, cf
            );
            res.curve_to(left.to_point(), right.to_point(), t.point.to_point());
        }
        res
    }

    fn get_controls(
        p: &Knot,
        q: &Knot, delta: &Delta,
        st: f64, ct: f64, sf: f64, cf: f64
    ) -> (Vec2, Vec2) {
        let lt = q.left_tension.abs();
        let rt = p.right_tension.abs();
        let mut rr = velocity(st, ct, sf, cf, rt);
        let mut ss = velocity(sf, cf, st, ct, lt);

        if p.right_tension < 0. || q.left_tension < 0. {
            if (st >= 0. && sf >= 0.) || (st <= 0. && st <= 0.) {
                let sine = st.abs() * cf + sf.abs() * ct;
                if sine > 0. {
                    let sine = sine * 2.;
                    if p.right_tension < 0. {
                        if sf.abs() < rr * sine {
                            rr = sf.abs() * sine
                        }
                    }
                    if q.left_tension < 0. {
                        if st.abs() < ss * sine {
                            ss = st.abs() * sine
                        }
                    }
                }
            }
        }
        (
            Vec2::new(
                p.point.x + (delta.x() * ct - delta.y() * st) * rr,
                p.point.y + (delta.y() * ct + delta.x() * st) * rr
            ),
            Vec2::new(
                q.point.x - (delta.x() * cf + delta.y() * sf) * ss,
                q.point.y - (delta.y() * cf - delta.x() * sf) * ss
            )
        )
    }
}


#[derive(Default)]
pub struct Delta {
    vec: Vec2,
    len: f64
}

impl Delta {
    fn calc(slice: &[Knot]) -> Self {
        let vec = slice[1].point - slice[0].point;
        Delta {
            len: vec.hypot(),
            vec
        }
    }

    fn x(&self) -> f64 {
        self.vec.x
    }

    fn y(&self) -> f64 {
        self.vec.y
    }

    fn len(&self) -> f64 {
        self.len
    }
}

fn curl_ratio(gamma: f64, a_tension: f64, b_tension: f64) -> f64 {
    let alpha = 1./a_tension;
    let beta = 1./b_tension;

    let res = ((3. - alpha) * alpha * alpha * gamma + beta * beta * beta)
            / (alpha * alpha * alpha * gamma + (3.0 - beta) * beta * beta);
    if res > 4. || res.is_nan() {
        4.
    }
    else {
        res
    }
}

fn cos_sin(theta: f64) -> (f64, f64) {
    (theta.cos(), theta.sin())
}

fn velocity(st: f64, ct: f64, sf: f64, cf: f64, t: f64) -> f64 {
    let sqrt5 = 5.0f64.sqrt();
    let res = (
        2.0 + SQRT_2 * (st - sf / 16.0) * (sf - st / 16.0) * (ct - cf)
    ) / (
        1.5 * t * ( 2. + (sqrt5 - 1.) * ct + (3. - sqrt5) * cf)
    );
    if res > 4. {
        4.
    }
    else {
        res
    }
}

