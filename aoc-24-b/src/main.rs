use bnum::BInt;
use nom::{
    bytes::complete::tag,
    character::complete::{i128, newline, space0},
    combinator::{map, map_res, opt},
    multi::many0,
    sequence::{delimited, terminated, tuple},
    IResult,
};
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

type Value = BInt<4>;
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

    let mut line_through_all: Option<Line> = None;
    for i in 0_u128.. {
        let point = first.position_at_time(&i.into());
        let line = line_through(&point, &second, &third);
        if intersect(&line, &fourth) {
            line_through_all = Some(line);
            break;
        }
        if i % 10000000 == 0 {
            println!("Iterations: {}", i);
        }
    }

    println!("Line through: {:?}", line_through_all);

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
