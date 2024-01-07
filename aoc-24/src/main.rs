use nom::{
    bytes::complete::tag,
    character::complete::{i128, newline, space0},
    combinator::{map, map_res, opt},
    multi::many0,
    sequence::{delimited, terminated, tuple},
    IResult,
};
use num_rational::Ratio;
use num_traits::sign::Signed;
use std::{
    cmp::{max, min},
    fs,
    mem::swap,
};

fn main() {
    println!("Let's solve AOC-24!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

const TEST_MIN: i128 = 200000000000000;
const TEST_MAX: i128 = 400000000000000;

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> IResult<&str, Vec<Hailstone>> {
    many0(terminated(hailstone, newline))(input)
}

fn hailstone(input: &str) -> IResult<&str, Hailstone> {
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
        |(px, py, _, _, vx, vy, _)| Hailstone { px, py, vx, vy },
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
struct Hailstone {
    px: Value,
    py: Value,
    vx: Value,
    vy: Value,
}
impl Hailstone {
    fn position(&self) -> Coord {
        (self.px, self.py)
    }
    fn position_at_time(&self, time: &Value) -> Coord {
        (self.px + time * self.vx, self.py + time * self.vy)
    }
}

type Value = Ratio<i128>;
type Coord = (Value, Value);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orientation {
    Positive,
    Negative,
    Zero,
}

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> u64 {
    let (_, hailstones) = problem_input(input).expect("Failed to parse problem input");
    let segments: Vec<_> = hailstones.iter().flat_map(points_of_interest).collect();

    let mut total = 0;
    for (i, first) in segments.iter().enumerate() {
        for second in segments.iter().skip(i + 1) {
            if intersect(first, second) {
                total += 1;
            }
        }
    }
    return total;
}

fn inside_bounds(coord: &Coord) -> bool {
    let (x, y) = coord;
    let min: Value = TEST_MIN.into();
    let max: Value = TEST_MAX.into();

    (min..=max).contains(&x) && (min..=max).contains(&y)
}

fn orientation(segment: &(Coord, Coord), point: &Coord) -> Orientation {
    let ((x1, y1), (x2, y2)) = segment;
    let (px, py) = point;

    let cross_product = (x1 - px) * (y2 - py) - (y1 - py) * (x2 - px);

    if cross_product.is_positive() {
        Orientation::Positive
    } else if cross_product.is_negative() {
        Orientation::Negative
    } else {
        Orientation::Zero
    }
}

fn orientations(segment: &(Coord, Coord), other: &(Coord, Coord)) -> (Orientation, Orientation) {
    let (p1, p2) = other;
    (orientation(segment, p1), orientation(segment, p2))
}

fn intersect(first: &(Coord, Coord), second: &(Coord, Coord)) -> bool {
    match (orientations(first, second), orientations(second, first)) {
        // If one doesn't flip, then they don't intersect
        ((Orientation::Positive, Orientation::Positive), _)
        | ((Orientation::Negative, Orientation::Negative), _)
        | (_, (Orientation::Positive, Orientation::Positive))
        | (_, (Orientation::Negative, Orientation::Negative)) => false,

        // If one is zeroed, then actually all four are zero;
        // the points are all collinear, and we have to check
        // to see if a point in one segment is contained in the other
        ((Orientation::Zero, Orientation::Zero), _)
        | (_, (Orientation::Zero, Orientation::Zero)) => {
            let (p1, p2) = second;
            segment_contains_point(first, p1) || segment_contains_point(first, p2)
            // (The reverse would also be fine)
        }

        // Otherwise, they have to intersect. This encompasses
        // cases where both of the orientations flip, but also
        // e.g. cases where one flips and one contains a single zero
        _ => true,
    }
}

// Given a segment and a point collinear with them, test whether
// the point is in the segment.
fn segment_contains_point(segment: &(Coord, Coord), point: &Coord) -> bool {
    let ((mut x1, mut y1), (mut x2, mut y2)) = segment;
    let (px, py) = point;

    // Swap so that they are in order; we do this because range
    // syntax doesn't work for reverse orders
    if x2 < x1 {
        swap(&mut x1, &mut x2);
    }
    if y2 < y1 {
        swap(&mut y1, &mut y2);
    }

    // Since we assume it is collinear, we test just by seeing if the
    // point is in the box bounded by the coordinates of the segment
    (x1..=x2).contains(&px) && (y1..=y2).contains(&py)
}

// Using the hailstone and the observation box, we compute
// two points of interest:
// > If the point is in the box, the first is the position,
//   and the second is the intersection with the boundary
// > If the point is outside the box, then it's the two
//   points where the ray intersects the box
// Note that sometimes the two points will coincide.
fn points_of_interest(hailstone: &Hailstone) -> Option<(Coord, Coord)> {
    let test_min: Value = TEST_MIN.into();
    let test_max: Value = TEST_MAX.into();
    match inside_bounds(&hailstone.position()) {
        true => {
            let horizontal_bound = if hailstone.vx > 0.into() {
                test_max
            } else {
                test_min
            };
            let t_horz = (horizontal_bound - hailstone.px) / hailstone.vx;
            let vertical_bound = if hailstone.vy > 0.into() {
                test_max
            } else {
                test_min
            };
            let t_vert = (vertical_bound - hailstone.py) / hailstone.vy;

            // We leave the box when the first coordinate is outside
            let t = min(t_horz, t_vert);
            return Some((hailstone.position(), hailstone.position_at_time(&t)));
        }
        false => {
            let horizontal_bound = if hailstone.vx > 0.into() {
                test_min
            } else {
                test_max
            };
            let t_horz = (horizontal_bound - hailstone.px) / hailstone.vx;
            let vertical_bound = if hailstone.vy > 0.into() {
                test_min
            } else {
                test_max
            };
            let t_vert = (vertical_bound - hailstone.py) / hailstone.vy;

            // This happens if the ray is pointed away from the box.
            // Notice that one of these can be negative and it's fine;
            // for instance, if the ray begins at a point bounded by one
            // of the two coordinates
            if t_horz.is_negative() && t_vert.is_negative() {
                return None;
            }

            // We enter the box when both coordinates are inside
            let t = max(t_horz, t_vert);
            let first_pt = hailstone.position_at_time(&t);

            // We might have moved outside by taking max
            // This happens if the ray is pointed in the direction of the box but misses
            if !inside_bounds(&first_pt) {
                return None;
            }

            // Now, we do the same thing as in the `true` case
            let horizontal_bound = if hailstone.vx > 0.into() {
                test_max
            } else {
                test_min
            };
            let t_horz = (horizontal_bound - hailstone.px) / hailstone.vx;
            let vertical_bound = if hailstone.vy > 0.into() {
                test_max
            } else {
                test_min
            };
            let t_vert = (vertical_bound - hailstone.py) / hailstone.vy;
            let t = min(t_horz, t_vert);
            let second_pt = hailstone.position_at_time(&t);

            return Some((first_pt, second_pt));
        }
    }
}
