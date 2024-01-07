use bnum::BInt;
use nom::{
    bytes::complete::tag,
    character::complete::{i128, newline, space0},
    combinator::{map, map_res, opt},
    multi::many0,
    sequence::{delimited, terminated, tuple},
    IResult,
};
use num_rational::Ratio;
use std::fs;
use std::ops::Add;

fn main() {
    println!("Let's solve AOC-24!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> IResult<&str, Vec<Line>> {
    many0(terminated(hailstone, newline))(input)
}

fn hailstone(input: &str) -> IResult<&str, Line> {
    map(
        tuple((
            padded_value,
            padded_value,
            padded_value,
            tag(" @"),
            padded_value,
            padded_value,
            padded_value,
        )),
        |(px, py, pz, _, vx, vy, vz)| Line {
            px,
            py,
            pz,
            vx,
            vy,
            vz,
        },
    )(input)
}

fn padded_value(input: &str) -> IResult<&str, Value> {
    delimited(space0, value, opt(tag(",")))(input)
}

fn value(input: &str) -> IResult<&str, Value> {
    map_res(i128, |v| v.try_into())(input)
}

/* --------------- */
/* Data Structures */
/* --------------- */

#[derive(Debug, Clone, Copy)]
struct Line {
    px: Value,
    py: Value,
    pz: Value,
    vx: Value,
    vy: Value,
    vz: Value,
}
impl Line {
    fn position(&self) -> Coord {
        (self.px, self.py, self.pz)
    }
    fn position_at_time(&self, time: &Value) -> Coord {
        (
            self.px + time * self.vx,
            self.py + time * self.vy,
            self.pz + time * self.vz,
        )
    }
    fn velocity(&self) -> Vector {
        Vector(self.vx, self.vy, self.vz)
    }
    fn from_coord_and_vector(coord: &Coord, vector: &Vector) -> Self {
        Line {
            px: coord.0,
            py: coord.1,
            pz: coord.2,
            vx: vector.0,
            vy: vector.1,
            vz: vector.2,
        }
    }
}
impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}, {}) @ ({}, {}, {})",
            self.px, self.py, self.pz, self.vx, self.vy, self.vz
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Vector(Value, Value, Value);
impl Add for Vector {
    type Output = Vector;
    fn add(self, rhs: Self) -> Self::Output {
        Vector(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

#[derive(Debug, Clone, Copy)]
struct Quadric {
    xx: Value,
    xy: Value,
    xz: Value,
    yy: Value,
    yz: Value,
    zz: Value,
    x: Value,
    y: Value,
    z: Value,
    c: Value,
}
impl Quadric {
    fn from_lines(first: &Line, second: &Line, third: &Line) -> Self {
        let PlückerCoord {
            displacement: c1,
            moment: d1,
        } = PlückerCoord::of_line(first);
        let PlückerCoord {
            displacement: c2,
            moment: d2,
        } = PlückerCoord::of_line(second);
        let PlückerCoord {
            displacement: c3,
            moment: d3,
        } = PlückerCoord::of_line(third);

        let xx = d1.0 * (c2.1 * c3.2 - c2.2 * c3.1)
            + d2.0 * (c3.1 * c1.2 - c3.2 * c1.1)
            + d3.0 * (c1.1 * c2.2 - c1.2 * c2.1);
        let yy = d1.1 * (c2.2 * c3.0 - c2.0 * c3.2)
            + d2.1 * (c3.2 * c1.0 - c3.0 * c1.2)
            + d3.1 * (c1.2 * c2.0 - c1.0 * c2.2);
        let zz = d1.2 * (c2.0 * c3.1 - c2.1 * c3.0)
            + d2.2 * (c3.0 * c1.1 - c3.1 * c1.0)
            + d3.2 * (c1.0 * c2.1 - c1.1 * c2.0);
        let yz = c1.0 * ((c2.1 * d3.1 - d2.1 * c3.1) + (c3.2 * d2.2 - d3.2 * c2.2))
            + c2.0 * ((c3.1 * d1.1 - d3.1 * c1.1) + (c1.2 * d3.2 - d1.2 * c3.2))
            + c3.0 * ((c1.1 * d2.1 - d1.1 * c2.1) + (c2.2 * d1.2 - d2.2 * c1.2));
        let xz = c1.1 * ((c2.2 * d3.2 - d2.2 * c3.2) + (c3.0 * d2.0 - d3.0 * c2.0))
            + c2.1 * ((c3.2 * d1.2 - d3.2 * c1.2) + (c1.0 * d3.0 - d1.0 * c3.0))
            + c3.1 * ((c1.2 * d2.2 - d1.2 * c2.2) + (c2.0 * d1.0 - d2.0 * c1.0));
        let xy = c1.2 * ((c2.0 * d3.0 - d2.0 * c3.0) + (c3.1 * d2.1 - d3.1 * c2.1))
            + c2.2 * ((c3.0 * d1.0 - d3.0 * c1.0) + (c1.1 * d3.1 - d1.1 * c3.1))
            + c3.2 * ((c1.0 * d2.0 - d1.0 * c2.0) + (c2.1 * d1.1 - d2.1 * c1.1));
        let x = d1.0 * ((c2.1 * d3.1 - d2.1 * c3.1) - (c3.2 * d2.2 - d3.2 * c2.2))
            + d2.0 * ((c3.1 * d1.1 - d3.1 * c1.1) - (c1.2 * d3.2 - d1.2 * c3.2))
            + d3.0 * ((c1.1 * d2.1 - d1.1 * c2.1) - (c2.2 * d1.2 - d2.2 * c1.2));
        let y = d1.1 * ((c2.2 * d3.2 - d2.2 * c3.2) - (c3.0 * d2.0 - d3.0 * c2.0))
            + d2.1 * ((c3.2 * d1.2 - d3.2 * c1.2) - (c1.0 * d3.0 - d1.0 * c3.0))
            + d3.1 * ((c1.2 * d2.2 - d1.2 * c2.2) - (c2.0 * d1.0 - d2.0 * c1.0));
        let z = d1.2 * ((c2.0 * d3.0 - d2.0 * c3.0) - (c3.1 * d2.1 - d3.1 * c2.1))
            + d2.2 * ((c3.0 * d1.0 - d3.0 * c1.0) - (c1.1 * d3.1 - d1.1 * c3.1))
            + d3.2 * ((c1.0 * d2.0 - d1.0 * c2.0) - (c2.1 * d1.1 - d2.1 * c1.1));
        let c = d1.0 * (d2.1 * d3.2 - d2.2 * d3.1)
            + d1.1 * (d2.2 * d3.0 - d2.0 * d3.2)
            + d1.2 * (d2.0 * d3.1 - d2.1 * d3.0);

        Quadric {
            xx,
            yy,
            zz,
            xy,
            xz,
            yz,
            x,
            y,
            z,
            c,
        }
    }

    fn line_intersection_eq(&self, line: &Line) -> QuadraticEq {
        let (px, py, pz) = line.position();
        let Vector(vx, vy, vz) = line.velocity();

        // Coefficients for quadratic equation describing time of intersection
        let c = self.c
            + self.x * px
            + self.y * py
            + self.z * pz
            + self.xy * px * py
            + self.yz * py * pz
            + self.xz * px * pz
            + self.xx * px * px
            + self.yy * py * py
            + self.zz * pz * pz;
        let t = self.x * vx
            + self.y * vy
            + self.z * vz
            + self.xy * (py * vx + px * vy)
            + self.yz * (pz * vy + py * vz)
            + self.xz * (px * vz + pz * vx)
            + (Value::from(2)) * (self.xx * vx * px + self.yy * vy * py + self.zz * vz * pz);
        let tt = self.xy * vx * vy
            + self.yz * vy * vz
            + self.xz * vz * vx
            + self.xx * vx * vx
            + self.yy * vy * vy
            + self.zz * vz * vz;

        QuadraticEq { c, t, tt }
    }

    fn tangent_plane_at_point(&self, point: &Coord) -> Plane {
        let (px, py, pz) = point;
        let gradient_vector = Vector(
            self.x + self.xy * py + self.xz * pz + Value::from(2) * self.xx * px,
            self.y + self.xy * px + self.yz * pz + Value::from(2) * self.yy * py,
            self.z + self.xz * px + self.yz * py + Value::from(2) * self.zz * pz,
        );

        Plane::from_point_and_normal(point, &gradient_vector)
    }
}

#[derive(Debug, Clone, Copy)]
struct QuadraticEq {
    c: Value,
    t: Value,
    tt: Value,
}
impl QuadraticEq {
    fn discriminant(&self) -> Value {
        self.t * self.t - Value::from(4) * self.tt * self.c
    }
}

#[derive(Debug, Clone, Copy)]
struct Plane {
    x: Value,
    y: Value,
    z: Value,
    c: Value,
}
impl Plane {
    fn from_point_and_normal(point: &Coord, normal: &Vector) -> Self {
        let Vector(vx, vy, vz) = normal;
        let origin = (Value::from(0), Value::from(0), Value::from(0));

        Plane {
            x: *vx,
            y: *vy,
            z: *vz,
            c: -dot(normal, &difference(point, &origin)),
        }
    }
}

struct PlückerCoord {
    displacement: Vector,
    moment: Vector,
}
impl PlückerCoord {
    fn of_line(line: &Line) -> Self {
        let origin = (0.into(), 0.into(), 0.into());
        let first_pt = line.position();
        let second_pt = line.position_at_time(&1.into());

        PlückerCoord {
            displacement: difference(&second_pt, &first_pt),
            moment: cross(
                &difference(&first_pt, &origin),
                &difference(&second_pt, &origin),
            ),
        }
    }
}

// Coefficients for a quadratic constraint on the solving line.
// px,py,pz are position variables, while vx,vy,vz are directional
#[derive(Debug, Clone, Copy)]
struct Coeffs {
    vx: Value,
    vy: Value,
    vz: Value,
    pxvy: Value,
    pxvz: Value,
    pyvx: Value,
    pyvz: Value,
    pzvx: Value,
    pzvy: Value,
}
impl Coeffs {
    fn from_line(line: &Line) -> Self {
        let origin = (0.into(), 0.into(), 0.into());
        let Vector(vx, vy, vz) = cross(&difference(&line.position(), &origin), &line.velocity());
        let Vector(wx, wy, wz) = line.velocity();

        Coeffs {
            vx,
            vy,
            vz,
            pxvy: wz,
            pxvz: -wy,
            pyvx: -wz,
            pyvz: wx,
            pzvx: wy,
            pzvy: -wx,
        }
    }
}

type Value = BInt<4>;
type Rational = Ratio<BInt<4>>;
type Coord = (Value, Value, Value);

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> Value {
    let (_, hailstones) = problem_input(input).expect("Failed to parse problem input");

    let skew_lines = four_skew_lines(&hailstones).expect("Failed to find four skew lines");
    let first = skew_lines[0];
    let second = skew_lines[1];
    let third = skew_lines[2];
    let fourth = skew_lines[3];

    let quadric123 = Quadric::from_lines(&first, &second, &third);

    println!("Quadric123 :");
    println!("{quadric123:?}");

    let quadratic_eq = quadric123.line_intersection_eq(&fourth);

    println!("Line intersection parameters:");
    println!("{quadratic_eq:?}");
    println!("Discriminant:");
    println!("{:?}", quadratic_eq.discriminant());

    /* ... analytic things outside this program ... */

    let intersection_pt = fourth.position_at_time(&Value::from(1013967010243u64));
    println!("Point of intersection:");
    println!("{intersection_pt:?}");

    let plane = quadric123.tangent_plane_at_point(&intersection_pt);
    println!("Tangent plane at intersection:");
    println!("{plane:?}");

    return 0.into();
}

fn four_skew_lines(lines: &[Line]) -> Result<[Line; 4], ()> {
    let mut skew_lines = Vec::default();
    for line in lines.iter() {
        if skew_lines
            .iter()
            .all(|l| (!intersect(l, line)) && (!parallel(l, line)))
        {
            skew_lines.push(*line);
        }
        if skew_lines.len() == 4 {
            break;
        }
    }

    if let Ok(output) = skew_lines.try_into() {
        Ok(output)
    } else {
        Err(())
    }
}

// The plane defined by a point and a line,
// presented as a line normal to it (with its basepoint)
fn plane_through(point: &Coord, line: &Line) -> Line {
    let first_vector = difference(&line.position(), point);
    let second_vector = difference(&line.position_at_time(&1.into()), point);

    Line::from_coord_and_vector(point, &cross(&first_vector, &second_vector))
}

fn line_through(point: &Coord, first: &Line, second: &Line) -> Line {
    let first = plane_through(point, first);
    let second = plane_through(point, second);

    // Both of these have the given point as their position.
    // Taking the cross product of velocities gives us the direction
    // of the line of intersection.

    Line::from_coord_and_vector(point, &cross(&first.velocity(), &second.velocity()))
}

// Compute whether two non-parallel lines intersect.
// Do this by testing whether the cross product of their velocities dotted
// with the difference in their positions is zero.
fn intersect(first: &Line, second: &Line) -> bool {
    let velocities_crossed = cross(&first.velocity(), &second.velocity());
    let diff = difference(&first.position(), &second.position());
    dot(&velocities_crossed, &diff) == 0.into()
}

fn parallel(first: &Line, second: &Line) -> bool {
    cross(&first.velocity(), &second.velocity()) == Vector(0.into(), 0.into(), 0.into())
}

/* ---------------- */
/* Helper Functions */
/* ---------------- */

fn cross(first: &Vector, second: &Vector) -> Vector {
    let Vector(x1, y1, z1) = first;
    let Vector(x2, y2, z2) = second;

    Vector(y1 * z2 - z1 * y2, z1 * x2 - x1 * z2, x1 * y2 - y1 * x2)
}

fn dot(first: &Vector, second: &Vector) -> Value {
    let Vector(x1, y1, z1) = first;
    let Vector(x2, y2, z2) = second;

    x1 * x2 + y1 * y2 + z1 * z2
}

fn difference(first: &Coord, second: &Coord) -> Vector {
    let (x1, y1, z1) = first;
    let (x2, y2, z2) = second;

    Vector(x1 - x2, y1 - y2, z1 - z2)
}

#[cfg(test)]
fn test() {
    let test_line_1 = Line::from_coord_and_vector(
        &((-3).into(), (-5).into(), (-2).into()),
        &Vector(1.into(), 0.into(), 0.into()),
    );
    let test_line_2 = Line::from_coord_and_vector(
        &((-3).into(), (-5).into(), 2.into()),
        &Vector(0.into(), 1.into(), 0.into()),
    );
    let test_line_3 = Line::from_coord_and_vector(
        &(3.into(), 5.into(), 0.into()),
        &Vector(0.into(), 0.into(), 1.into()),
    );
    let test_line_4 = Line::from_coord_and_vector(
        &(3.into(), 10.into(), 0.into()),
        &Vector(1.into(), 0.into(), 1.into()),
    );

    let test_quadric_123 = Quadric::from_lines(&test_line_1, &test_line_2, &test_line_3);
    let test_quadric_124 = Quadric::from_lines(&test_line_1, &test_line_2, &test_line_4);
    println!("{test_quadric_123:?}");
    println!("{test_quadric_124:?}");
}
