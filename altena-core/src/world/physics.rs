use euclid;
pub struct PhysicalSpace;
pub type Float = f32;
pub type Point = euclid::TypedPoint2D<Float, PhysicalSpace>;
pub type Vector = euclid::TypedVector2D<Float, PhysicalSpace>;
pub type Angle = euclid::Angle<Float>;

pub fn angle_to_vector(a: Angle) -> Vector {
    let (s, c) = a.sin_cos();
    Vector::new(c, s)
}

// FIXME: sholdn't use constant
const TICK: Float = 0.001;
/// Moving Object
pub trait Move {
    fn point(&self) -> Point;
    /// V(t)
    fn velocity(&self) -> Vector;
}
/// 2nd order Runge Kutta
pub fn simulate_rk2(obj: &impl Move, force: impl Fn(Vector) -> Vector) -> (Point, Vector) {
    let cur_p = obj.point();
    let cur_v = obj.velocity();
    let half_v = cur_v + force(cur_v) * 0.5 * TICK;
    let nxt_p = cur_p + half_v * TICK;
    let nxt_v = cur_v + force(half_v) * TICK;
    (nxt_p, nxt_v)
}
/// euler
pub fn simulate_euler(obj: &impl Move, force: impl Fn(Vector) -> Vector) -> (Point, Vector) {
    let cur_p = obj.point();
    let cur_v = obj.velocity();
    (cur_p + cur_v * TICK, cur_v + force(cur_v) * TICK)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::BufWriter;
    use std::io::Write as IoWrite;
    #[test]
    fn test_rk2() {
        struct Ball {
            p: Point,
            v: Vector,
        }
        impl physics::Move for Ball {
            fn point(&self) -> Point {
                self.p
            }
            fn velocity(&self) -> Vector {
                self.v
            }
        }
        let angle = Angle::degrees(30.0);
        let v0 = 25.0;
        let mut b = Ball {
            p: Point::new(0.0, 0.0),
            v: angle_to_vector(angle) * v0,
        };
        let f = File::create("out.csv").unwrap();
        let mut buf = BufWriter::new(f);
        writeln!(buf, "x,y").unwrap();
        (0..4000).for_each(|_| {
            let nxt = physics::simulate_rk2(&b, |v| Vector::new(0.0, -9.8) - v * 5e-3 * v.length());
            b = Ball { p: nxt.0, v: nxt.1 };
            writeln!(buf, "{}, {}", b.p.x, b.p.y).unwrap();
        });
    }
}
