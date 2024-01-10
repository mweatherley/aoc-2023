use nom::branch::alt;
use nom::character::complete::{char, newline, u32};
use nom::combinator::value;
use nom::combinator::{map, opt};
use nom::multi::{many0, many1};
use nom::sequence::separated_pair;
use nom::{sequence::terminated, IResult};
use std::fs;

fn main() {
    println!("Let's solve AOC-12!");
    let input = fs::read_to_string("aoc-12-input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Solution: {:?}", solution);
}

/* --------------- */
/* Data Structures */
/* --------------- */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Unknown,
    Broken,
    Okay,
}

type StateList = Vec<State>;
type BlockList = Vec<u32>;

#[derive(Clone)]
struct SpringProblem {
    states: StateList,
    blocks: BlockList,
}

enum ReductionResult {
    // Store the reduced problem
    ReducedProblem(SpringProblem),
    // Store the index of the last '?' when we run into one
    Ambiguous(usize),
    // Even a local solution is verified to be impossible
    Insolvable,
    EmptyStates,
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> IResult<&str, Vec<SpringProblem>> {
    many0(spring_problem)(input)
}

fn spring_problem(input: &str) -> IResult<&str, SpringProblem> {
    map(
        terminated(separated_pair(states, char(' '), blocks), newline),
        |(st, bl)| SpringProblem {
            states: st,
            blocks: bl,
        },
    )(input)
}

fn states(input: &str) -> IResult<&str, StateList> {
    many1(alt((
        value(State::Unknown, char('?')),
        value(State::Okay, char('.')),
        value(State::Broken, char('#')),
    )))(input)
}

fn blocks(input: &str) -> IResult<&str, BlockList> {
    many1(terminated(u32, opt(char(','))))(input)
}

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> u32 {
    let (_, problems) = problem_input(input).expect("Failed to parse problem input");
    let mut total = 0;
    for problem in problems.into_iter() {
        total += total_solutions(problem);
    }
    return total;
}

fn total_solutions(spring_problem: SpringProblem) -> u32 {
    // Base case: The spring problem has an empty state.
    if spring_problem.states.is_empty() {
        if spring_problem.blocks.is_empty() {
            return 1;
        } else {
            return 0;
        }
    }
    // If it's not empty, try to reduce it by looking at its end
    match try_reduce(&spring_problem) {
        ReductionResult::EmptyStates => {
            panic!("Tried to reduce an empty spring problem");
        }
        ReductionResult::Insolvable => {
            return 0;
        }
        ReductionResult::ReducedProblem(red_problem) => {
            return total_solutions(red_problem);
        }
        ReductionResult::Ambiguous(idx) => {
            let mut try_broken = spring_problem.clone();
            try_broken.states[idx] = State::Broken;

            let mut try_okay = spring_problem.clone();
            try_okay.states[idx] = State::Okay;

            return total_solutions(try_broken) + total_solutions(try_okay);
        }
    }
}

fn try_reduce(spring_problem: &SpringProblem) -> ReductionResult {
    match spring_problem.states.last() {
        None => {
            return ReductionResult::EmptyStates;
        }
        Some(state) => {
            let mut last_state = *state;
            let mut idx = spring_problem.states.len() - 1;
            loop {
                match last_state {
                    // We found '.', which usually means to keep looking back;
                    // every other branch of this `match` statement leads to a `return`
                    State::Okay => {
                        // We reached the end of the thing and only found '.' the whole time;
                        // We return the "reduced problem" consisting of no states and the same blocks
                        if idx == 0 {
                            let blocks_copy = spring_problem.blocks.clone();
                            let reduced_problem = SpringProblem {
                                states: vec![],
                                blocks: blocks_copy,
                            };
                            return ReductionResult::ReducedProblem(reduced_problem);
                        } else {
                            idx = idx - 1;
                            last_state = spring_problem.states[idx];
                            continue;
                        }
                    }

                    // We found '?' before '#', so this state is not terminally solvable
                    State::Unknown => {
                        return ReductionResult::Ambiguous(idx);
                    }
                    State::Broken => {
                        let mut reduced_problem = spring_problem.clone();
                        let maybe_block = reduced_problem.blocks.pop();
                        match maybe_block {
                            None => {
                                return ReductionResult::Insolvable;
                            }
                            Some(block_size) => {
                                let block_size = block_size as usize;
                                if block_size == 0 {
                                    panic!("Read a block size of 0");
                                }

                                // The block is too big to fit
                                if block_size > idx + 1 {
                                    return ReductionResult::Insolvable;
                                }

                                // States in this range have to be blocks
                                for i in (idx - (block_size - 1))..idx {
                                    // Contradiction!
                                    if reduced_problem.states[i] == State::Okay {
                                        return ReductionResult::Insolvable;
                                    }
                                }

                                // We ran out of input before we can check that the next thing is '.'
                                if idx + 1 == block_size {
                                    reduced_problem.states.clear();
                                    return ReductionResult::ReducedProblem(reduced_problem);
                                }
                                // We didn't run out of room, so check that the next thing can legally be '.'
                                else {
                                    if reduced_problem.states[idx - block_size] == State::Broken {
                                        return ReductionResult::Insolvable;
                                    }
                                    reduced_problem.states.truncate(idx - block_size);
                                    return ReductionResult::ReducedProblem(reduced_problem);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
