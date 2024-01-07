use nom::bytes::complete::take;
use nom::{
    character::complete::{anychar, char, newline, space0, u32},
    combinator::{map_opt, map_res},
    multi::many0,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};
use std::{char, fs, num::ParseIntError};

fn main() {
    println!("Let's solve AOC-18!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> IResult<&str, Vec<Instruction>> {
    many0(instruction)(input)
}

fn instruction(input: &str) -> IResult<&str, Instruction> {
    let (rest, direction) = terminated(direction, space0)(input)?;
    let (rest, distance) = terminated(distance, space0)(rest)?;
    let (rest, color) = terminated(color, newline)(rest)?;
    let instruction = Instruction {
        direction,
        distance,
        color,
    };
    Ok((rest, instruction))
}

fn color(input: &str) -> IResult<&str, Color> {
    delimited(
        char('('),
        preceded(
            char('#'),
            tuple((
                map_res(take(2u8), str_to_hex),
                map_res(take(2u8), str_to_hex),
                map_res(take(2u8), str_to_hex),
            )),
        ),
        char(')'),
    )(input)
}

fn str_to_hex(s: &str) -> Result<u8, ParseIntError> {
    u8::from_str_radix(s, 16)
}

fn distance(input: &str) -> IResult<&str, isize> {
    map_res(u32, |n| n.try_into())(input)
}

fn direction(input: &str) -> IResult<&str, Direction> {
    map_opt(anychar, char_to_dir)(input)
}

fn char_to_dir(c: char) -> Option<Direction> {
    match c {
        'U' => Some(Direction::N),
        'D' => Some(Direction::S),
        'L' => Some(Direction::W),
        'R' => Some(Direction::E),
        _ => None,
    }
}

/* --------------- */
/* Data Structures */
/* --------------- */
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Direction {
    N,
    S,
    E,
    W,
}

type Coord = (isize, isize);
type Color = (u8, u8, u8);

#[derive(Clone, Copy, Debug)]
struct Instruction {
    direction: Direction,
    distance: isize,
    color: Color,
}

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> isize {
    let (_, instructions) = problem_input(input).expect("Failed to parse problem input");
    let start_coord: Coord = (0, 0);
    let mut current_coord: Coord = start_coord;
    let mut volume: isize = 0;
    let mut boundary_volume: isize = 0;
    for instruction in instructions.iter() {
        let next_coord =
            coord_in_direction(current_coord, instruction.direction, instruction.distance);
        volume += shoelace(current_coord, next_coord); // Volume contribution by shoelace formula
        boundary_volume += instruction.distance;
        current_coord = next_coord;
    }

    // Shoelace formula actually produces twice the volume...
    volume = volume / 2;
    // And it is signed by the boundary orientation, which we don't actually know a priori.
    volume = volume.abs();

    // Now, we account for the contribution of the boundary. Since we imagine that each 'dig'
    // action is centered on a lattice point, the `volume` misses the area outside the line
    // that passes through these lattice points. That area is approximately half the number of
    // tiles dug out, but because of corner contributions, the area on the outside is actually
    // exactly 1 larger than that. (And the area on the inside would be 1 smaller.)
    let boundary_contribution = boundary_volume / 2 + 1;
    let dug_volume = volume + boundary_contribution;
    return dug_volume;
}

fn shoelace(first: Coord, second: Coord) -> isize {
    let (x1, y1) = first;
    let (x2, y2) = second;
    (x1 * y2) - (x2 * y1)
}

fn coord_in_direction(start: Coord, direction: Direction, distance: isize) -> Coord {
    let (x, y) = start;
    match direction {
        Direction::N => (x, y + distance),
        Direction::S => (x, y - distance),
        Direction::E => (x + distance, y),
        Direction::W => (x - distance, y),
    }
}
