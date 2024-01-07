use std::cmp::min;
use std::collections::BTreeMap;
use std::{fs, time::Instant};

use num::integer::lcm;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char, newline},
    combinator::{map, value},
    multi::many0,
    sequence::{delimited, separated_pair, terminated, tuple},
    IResult,
};

fn main() {
    let now = Instant::now();
    println!("Let's solve AOC-08!");
    let input = fs::read_to_string("aoc-08-input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Solution: {}", solution);
    println!("Time elapsed: {:?}", now.elapsed());
}

/* --------------- */
/* Data Structures */
/* --------------- */

#[derive(Clone, Copy, Debug)]
enum Direction {
    Right,
    Left,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
struct Symbol(char, char, char);

#[derive(Debug)]
struct SymbolGraph(BTreeMap<Symbol, (Symbol, Symbol)>);

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> IResult<&str, (Vec<Direction>, SymbolGraph)> {
    separated_pair(directions, tag("\n\n"), symbol_graph)(input)
}

fn directions(input: &str) -> IResult<&str, Vec<Direction>> {
    many0(direction)(input)
}

fn direction(input: &str) -> IResult<&str, Direction> {
    alt((
        value(Direction::Left, char('L')),
        value(Direction::Right, char('R')),
    ))(input)
}

fn symbol_graph(input: &str) -> IResult<&str, SymbolGraph> {
    map(many0(symbol_line), |symbol_lines| {
        SymbolGraph(symbol_lines.into_iter().collect())
    })(input)
}

fn symbol_line(input: &str) -> IResult<&str, (Symbol, (Symbol, Symbol))> {
    terminated(separated_pair(symbol, tag(" = "), symbol_pair), newline)(input)
}

fn symbol_pair(input: &str) -> IResult<&str, (Symbol, Symbol)> {
    delimited(
        char('('),
        separated_pair(symbol, tag(", "), symbol),
        char(')'),
    )(input)
}

fn symbol(input: &str) -> IResult<&str, Symbol> {
    map(tuple((anychar, anychar, anychar)), |(x, y, z)| {
        Symbol(x, y, z)
    })(input)
}

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> i128 {
    let (_, (directions, symbol_graph)) =
        problem_input(input).expect("Failed to parse problem input");
    let SymbolGraph(graph) = &symbol_graph;
    let total_directions = directions.len();
    let total_symbols = graph.keys().len();
    let starting_symbols: Vec<&Symbol> = graph.keys().filter(|s| ends_in_A(*s)).collect();

    // Start by proccessing every future that starts with the symbols ending in 'A' until we reach a point where we know
    // they must have looped for purely mathematical reasons (literally the pigeonhole principle)
    let mut processed_futures: Vec<(usize, Vec<usize>)> = vec![];
    for start in starting_symbols.iter() {
        let (period, z_history) = get_future(
            **start,
            &directions,
            &symbol_graph,
            total_directions * total_symbols,
            total_directions,
        );
        println!("Period: {:?}, Z-history: {:?}", period, z_history);
        processed_futures.push((period, z_history));
    }

    // Next, compute any universal overlap between Z-histories; if we find one, we are done.
    let just_z_histories: Vec<Vec<usize>> = processed_futures
        .clone()
        .into_iter()
        .map(|(_, y)| y)
        .collect();
    let overlap = just_z_histories
        .into_iter()
        .reduce(|x, y| vec_intersect(&x, &y))
        .expect("Failed to find any histories");
    println!("Overlap of all Z-histories: {:?}", overlap);
    if !overlap.is_empty() {
        return overlap.into_iter().reduce(min).unwrap() as i128;
    }
    // The first common intersection lies beyond the horizon!
    else {
        let asymptotics_data: Vec<(i128, Vec<i128>)> = processed_futures
            .iter()
            .map(|x| get_asymptotics(x.clone(), total_directions * total_symbols))
            .collect();
        let (_total_period, intersected_asymp) = asymptotics_data
            .into_iter()
            .reduce(|x, y| intersect_asymptotics(&x, &y))
            .unwrap();
        let first_intersection = intersected_asymp.iter().reduce(min).unwrap();
        return first_intersection + (total_directions * total_symbols) as i128;
    }
}

fn vec_intersect<T>(vec1: &Vec<T>, vec2: &Vec<T>) -> Vec<T>
where
    T: PartialEq + Clone,
{
    let mut common: Vec<T> = vec![];
    for val1 in vec1.iter() {
        for val2 in vec2.iter() {
            if val1 == val2 {
                common.push(val1.clone())
            }
        }
    }
    return common;
}

fn get_asymptotics(future: (usize, Vec<usize>), sample_size: usize) -> (i128, Vec<i128>) {
    let mut z_asymptotics: Vec<i128> = vec![];
    let (period, z_history) = future;
    for step in z_history.iter() {
        let step = *step as i128;
        let sample_size = sample_size as i128;
        let period = period as i128;
        let translated_step = step - &sample_size + period;
        if (0..period).contains(&translated_step) {
            z_asymptotics.push(translated_step);
        }
    }
    return (period as i128, z_asymptotics);
}

// Almost all of the compute time is spent on this function
fn intersect_asymptotics(
    asymp1: &(i128, Vec<i128>),
    asymp2: &(i128, Vec<i128>),
) -> (i128, Vec<i128>) {
    let (period1, z_asymp1) = asymp1;
    let (period2, z_asymp2) = asymp2;
    let period = lcm(*period1, *period2);
    let div1 = period / period1;
    let div2 = period / period2;
    let mut asymp: Vec<i128> = vec![];
    for val1 in z_asymp1.iter() {
        for val2 in z_asymp2.iter() {
            for mult1 in 0..div1 {
                for mult2 in 0..div2 {
                    if val1 + (mult1 * period1) == val2 + (mult2 * period2) {
                        asymp.push(val1 + (mult1 * period1));
                    }
                }
            }
        }
    }
    println!("Period: {:?}, Z-asymptotics: {:?}", period, asymp);
    return (period, asymp);
}

fn get_future(
    starting_symbol: Symbol,
    directions: &Vec<Direction>,
    graph: &SymbolGraph,
    limit: usize,
    tape_size: usize,
) -> (usize, Vec<usize>) {
    let mut current_symbol = starting_symbol;

    // Steps taken after the starting position
    let mut step_counter: usize = 0;

    // The entire history up to `limit`
    let mut history: Vec<(Symbol, usize)> = vec![(starting_symbol, 0)];

    // The locations of symbols ending in Z within the history
    let mut z_history: Vec<usize> = vec![];

    for d in directions.iter().cycle() {
        // Take one step
        current_symbol = next_step(&graph, &current_symbol, d).expect("Failed to find symbol");
        step_counter += 1;

        history.push((current_symbol, step_counter % tape_size));

        // Add valid ending positions to the `z_history`
        if ends_in_Z(&current_symbol) {
            z_history.push(step_counter)
        }
        if step_counter == limit {
            break;
        }
    }

    // Now, using this, let's find the asymptotic period of our guy (which is necessarily at least `tape_size`)
    let final_state = history.pop();
    let mut current_state;
    let mut steps_backward: usize = 0;
    loop {
        current_state = history.pop();
        steps_backward += 1;
        if current_state == final_state {
            break;
        }
    }

    return (steps_backward, z_history);
}

fn next_step(graph: &SymbolGraph, current: &Symbol, direction: &Direction) -> Option<Symbol> {
    let SymbolGraph(graph) = graph;
    if let Some((left, right)) = graph.get(current) {
        match direction {
            Direction::Left => Some(*left),
            Direction::Right => Some(*right),
        }
    } else {
        return None;
    }
}

fn ends_in_A(symbol: &Symbol) -> bool {
    let Symbol(c1, c2, c3) = symbol;
    match (c1, c2, c3) {
        (_, _, 'A') => true,
        _ => false,
    }
}

fn ends_in_Z(symbol: &Symbol) -> bool {
    let Symbol(c1, c2, c3) = symbol;
    match (c1, c2, c3) {
        (_, _, 'Z') => true,
        _ => false,
    }
}
