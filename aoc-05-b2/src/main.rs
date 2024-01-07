use std::cmp::{max, min};
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
#[derive(Clone, Debug)]
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

// Parsers
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

// Logical functions
fn solve_problem(input: &str) -> i64 {
    let (_, (seed_ranges, maps)) = parse_input(input).ok().unwrap();
    let tot_function = compose_all(&maps);
    let mut output_vals: Vec<i64> = vec![];
    for seed_range in seed_ranges.iter() {
        let crit_points = crit_points(&seed_range, &tot_function);
        for pt in crit_points.into_iter() {
            output_vals.push(composite_fn(&tot_function)(pt));
        }
    }
    return output_vals.into_iter().reduce(min).unwrap();
}

fn translate_range(range: &Range<i64>, offset: i64) -> Range<i64> {
    return (range.start - offset)..(range.end - offset);
}

// Given a range and another range to intersect with it, returns the intersection
// of the two in a vector, and the intersection's complement pieces as ranges in another vector
fn intersect(chompee: &Range<i64>, chomper: &Range<i64>) -> (Vec<Range<i64>>, Vec<Range<i64>>) {
    let mut leftover_segments: Vec<Range<i64>> = vec![];
    // RHS-exhausting pattern
    if chompee.end <= chomper.end {
        // There is actually overlap
        if chompee.end > chomper.start {
            let cut_point = max(chompee.start, chomper.start);
            let intersection_range = cut_point..chompee.end;
            if cut_point != chompee.start {
                leftover_segments.push(chompee.start..chomper.start);
            }
            return (vec![intersection_range], leftover_segments);
        }
        // There is no overlap
        else {
            return (vec![], vec![chompee.clone()]);
        }
    }
    // LHS-exhausting pattern
    else if chompee.start >= chomper.start {
        // There is actually overlap
        if chompee.start < chomper.end {
            let cut_point = min(chompee.end, chomper.end);
            let intersection_range = chompee.start..cut_point;
            if cut_point != chompee.end {
                // This is technically precluded by the previous branch
                leftover_segments.push(chomper.end..chompee.end);
            }
            return (vec![intersection_range], leftover_segments);
        }
        // There is no overlap
        else {
            return (vec![], vec![chompee.clone()]);
        }
    }
    // Intersection in the middle of the thing
    else {
        let intersection_range = chomper.start..chomper.end;
        leftover_segments.push(chompee.start..chomper.start);
        leftover_segments.push(chomper.end..chompee.end);
        return (vec![intersection_range], leftover_segments);
    }
}

// Given a partial for a function 'g' and a function 'f' defined by a vector of partials,
// we compute the function composite as a vector of partials.
fn expand(fn_piece: &FunctionPiece, next_pieces: &CompositeFunction) -> CompositeFunction {
    let mut output_partials: CompositeFunction = vec![];
    let mut to_process: Vec<Range<i64>> = vec![];
    to_process.push(fn_piece.domain.clone());

    // iterate over the next possible pieces
    for future in next_pieces.iter() {
        let mut leftovers: Vec<Range<i64>> = vec![];

        // Iterate over remaining chunks of domain
        for domain_piece in to_process.iter() {
            // Compute the intersection of the domain with the inverse translated domain of the future
            let (intersections, mut extras) = intersect(
                domain_piece,
                &translate_range(&future.domain, fn_piece.translation),
            );

            // When the domain gets split up, add the chunks to the leftovers
            leftovers.append(&mut extras);
            for domain_overlap in intersections.into_iter() {
                let new_piece = FunctionPiece {
                    domain: domain_overlap,
                    translation: fn_piece.translation + future.translation,
                };
                output_partials.push(new_piece);
            }
        }
        to_process.clear();
        to_process = leftovers;
    }
    // Process leftover blank ranges with no intersection, on which fg(x) = g(x)
    for blank_range in to_process.into_iter() {
        let blank_piece = FunctionPiece {
            domain: blank_range,
            translation: fn_piece.translation,
        };
        output_partials.push(blank_piece);
    }
    return output_partials;
}

// Compute the composite of first and second (the first is applied first) as a set of linear things
fn compose(first: &CompositeFunction, second: &CompositeFunction) -> CompositeFunction {
    let mut total_function: Vec<FunctionPiece> = vec![];

    for first_fn in first.iter() {
        total_function.append(&mut expand(first_fn, second));
    }

    for second_fn in second.iter() {
        let mut remaining: Vec<Range<i64>> = vec![second_fn.domain.clone()];
        for first_fn in first.iter() {
            let mut all_survivors: Vec<Range<i64>> = vec![];
            for segment in remaining.iter() {
                let (_, mut survivors) = intersect(segment, &first_fn.domain);
                all_survivors.append(&mut survivors);
            }
            remaining.clear();
            remaining = all_survivors;
        }
        for default_domain in remaining.into_iter() {
            let default_piece = FunctionPiece {
                domain: default_domain,
                translation: second_fn.translation,
            };
            total_function.push(default_piece);
        }
    }
    return total_function;
}

fn compose_all(functions: &Vec<CompositeFunction>) -> CompositeFunction {
    let mut total_function: CompositeFunction = vec![];
    for f in functions.iter() {
        total_function = compose(&total_function, f);
    }
    return total_function;
}

/// Given a function and a domain range, output a vector of all of the
/// critical points (points at which a minimum could occur)
fn crit_points(range: &Range<i64>, func: &CompositeFunction) -> Vec<i64> {
    // The minimum of the range always has to be checked
    let mut crit_points: Vec<i64> = vec![range.start];
    for piece in func.iter() {
        if range.contains(&piece.domain.start) {
            crit_points.push(piece.domain.start);
        }
        // Necessary because the end of one domain is the start of the neutral domain,
        // although this will sometimes double-count if two partials are adjacent
        if range.contains(&piece.domain.end) {
            crit_points.push(piece.domain.end);
        }
    }
    return crit_points;
}
