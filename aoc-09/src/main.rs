use nom::{
    character::complete::{i64, newline, space0},
    multi::many1,
    sequence::{delimited, terminated},
    IResult,
};
use std::fs;

fn main() {
    println!("Let's solve AOC-09!");
    let input = fs::read_to_string("aoc-09-input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Solution: {}", solution);
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> IResult<&str, Vec<Vec<i64>>> {
    many1(sequence)(input)
}

fn sequence(input: &str) -> IResult<&str, Vec<i64>> {
    terminated(many1(padded_i64), newline)(input)
}

fn padded_i64(input: &str) -> IResult<&str, i64> {
    delimited(space0, i64, space0)(input)
}

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> i64 {
    let (_, seqs) = problem_input(input).expect("Failed to parse problem input");
    let predictions: Vec<i64> = seqs.iter().map(predict).collect();
    let total = predictions.iter().fold(0, |x, y| x + y);
    return total;
}

fn predict(seq: &Vec<i64>) -> i64 {
    // Start by generating the list of 'derivative' sequences
    let start_seq = seq.clone();
    let mut last_seq: Vec<i64> = start_seq.clone();
    let mut diff_seqs: Vec<Vec<i64>> = vec![start_seq];
    loop {
        let next_diff = differences(&last_seq);
        last_seq = next_diff.clone();
        diff_seqs.push(next_diff);
        if is_all_zero(&last_seq) || last_seq.is_empty() {
            break;
        }
    }

    // Now, perform prediction for each of them and percolate up to the top
    let mut last_val = 0;
    loop {
        let mut last_diffs = diff_seqs.pop().unwrap();
        last_val = last_diffs.pop().unwrap() + last_val;
        if diff_seqs.is_empty() {
            break;
        }
    }

    return last_val;
}

fn differences(seq: &Vec<i64>) -> Vec<i64> {
    let mut differences: Vec<i64> = vec![];
    let mut last_val = None;
    for val in seq.iter() {
        if let Some(last) = last_val {
            differences.push(*val - last);
            last_val = Some(*val);
        } else {
            last_val = Some(*val);
        }
    }
    return differences;
}

fn is_all_zero(seq: &Vec<i64>) -> bool {
    seq.iter().all(|val| *val == 0)
}
