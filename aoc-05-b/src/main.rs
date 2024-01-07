use std::cmp::min;
use std::fs;
use std::ops::Range;

use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{i64, newline, space0};
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

fn main() {
    println!("Let's solve AOC-05!");
    let input = fs::read_to_string("aoc-05-input.txt").expect("Unable to read file");
    let output = solve_problem(&input);
    println!("Solution: {}", output);
}

// If you are in the domain, you get moved by the translation
// i.e. |x| x + translation
#[derive(Clone)]
pub struct FunctionPiece {
    domain: Range<i64>,
    translation: i64,
}

pub type CompositeFunction = Vec<FunctionPiece>;

pub type SeedRange = Range<i64>;

fn composite_fn(pieces: &CompositeFunction) -> impl Fn(i64) -> i64 {
    let pieces_two = pieces.clone();
    let f = move |x: i64| -> i64 {
        for p in pieces_two.iter() {
            if p.domain.contains(&x) {
                return x + p.translation;
            }
        }
        return x;
    };
    return f;
}

fn solve_problem(input: &str) -> i64 {
    let mut min_answer = i64::MAX;
    let (_, (seed_ranges, maps)) = parse_input(input).ok().unwrap();
    let mut range_counter = 1;
    for r in seed_ranges {
        println!("Range number: {}", range_counter);

        let mut seed_counter = 1;
        for s in r.into_iter() {
            if seed_counter % 1000 == 0 {
                println!("Seed number: {}", seed_counter);
            }
            let mut cur_val = s;
            for m in maps.iter() {
                cur_val = composite_fn(m)(cur_val);
            }
            min_answer = min(min_answer, cur_val);
            seed_counter += 1;
        }
        range_counter += 1;
    }
    return min_answer;
}

fn parse_input(input: &str) -> IResult<&str, (Vec<SeedRange>, Vec<CompositeFunction>)> {
    let (rest, seed_ranges) = seed_ranges(input)?;
    let (rest, maps) = many0(function)(rest)?;
    return Ok((rest, (seed_ranges, maps)));
}

fn seed_ranges(input: &str) -> IResult<&str, Vec<SeedRange>> {
    preceded(tag("seeds:"), many0(seed_range))(input)
}

fn seed_range(input: &str) -> IResult<&str, SeedRange> {
    let (rest, (seed_start, seed_window_size)) = pair(padded_i64, padded_i64)(input)?;

    let seed_range = SeedRange {
        start: seed_start,
        end: seed_start + seed_window_size,
    };
    return Ok((rest, seed_range));
}

fn function(input: &str) -> IResult<&str, CompositeFunction> {
    let (rest, _) = preceded(take_until("map:"), tag("map:\n"))(input)?;

    let (rest, fn_pieces) = many0(function_piece)(rest)?;

    return Ok((rest, fn_pieces));
}

fn function_piece(input: &str) -> IResult<&str, FunctionPiece> {
    let (rest, (dest_start, source_start, window_size)) =
        terminated(tuple((padded_i64, padded_i64, padded_i64)), newline)(input)?;

    let fn_piece = FunctionPiece {
        domain: source_start..(source_start + window_size),
        translation: dest_start - source_start,
    };

    return Ok((rest, fn_piece));
}

fn padded_i64(input: &str) -> IResult<&str, i64> {
    delimited(space0, i64, space0)(input)
}
