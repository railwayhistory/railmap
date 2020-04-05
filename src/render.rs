/// Tools for rendering.
use kurbo::Vec2;
use crate::path::Path;
use crate::tile::TileId;


pub struct Curve<'a> {
    path: &'a Path,
    tile: &'a TileId,
}

impl<'a> Curve<'a> {
    pub fn new(path: &'a Path, tile: &'a TileId) -> Self {
        Curve {
            path, tile
        }
    }

    /// Applies the curve to the given surface.
    ///
    /// This updates the current path of the surface to the path.
    pub fn apply(&self, context: &cairo::Context) {
        match self.path.len() {
            0 => return,
            1 => {
                move_to(context, self.z(0));
                return;
            }
            _ => { }
        }
        
        for i in 0..self.path.len() - 2 {
            let z0 = self.z(i);
            let z1 = self.z(i + 1);
            let w0 = self.w(i);
            let w1 = self.w(i + 1);
            let t0 = self.t_out(i);
            let t1 = self.t_in(i + 1);

            let theta = w0.atan2() - (z1 - z0).atan2();
            let phi = (z1 - z0).atan2() - w1.atan2();

            let (rho, sigma) = velocity_params(theta, phi);

            curve_to(
                context,
                z0 + rho / (3. * t0) * (z1 - z0).hypot() * w0,
                z1 - sigma / (3. * t1) * (z1 - z0).hypot() * w1,
                z1
            )
        }
    }
}

impl<'a> Curve<'a> {
    fn z(&self, i: usize) -> Vec2 {
        self.tile.proj(self.path.node(i).lonlat())
    }

    /// Returns the slope at point `i`.
    ///
    /// This is a unit vector describing the direction of the path through
    /// the point at index `i`.
    fn w(&self, i: usize) -> Vec2 {
        if i == 0 {
            (self.z(1) - self.z(0)).normalize()
        }
        else if i == self.path.len() - 1 {
            (self.z(i) - self.z(i - 1)).normalize()
        }
        else {
            (-self.z(i - 1) + self.z(i + 1)).normalize()
        }
    }

    fn t_in(&self, i: usize) -> f64 {
        self.path.node(i).pre
    }

    fn t_out(&self, i: usize) -> f64 {
        self.path.node(i).post
    }
}

fn velocity_params(theta: f64, phi: f64) -> (f64, f64) {
    let a = 2.0f64.sqrt(); //1.597;
    let b = 1./16.; //0.07;
    let c = (3. - 5.0f64.sqrt()) / 2.; //0.37;

    let st = theta.sin();
    let ct = theta.cos();
    let sp = phi.sin();
    let cp = phi.cos();

    let alpha = a * (st - b * sp) * (sp - b * st) * (ct - cp);
    let rho = (2. + alpha) / (1. + (1. - c) * ct + c * cp);
    let sigma = (2. - alpha) / (1. + (1. - c) * cp + c * ct);
    (rho, sigma)
}

fn move_to(context: &cairo::Context, z: Vec2) {
    context.move_to(z.x, z.y)
}

fn curve_to(context: &cairo::Context, c0: Vec2, c1: Vec2, z: Vec2) {
    context.curve_to(c0.x, c0.y, c1.x, c1.y, z.x, z.y)
}

