use std::collections::BTreeMap;
use std::fs;

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

/* Data Structures */
#[derive(Clone, Copy, Debug)]
enum Direction {
    Right,
    Left,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
struct Symbol(char, char, char);

#[derive(Debug)]
struct SymbolGraph(BTreeMap<Symbol, (Symbol, Symbol)>);

/* Parsers */
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

/* Logic */
fn solve_problem(input: &str) -> u64 {
    let (_, (directions, symbol_graph)) =
        problem_input(input).expect("Failed to parse problem input");
    let mut current_symbol = Symbol('A', 'A', 'A');
    let mut step_counter: u64 = 0;
    for d in directions.iter().cycle() {
        current_symbol =
            next_step(&symbol_graph, &current_symbol, d).expect("Failed to find symbol");
        step_counter += 1;
        if current_symbol == Symbol('Z', 'Z', 'Z') {
            break;
        }
    }

    return step_counter;
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
