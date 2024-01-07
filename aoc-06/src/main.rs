use nom::{
    bytes::complete::tag,
    character::complete::{char, space0, u64},
    multi::many0,
    sequence::delimited,
    IResult,
};
use std::fs;
use std::iter::zip;

fn main() {
    println!("Let's solve AOC-06!");
    let input = fs::read_to_string("aoc-06-input.txt").expect("Unable to read file");
    let output = solve_problem(&input);
    println!("Solution: {}", output);
}

// Data structures
type BoatRace = (u64, u64);

// Actual solution
fn solve_problem(input: &str) -> u64 {
    let (_, boat_races) = boat_races(input).ok().unwrap();
    return boat_races.iter().map(num_solutions).fold(1, |x, y| x * y);
}

fn num_solutions(boat_race: &BoatRace) -> u64 {
    let (time, distance) = boat_race;
    let mut first_success = time / 2;
    for a in 1..=(time / 2) {
        if a * (time - a) > *distance {
            first_success = a;
            break;
        }
    }
    let sol_count = if time % 2 == 0 {
        ((time / 2) - first_success) * 2 + 1
    } else {
        ((time / 2) - first_success + 1) * 2
    };
    return sol_count;
}

// Parsers
fn boat_races(input: &str) -> IResult<&str, Vec<BoatRace>> {
    let (rest, times) = times(input)?;
    let (rest, distances) = distances(rest)?;
    let races: Vec<BoatRace> = zip(times, distances).collect();
    return Ok((rest, races));
}

fn times(input: &str) -> IResult<&str, Vec<u64>> {
    delimited(tag("Time:"), many0(u64_padded), char('\n'))(input)
}

fn distances(input: &str) -> IResult<&str, Vec<u64>> {
    delimited(tag("Distance:"), many0(u64_padded), char('\n'))(input)
}

fn u64_padded(input: &str) -> IResult<&str, u64> {
    delimited(space0, u64, space0)(input)
}
