use std::fs;
use std::ops::Range;

use core::mem::take;

use nom::bytes::complete::{take_until, take_while};
use nom::character::complete::{newline, u32};
use nom::sequence::terminated;
use nom::IResult;

fn main() {
    println!("Let's solve AOC-03!");
    let input = fs::read_to_string("aoc-03-input.txt").expect("Unable to read input");
    let output = solve_problem(&input);
    println!("Solution: {}", output);
}

#[derive(Debug)]
pub struct PartDatum {
    position: Range<usize>,
    number: u32,
}

#[derive(Debug)]
pub struct SymbolDatum {
    position: usize,
}

pub struct ProblemState {
    last_line_symbols: Vec<SymbolDatum>,
    last_line_leftover_parts: Vec<PartDatum>,
    current_line_symbols: Vec<SymbolDatum>,
    current_line_parts: Vec<PartDatum>,
    current_line_cursor: usize,
    total: u32,
}

impl ProblemState {
    fn new() -> Self {
        return ProblemState {
            last_line_symbols: Vec::new(),
            last_line_leftover_parts: Vec::new(),
            current_line_symbols: Vec::new(),
            current_line_parts: Vec::new(),
            current_line_cursor: 0,
            total: 0,
        };
    }

    fn absorb_update(&mut self, upd: UpdateDatum) {
        match upd {
            UpdateDatum::Part(mut part_datum, offset) => {
                part_datum.position.start += self.current_line_cursor;
                part_datum.position.end += self.current_line_cursor;
                self.current_line_parts.push(part_datum);
                self.current_line_cursor += offset;
            }
            UpdateDatum::Symbol(mut symb_datum, offset) => {
                symb_datum.position += self.current_line_cursor;
                self.current_line_symbols.push(symb_datum);
                self.current_line_cursor += offset;
            }
            UpdateDatum::None(offset) => {
                self.current_line_cursor += offset;
            }
        }
    }

    fn next_line(&mut self) {
        self.last_line_symbols = take(&mut self.current_line_symbols);
        self.last_line_leftover_parts = take(&mut self.current_line_parts);
        self.current_line_cursor = 0;
    }

    fn clear_current_parts(&mut self) {
        let mut uncleared_parts: Vec<PartDatum> = Vec::new();
        for part in take(&mut self.current_line_parts).into_iter() {
            if symbols_meet_range(&self.last_line_symbols, &expand(&part.position))
                || symbols_meet_range(&self.current_line_symbols, &expand(&part.position))
            {
                self.total += part.number;
                continue;
            } else {
                uncleared_parts.push(part);
            }
        }
        self.current_line_parts = uncleared_parts;
    }

    // Leftover parts are cleared only using the new line data, with the assumption that they
    // would not be leftover if they didn't match against symbols on the same line
    fn clear_leftover_parts(&mut self) {
        let mut uncleared_parts: Vec<PartDatum> = Vec::new();
        for part in take(&mut self.last_line_leftover_parts).into_iter() {
            if symbols_meet_range(&self.current_line_symbols, &expand(&part.position)) {
                self.total += part.number;
                continue;
            } else {
                uncleared_parts.push(part);
            }
        }
        self.last_line_leftover_parts = uncleared_parts;
    }
}

fn symbols_meet_range(symbs: &Vec<SymbolDatum>, range: &Range<usize>) -> bool {
    for symb in symbs.iter() {
        if range.contains(&symb.position) {
            return true;
        }
    }
    return false;
}

fn expand(range: &Range<usize>) -> Range<usize> {
    if range.start == 0 {
        return 0..(range.end + 1);
    } else {
        return (range.start - 1)..(range.end + 1);
    }
}

// Update includes:
// - part + offset
// - symbol + offset
// - nothing read => just offset
pub enum UpdateDatum {
    Part(PartDatum, usize),
    Symbol(SymbolDatum, usize),
    None(usize),
}

fn solve_problem(input: &str) -> u32 {
    let mut problem_state = ProblemState::new();
    let mut input_to_read = input;
    loop {
        let (rest, line) = get_line(input_to_read).ok().unwrap();
        input_to_read = rest;
        let mut to_read = line;

        // Absorb data from the current line:
        loop {
            let (rest_of_line, update) = line_datum(&to_read).ok().unwrap();
            to_read = rest_of_line;
            problem_state.absorb_update(update);
            if to_read.is_empty() {
                break;
            }
        }
        println!("Current parts: {:?}", problem_state.current_line_parts);
        println!("Current symbols: {:?}", problem_state.current_line_symbols);
        println!(
            "Previous parts: {:?}",
            problem_state.last_line_leftover_parts
        );
        println!("Previous symbols: {:?}", problem_state.last_line_symbols);
        problem_state.clear_current_parts();
        problem_state.clear_leftover_parts();
        problem_state.next_line();
        println!("Total after clearing: {:?}", problem_state.total);
        println!("-------");

        if input_to_read.is_empty() {
            break;
        }
    }
    return problem_state.total;
}

fn line_datum(input: &str) -> IResult<&str, UpdateDatum> {
    let (rest, dots) = take_while(is_dot)(input)?;
    let offset = dots.len();
    if rest == "" {
        return Ok((rest, UpdateDatum::None(offset)));
    }
    if let Ok((rest, part_no)) = (u32::<&str, ()>)(rest) {
        let length = part_no.to_string().len();
        let start_idx = offset;
        let end_idx = offset + length;
        let part_datum = PartDatum {
            position: start_idx..end_idx,
            number: part_no,
        };
        return Ok((rest, UpdateDatum::Part(part_datum, offset + length)));
    } else {
        let symbol_datum = SymbolDatum { position: offset };
        return Ok((&rest[1..], UpdateDatum::Symbol(symbol_datum, offset + 1)));
    }
}

fn is_dot(c: char) -> bool {
    c == '.'
}

fn get_line(input: &str) -> IResult<&str, &str> {
    terminated(take_until("\n"), newline)(input)
}
