use std::fs;
use std::ops::Range;

use nom::bytes::complete::{take_until, take_while};
use nom::character::complete::{newline, u32};
use nom::sequence::terminated;
use nom::IResult;

fn main() {
    println!("Let's solve AOC-03 (Part 2)!");
    let input = fs::read_to_string("aoc-03-input.txt").expect("Unable to read input");
    let output = solve_problem(&input);
    println!("Solution: {}", output);
}

fn expand(range: &Range<usize>) -> Range<usize> {
    if range.start == 0 {
        return 0..(range.end + 1);
    } else {
        return (range.start - 1)..(range.end + 1);
    }
}

fn line_to_range(line_no: usize) -> Range<usize> {
    if line_no == 0 {
        return 0..2;
    } else {
        return (line_no - 1)..(line_no + 2);
    }
}

// usize parameter is an offset which reflects the amount of input consumed
pub enum UpdateDatum {
    Part(PartDatum, usize),
    Gear(GearDatum, usize),
    None(usize),
}

#[derive(Debug)]
pub struct PartDatum {
    adj_range: Range<usize>,
    adj_lines: Range<usize>,
    number: u32,
}

#[derive(Debug)]
pub struct GearDatum {
    adj_pos: usize,
    adj_line: usize,
}

fn solve_problem(input: &str) -> u32 {
    let (_, (parts, gears)) = parse_input(input).ok().unwrap();
    let mut total = 0;
    for g in gears.iter() {
        let mut adj_parts: Vec<u32> = Vec::new();
        for p in parts.iter() {
            if p.adj_range.contains(&g.adj_pos) && p.adj_lines.contains(&g.adj_line) {
                adj_parts.push(p.number);
            }
        }
        if adj_parts.len() == 2 {
            total += adj_parts.iter().fold(1, |x, y| x * y);
        }
    }
    return total;
}

fn parse_input(input: &str) -> IResult<&str, (Vec<PartDatum>, Vec<GearDatum>)> {
    let mut parts: Vec<PartDatum> = Vec::new();
    let mut gears: Vec<GearDatum> = Vec::new();
    let mut cursor = 0;
    let mut line_no = 0;

    let mut input_to_read = input;
    loop {
        let (rest, line) = get_line(input_to_read).ok().unwrap();
        input_to_read = rest;
        let mut to_read = line;

        // Absorb data from the current line
        loop {
            let (rest_of_line, update) = update_datum(to_read, line_no).ok().unwrap();
            to_read = rest_of_line;
            match update {
                UpdateDatum::None(offset) => {
                    cursor += offset;
                }
                UpdateDatum::Gear(mut gear, offset) => {
                    gear.adj_pos += cursor;
                    gears.push(gear);
                    cursor += offset;
                }
                UpdateDatum::Part(mut part, offset) => {
                    let rng = (part.adj_range.start + cursor)..(part.adj_range.end + cursor);
                    part.adj_range = expand(&rng);
                    parts.push(part);
                    cursor += offset;
                }
            }
            if to_read.is_empty() {
                break;
            }
        }

        // If we have more input to parse, continue
        if input_to_read.is_empty() {
            break;
        }
        line_no += 1;
        cursor = 0;
    }
    return Ok(("", (parts, gears)));
}

// Note: the part data don't include their "expanded" diagonal range until they are absorbed
// (i.e. not in this function)
fn update_datum(input: &str, line_no: usize) -> IResult<&str, UpdateDatum> {
    let (rest, dots) = take_while(is_dot)(input)?;
    let offset = dots.len();

    // Nothing to parse, so no update, but we consumed some dots
    if rest == "" {
        return Ok((rest, UpdateDatum::None(offset)));
    }

    // The next thing is a number, which we parse into a Part update
    if let Ok((rest, part_no)) = (u32::<&str, ()>)(rest) {
        let length = part_no.to_string().len();
        let start_idx = offset;
        let end_idx = offset + length;
        let part_datum = PartDatum {
            adj_range: start_idx..end_idx,
            adj_lines: line_to_range(line_no),
            number: part_no,
        };
        return Ok((rest, UpdateDatum::Part(part_datum, offset + length)));
    }
    // Consume 1 character, returning a Gear update if it's a gear
    else {
        if rest.chars().next() == Some('*') {
            let gear_datum = GearDatum {
                adj_pos: offset,
                adj_line: line_no,
            };
            return Ok((&rest[1..], UpdateDatum::Gear(gear_datum, offset + 1)));
        } else {
            return Ok((&rest[1..], UpdateDatum::None(offset + 1)));
        }
    }
}

fn is_dot(c: char) -> bool {
    c == '.'
}

fn get_line(input: &str) -> IResult<&str, &str> {
    terminated(take_until("\n"), newline)(input)
}
