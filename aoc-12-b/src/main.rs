use std::collections::BTreeSet;
use std::fs;

use nalgebra::{DMatrix, DVector, OMatrix};
use nom::branch::alt;
use nom::character::complete::{char, newline, u32};
use nom::combinator::value;
use nom::combinator::{map, opt};
use nom::multi::{many0, many1};
use nom::sequence::separated_pair;
use nom::{sequence::terminated, IResult};

fn main() {
    println!("Let's solve AOC-12!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("aoc-12-input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Finished in {:?}", now.elapsed());
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

// A `BlockPositions` is, for each block, a set of potential starting indices for that block
type LocationSet = BTreeSet<usize>;
type BlockPositions = Vec<(usize, LocationSet)>;

type MatrixEntry = i128;

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

fn solve_problem(input: &str) -> MatrixEntry {
    let (_, problems) = problem_input(input).expect("Failed to parse problem input");
    let mut total = 0;
    for mut problem in problems.into_iter() {
        expand_problem(&mut problem);
        total += total_solutions(&problem);
    }
    return total;
}

fn expand_problem(problem: &mut SpringProblem) {
    let states = problem.states.clone();
    let blocks = problem.blocks.clone();
    for _i in 0..4 {
        let mut states_copy = states.clone();
        problem.states.push(State::Unknown);
        problem.states.append(&mut states_copy);

        let mut blocks_copy = blocks.clone();
        problem.blocks.append(&mut blocks_copy);
    }
}

// This generate a 'crude' location sets that only take into account:
// > theoretical bounds imposed by block sizes
// > pruning from the state set; i.e. not trying to map blocks where they couldn't ever be in isolation
// > pruning from leftover '#' tokens near the edges
fn generate_location_sets(spring_problem: &SpringProblem) -> BlockPositions {
    let blocks = &spring_problem.blocks;
    let states = &spring_problem.states;
    let states_length = states.len();

    // Generate sums used in bounds for pruning:
    let mut sum: usize = 0;
    let mut partial_sums: Vec<usize> = vec![];
    let mut partial_antisums: Vec<usize> = vec![];
    for size in blocks.iter() {
        let size = *size as usize;
        partial_sums.push(sum);
        sum += size + 1;
    }
    for size in blocks.iter() {
        let size = *size as usize;
        partial_antisums.push(sum);
        sum -= size + 1;
    }

    // This is just based on theoretical bounds imposed by the block lengths
    let mut initial_location_sets: BlockPositions = vec![];
    for (idx, size) in blocks.iter().enumerate() {
        let mut legal_positions: LocationSet = BTreeSet::new();
        for pos in partial_sums[idx]..=(states_length + 1 - partial_antisums[idx]) {
            legal_positions.insert(pos);
        }
        initial_location_sets.push((*size as usize, legal_positions));
    }

    // Now, we prune to only the locations that are actually possible in isolation based on states
    for (size, legal_positions) in initial_location_sets.iter_mut() {
        let mut bad_positions: Vec<usize> = vec![];
        'position: for pos in legal_positions.iter() {
            // If the block overlaps a '.' this is a bad position
            for idx in *pos..(*pos + *size) {
                if states[idx] == State::Okay {
                    bad_positions.push(*pos);
                    continue 'position;
                }
            }

            // If an end is '#' then this is a bad position
            if (!(*pos == 0)) && states[*pos - 1] == State::Broken {
                bad_positions.push(*pos);
                continue 'position;
            }
            if (!(*pos == states_length - *size)) && states[*pos + *size] == State::Broken {
                bad_positions.push(*pos);
                continue 'position;
            }
        }
        for pos in bad_positions.iter() {
            legal_positions.remove(pos);
        }
    }

    // Do pruning for edge effects involving leftover '#'
    if !initial_location_sets.is_empty() {
        if let Some((first_idx, last_idx)) = first_and_last_broken(spring_problem) {
            // At the beginning:
            let (_, first_locs) = initial_location_sets.first_mut().unwrap();
            first_locs.retain(|pos| *pos <= first_idx);

            // At the end:
            let (last_size, last_locs) = initial_location_sets.last_mut().unwrap();
            last_locs.retain(|pos| *pos + *last_size >= last_idx);
        }
    }

    return initial_location_sets;
}

// This prunes the block positions based on their current understanding of bounds on
// where their adjacent nodes can be. Note that this is not idempotent.
fn neighbor_prune(block_positions: &mut BlockPositions) {
    let mut thresholds = vec![];
    for ((first_size, first_locs), (_second_size, second_locs)) in
        block_positions.iter().zip(block_positions.iter().skip(1))
    {
        let low_val = first_locs.first().unwrap() + first_size + 1;
        let high_val = second_locs.last().unwrap() - first_size;
        thresholds.push((low_val, high_val));
    }

    let length = block_positions.len();
    for (idx, (_, locs)) in block_positions.iter_mut().enumerate() {
        if idx > 0 {
            let (lb, _) = thresholds[idx - 1];
            locs.retain(|pos| *pos >= lb);
        }
        if idx < length - 1 {
            let (_, ub) = thresholds[idx];
            locs.retain(|pos| *pos <= ub);
        }
    }
}

// This produces the casuality matrices for each adjacent pair in `block_positions`
fn causality_matrices(
    problem: &SpringProblem,
    block_positions: &BlockPositions,
) -> Vec<DMatrix<MatrixEntry>> {
    let mut matrices: Vec<DMatrix<MatrixEntry>> = vec![];
    for (first, second) in block_positions.iter().zip(block_positions.iter().skip(1)) {
        matrices.push(allowance_matrix(problem, first, second));
    }
    return matrices;
}

// Local causality is determined by the following:
// for each block b_i, the block b_i+1 must start:
// > after b_i ends (plus one index)
// > before the next '#' after b_i
// This gives a bunch of matrices, which can be multiplied to give the answer
fn allowance_matrix(
    problem: &SpringProblem,
    first: &(usize, LocationSet),
    second: &(usize, LocationSet),
) -> DMatrix<MatrixEntry> {
    let (first_size, first_locs) = first;
    let (_second_size, second_locs) = second;

    if second_locs.is_empty() || first_locs.is_empty() {
        panic!("Failed to compute allowance matrix: one or more inputs with no legal locations");
    }

    // We build a matrix where each column corresponds to a position in `first_locs`;
    // each row corresponds to a position in `second_locs`. An entry is a 0 if the second
    // position is disallowed by the first.

    let mut matrix_columns: Vec<DVector<MatrixEntry>> = vec![];

    let max_second = *second_locs.last().unwrap();
    for first_pos in first_locs.iter() {
        let lb = first_pos + first_size + 1;
        let mut column: Vec<MatrixEntry> = vec![];
        for second_pos in second_locs.iter() {
            // The position of the second block is too far to the left; the blocks are stepping
            // on each others' toes.
            if *second_pos < lb {
                column.push(0);
                continue;
            }
            // The position of the second block is too far to the right, exposing a '#' which
            // is not accounted for by a block position.
            if let Some(broken_pos) = first_broken(problem, lb, max_second) {
                if broken_pos < *second_pos {
                    column.push(0);
                    continue;
                }
            }
            column.push(1);
            continue;
        }
        let column: DVector<MatrixEntry> = DVector::from_vec(column);
        matrix_columns.push(column);
    }
    return OMatrix::from_columns(matrix_columns.as_slice());
}

// Search in range start_idx..end_idx for the first broken symbol in the problem
fn first_broken(problem: &SpringProblem, start_idx: usize, end_idx: usize) -> Option<usize> {
    let states = &problem.states[start_idx..end_idx];
    for (idx, state) in states.iter().enumerate() {
        if *state == State::Broken {
            return Some(idx + start_idx);
        }
    }
    return None;
}

// Finding the first and last broken springs for edge effects
fn first_and_last_broken(problem: &SpringProblem) -> Option<(usize, usize)> {
    let mut first_broken: Option<usize> = None;
    let mut last_broken: Option<usize> = None;
    for idx in 0..problem.states.len() {
        if problem.states[idx] == State::Broken {
            if first_broken.is_none() {
                first_broken = Some(idx);
            }
            last_broken = Some(idx);
        }
    }
    match (first_broken, last_broken) {
        (Some(idx1), Some(idx2)) => Some((idx1, idx2)),
        _ => None,
    }
}

fn total_solutions(spring_problem: &SpringProblem) -> MatrixEntry {
    let mut block_positions = generate_location_sets(spring_problem);
    neighbor_prune(&mut block_positions);
    let matrices = causality_matrices(spring_problem, &block_positions);
    let prod = matrices.into_iter().reduce(|x, y| y * x).unwrap();
    return prod.sum();

    // Morally speaking, the reason this sum works is that the edges of the container
    // also impose constraints, which would contribute a single column and row vector of 1s.
    // Actually, the only reason these consist of 1s is that the edge effects are already absorbed
    // into the initial mappings produced by `generate_location_sets`; otherwise, they would have 0s
    // where the adjacent block locations become illegal because of leftover '#' symbols.

    // Of course, you can also just think of it this way: each point of mass in the matrix
    // corresponds to a legal assignment of all the block locations, so the total mass is the
    // total number of legal assignments; but this representation privileges the first and last
    // block locations for no good reason.
}
