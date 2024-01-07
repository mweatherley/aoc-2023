use nom::{
    bytes::complete::tag,
    character::complete::{char, digit1, space0, u64},
    multi::many0,
    sequence::delimited,
    IResult,
};
use std::fs;

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
    let (_, boat_race) = boat_race(input).ok().unwrap();
    return num_solutions(&boat_race);
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
fn boat_race(input: &str) -> IResult<&str, BoatRace> {
    let (rest, time) = time(input)?;
    let (rest, distance) = distance(rest)?;
    let race = (time, distance);
    return Ok((rest, race));
}

fn time(input: &str) -> IResult<&str, u64> {
    delimited(tag("Time:"), u64_spaced_out, char('\n'))(input)
}

fn distance(input: &str) -> IResult<&str, u64> {
    delimited(tag("Distance:"), u64_spaced_out, char('\n'))(input)
}

fn nonspace_padded(input: &str) -> IResult<&str, &str> {
    delimited(space0, digit1, space0)(input)
}

fn u64_spaced_out(input: &str) -> IResult<&str, u64> {
    let (rest, pieces) = many0(nonspace_padded)(input)?;
    let mut stripped_str = String::new();
    for s in pieces.iter() {
        stripped_str.push_str(s);
    }
    let (_, val) = u64::<&str, ()>(stripped_str.as_str()).ok().unwrap();
    return Ok((rest, val));
}
