use std::fs;

fn main() {
    println!("Let's solve AOC-15!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("aoc-15-input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

/* ------- */
/* Parsers */
/* ------- */

fn problem(input: &str) -> Vec<u8> {
    input.split(',').map(|ins| hash(ins.as_bytes())).collect()
}

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> u64 {
    let hashes = problem(input);
    return hashes.iter().map(|x| *x as u64).sum();
}

fn hash(xs: &[u8]) -> u8 {
    let mut val: u8 = 0;
    for x in xs.iter() {
        if *x != b'\n' {
            val = val.wrapping_add(*x);
            val = val.wrapping_mul(17);
        }
    }
    return val;
}
