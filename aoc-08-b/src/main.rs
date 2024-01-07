use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::sync::{Mutex, RwLock};
use std::thread;

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
    println!("Let's solve AOC-08!");
    let input = fs::read_to_string("aoc-08-input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Solution: {}", solution);
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

struct SharedPad {
    limit: usize,
    pad: Mutex<HashMap<u64, usize>>,
    flag: RwLock<Option<u64>>,
}

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

fn solve_problem(input: &str) -> u64 {
    let (_, (directions, symbol_graph)) =
        problem_input(input).expect("Failed to parse problem input");
    let SymbolGraph(graph) = &symbol_graph;
    let starting_symbols: Vec<&Symbol> = graph.keys().filter(|s| ends_in_A(*s)).collect();
    let total_threads = starting_symbols.len();
    let shared_pad = SharedPad {
        limit: total_threads,
        pad: Mutex::new(HashMap::new()),
        flag: RwLock::new(None),
    };
    thread::scope(|s| {
        for starting_symbol in starting_symbols.into_iter() {
            s.spawn(|| run_off(*starting_symbol, &directions, &symbol_graph, &shared_pad));
        }
    });

    let flag_after = shared_pad
        .flag
        .read()
        .expect("Failed to read shared flag after scoped threads");
    let output = flag_after.unwrap();
    return output;
}

fn run_off(
    starting_symbol: Symbol,
    directions: &Vec<Direction>,
    graph: &SymbolGraph,
    shared: &SharedPad,
) {
    let mut current_symbol = starting_symbol;
    let mut step_counter: u64 = 0;
    for d in directions.iter().cycle() {
        if step_counter % 100 == 0 {
            let flag = shared
                .flag
                .read()
                .expect("Failed to acquire read lock on shared flag");
            if (*flag).is_some() {
                return;
            }
        }
        current_symbol = next_step(&graph, &current_symbol, d).expect("Failed to find symbol");
        step_counter += 1;
        if ends_in_Z(&current_symbol) {
            let mut pad = shared
                .pad
                .lock()
                .expect("Failed to acquire lock on shared pad");
            pad.entry(step_counter).and_modify(|x| *x += 1).or_insert(1);
            if *pad.get(&step_counter).unwrap() == shared.limit {
                println!("I am so super cool");
                let mut flag = shared
                    .flag
                    .write()
                    .expect("Failed to acquire write lock on shared flag");
                if flag.is_some() {
                    return;
                }
                *flag = Some(step_counter);
            }
        }
    }
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
