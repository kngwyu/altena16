use euclid;
pub struct SimulatorSpace;
pub type Float = f64;
pub type Point = euclid::TypedPoint2D<Float, SimulatorSpace>;
pub type Vector = euclid::TypedVector2D<Float, SimulatorSpace>;
pub type Angle = euclid::Angle<Float>;

pub fn angle_to_vector(a: Angle) -> Vector {
    let (s, c) = a.sin_cos();
    Vector::new(c, s)
}

/// Moving Object
pub trait Move {
    fn point(&self) -> Point;
    /// V(t)
    fn velocity(&self) -> Vector;
}

/// 2nd order Runge Kutta
pub fn simulate_rk2(
    obj: &impl Move,
    dt: Float,
    force: impl Fn(Vector) -> Vector,
) -> (Point, Vector) {
    let cur_p = obj.point();
    let cur_v = obj.velocity();
    let half_v = cur_v + force(cur_v) * 0.5 * dt;
    let nxt_p = cur_p + half_v * dt;
    let nxt_v = cur_v + force(half_v) * dt;
    (nxt_p, nxt_v)
}
/// euler
pub fn simulate_euler(
    obj: &impl Move,
    dt: Float,
    force: impl Fn(Vector) -> Vector,
) -> (Point, Vector) {
    let cur_p = obj.point();
    let cur_v = obj.velocity();
    (cur_p + cur_v * dt, cur_v + force(cur_v) * dt)
}

#[cfg(test)]
mod simulator_test {
    use super::*;
    use std::fs::File;
    use std::io::BufWriter;
    use std::io::Write as IoWrite;
    #[test]
    fn test_rk2() {
        #[derive(Clone)]
        struct Ball {
            p: Point,
            v: Vector,
        }
        impl Move for Ball {
            fn point(&self) -> Point {
                self.p
            }
            fn velocity(&self) -> Vector {
                self.v
            }
        }
        let angle = Angle::degrees(30.0);
        let v0 = 25.0;
        let mut b1 = Ball {
            p: Point::new(0.0, 0.0),
            v: angle_to_vector(angle) * v0,
        };
        let mut b2 = b1.clone();
        let f = File::create("out.csv").unwrap();
        let mut buf = BufWriter::new(f);
        writeln!(buf, "x,y").unwrap();
        let dt = 0.001;
        let _ = (0..4000).fold(0.0, |diff_before, i| {
            let nxt = simulate_rk2(&b1, dt, |v| Vector::new(0.0, -9.8) - v * 5e-3 * v.length());
            b1 = Ball { p: nxt.0, v: nxt.1 };
            writeln!(buf, "{}, {}", b1.p.x, b1.p.y).unwrap();
            let nxt = simulate_euler(&b2, dt, |v| Vector::new(0.0, -9.8) - v * 5e-3 * v.length());
            b2 = Ball { p: nxt.0, v: nxt.1 };
            let diff = (b2.p - b1.p).length();
            assert!(diff < f64::from(i + 1) * 0.00001);
            assert!(diff > diff_before);
            diff
        });
    }
}
