use std::{collections::BTreeMap, fs};

fn main() {
    println!("Let's solve AOC-14!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("aoc-14-input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

/* --------------- */
/* Data Structures */
/* --------------- */
#[derive(Clone, Copy, Debug)]
enum Rock {
    Round,
    Square,
}

struct RockMap {
    width: isize,
    height: isize,
    map: BTreeMap<(isize, isize), Rock>,
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> RockMap {
    // Things we build up:
    let mut map: BTreeMap<(isize, isize), Rock> = BTreeMap::new();

    // Parser state:
    let mut cursor = 0;
    let mut line = 0;
    let mut width: Option<isize> = None;
    for c in input.chars() {
        match c {
            '\n' => {
                line += 1;
                if width.is_none() {
                    width = Some(cursor);
                }
                cursor = 0;
            }
            'O' => {
                map.insert((cursor, line), Rock::Round);
                cursor += 1;
            }
            '#' => {
                map.insert((cursor, line), Rock::Square);
                cursor += 1;
            }
            '.' => {
                cursor += 1;
            }
            _ => {
                panic!("Illegal character parsed");
            }
        }
    }
    return RockMap {
        width: width.expect("Failed to find width"),
        height: line,
        map: map,
    };
}

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> isize {
    let rock_map = problem_input(input);
    let mut total = 0;
    for col in 0..rock_map.width {
        let col_sum = column_sum(col, &rock_map);
        println!("Sum for column {}: {}", col, col_sum);
        total += col_sum;
    }
    return total;
}

fn column_sum(column: isize, rock_map: &RockMap) -> isize {
    let this_column = (column, 0)..(column + 1, 0);
    let height: isize = rock_map.height;
    let mut current_blockage = -1;
    let mut current_rounds = 0;
    let mut total = 0;
    for ((_, pos), rock) in rock_map.map.range(this_column) {
        match rock {
            Rock::Round => {
                // Increment the rounds
                current_rounds += 1;
            }
            Rock::Square => {
                // Update the total and change the blockage
                let weight = weight_contribution(height - (current_blockage + 1), current_rounds);
                total += weight;
                current_blockage = *pos;
                current_rounds = 0;
            }
        }
    }
    let leftover_weight = weight_contribution(height - (current_blockage + 1), current_rounds);
    total += leftover_weight;
    return total;
}

fn weight_contribution(max_height: isize, num_rounds: isize) -> isize {
    let min_height = max_height - num_rounds + 1;
    return ((max_height + min_height) * num_rounds) / 2;
}
