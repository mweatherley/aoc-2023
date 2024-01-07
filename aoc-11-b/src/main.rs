use nom::bytes::complete::take_until;
use nom::character::complete::{char, newline};
use nom::combinator::map;
use nom::error::ParseError;
use nom::multi::many0;
use nom::sequence::{pair, preceded, tuple};
use nom::{IResult, Offset, Parser};
use std::collections::BTreeMap;
use std::{cell::RefCell, collections::HashMap, fs};

use std::time::Instant;

fn main() {
    println!("Let's solve AOC-11!");
    let now = Instant::now();
    let input = fs::read_to_string("aoc-11-input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

/* --------------- */
/* Data Structures */
/* --------------- */

type GalaxyMap = HashMap<(usize, usize), usize>;

type Dims = (usize, usize);

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> IResult<&str, (Dims, GalaxyMap)> {
    let (_, (length, _)) = with_offset(pair(take_until("\n"), char('\n')))(input)?;
    let width = length - 1;

    let parser = ProblemParser {
        data: RefCell::new(ProblemParserData::new()),
    };
    let (rest, _) = parser.parse_input()(input)?;
    let data = parser.data.into_inner();
    Ok((rest, ((width, data.current_line), data.galaxy_map)))
}

#[derive(Clone)]
struct ProblemParserData {
    current_line: usize,
    current_cursor: usize,
    galaxy_map: GalaxyMap,
}

impl ProblemParserData {
    fn new() -> Self {
        ProblemParserData {
            current_line: 0,
            current_cursor: 0,
            galaxy_map: HashMap::new(),
        }
    }
}

struct ProblemParser {
    data: RefCell<ProblemParserData>,
}

impl ProblemParser {
    fn next_line(&self) {
        let mut data = self.data.borrow_mut();
        data.current_line = data.current_line + 1;
        data.current_cursor = 0;
    }

    fn parse_input<'a>(&'a self) -> impl FnMut(&str) -> IResult<&str, ()> + 'a {
        |input| {
            map(
                many0(tuple((
                    many0(self.parse_galaxy()),
                    self.parse_blank_space(),
                    self.parse_newline(),
                ))),
                |_| (),
            )(input) // Literally kill the output
        }
    }

    fn parse_galaxy<'a>(&'a self) -> impl FnMut(&str) -> IResult<&str, ()> + 'a {
        |input| {
            let (rest, (offset, _)) = with_offset(galaxy)(input)?;
            let mut data = self.data.borrow_mut();
            data.current_cursor += offset;
            let y_position = data.current_line;
            let x_position = data.current_cursor - 1;
            data.galaxy_map.insert((x_position, y_position), 0);
            Ok((rest, ()))
        }
    }

    fn parse_blank_space<'a>(&'a self) -> impl FnMut(&str) -> IResult<&str, ()> + 'a {
        |input| {
            let (rest, (offset, _)) = with_offset(many0(char('.')))(input)?;
            let mut data = self.data.borrow_mut();
            data.current_cursor += offset;
            Ok((rest, ()))
        }
    }

    fn parse_newline<'a>(&'a self) -> impl FnMut(&str) -> IResult<&str, ()> + 'a {
        |input| {
            let (rest, _) = newline(input)?;
            self.next_line();
            Ok((rest, ()))
        }
    }
}

fn galaxy(input: &str) -> IResult<&str, char> {
    preceded(many0(char('.')), char('#'))(input)
}

fn with_offset<F, I, O, E>(mut parser: F) -> impl FnMut(I) -> IResult<I, (usize, O), E>
where
    F: Parser<I, O, E>,
    I: Clone + Offset,
    E: ParseError<I>,
{
    move |input: I| {
        let i = input.clone();
        match parser.parse(i) {
            Ok((rest, output)) => {
                let offset = input.offset(&rest);
                return Ok((rest, (offset, output)));
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> usize {
    let (_, ((width, height), galaxy_map)) =
        problem_input(input).expect("Failed to parse problem input");

    // All of this is just doing expansion:
    // -----------------------------------
    let mut row_mass: BTreeMap<usize, usize> = BTreeMap::new();
    let mut column_mass: BTreeMap<usize, usize> = BTreeMap::new();
    let mut total_mass: usize = 0;
    for (x, y) in galaxy_map.keys() {
        row_mass.entry(*y).and_modify(|v| *v += 1).or_insert(1);
        column_mass.entry(*x).and_modify(|v| *v += 1).or_insert(1);
        total_mass += 1;
    }

    // Iterate over rows and columns;
    // for each blank one, we store (val, location), so that every point with x-coordinate (say)
    // `x > location` (and preceding the preceding marker) needs to have its value increased by `val`
    let mut row_thresholds: Vec<(usize, usize)> = vec![];
    let mut blank_rows = 0;
    for i in 0..height {
        if !row_mass.contains_key(&i) {
            blank_rows += 999999;
            row_thresholds.push((blank_rows, i));
        }
    }
    let mut column_thresholds: Vec<(usize, usize)> = vec![];
    let mut blank_columns = 0;
    for i in 0..width {
        if !column_mass.contains_key(&i) {
            blank_columns += 999999;
            column_thresholds.push((blank_columns, i));
        }
    }

    // This pair of loops is where we finally do the actual work of replacing the mass
    // functions with the expanded versions.
    let (mut expanded_row_mass, weighted_row_total) =
        expand_mass_function(&mut row_mass, &mut row_thresholds);
    let (mut expanded_column_mass, weighted_column_total) =
        expand_mass_function(&mut column_mass, &mut column_thresholds);

    // Now we have the finalized versions of our row and column mass functions;
    // using them, we compute the total distance
    let row_total = linear_distance_total(&mut expanded_row_mass, weighted_row_total, total_mass);
    let column_total =
        linear_distance_total(&mut expanded_column_mass, weighted_column_total, total_mass);
    return row_total + column_total;
}

fn expand_mass_function(
    mass_function: &mut BTreeMap<usize, usize>,
    thresholds: &mut Vec<(usize, usize)>,
) -> (BTreeMap<usize, usize>, usize) {
    let mut expanded_mass_function: BTreeMap<usize, usize> = BTreeMap::new();
    let mut weighted_total: usize = 0;
    let mut maybe_thresh = thresholds.pop();
    while !mass_function.is_empty() {
        let (pt, val) = mass_function.pop_last().unwrap();

        // We iterate until we find a lower threshold for our point
        loop {
            match maybe_thresh {
                None => {
                    expanded_mass_function.insert(pt, val);
                    weighted_total += pt * val;
                    break;
                }
                Some((offset, threshold)) => {
                    if pt < threshold {
                        maybe_thresh = thresholds.pop();
                        continue;
                    } else {
                        expanded_mass_function.insert(pt + offset, val);
                        weighted_total += (pt + offset) * val;
                        break;
                    }
                }
            }
        }
    }
    return (expanded_mass_function, weighted_total);
}

fn linear_distance_total(
    mass_function: &mut BTreeMap<usize, usize>,
    weighted_mass: usize,
    total_mass: usize,
) -> usize {
    let mut leftover_weighted_mass = weighted_mass;
    let mut leftover_mass = total_mass;
    let mut total = 0;
    while !mass_function.is_empty() {
        let (pt, val) = mass_function.pop_first().unwrap();
        leftover_mass = leftover_mass - val;
        leftover_weighted_mass = leftover_weighted_mass - (pt * val);
        total += (leftover_weighted_mass * val) - (leftover_mass * pt * val);
    }
    return total;
}
