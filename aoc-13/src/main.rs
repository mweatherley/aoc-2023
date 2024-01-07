use std::{
    collections::{BTreeSet, HashMap},
    fs,
};

fn main() {
    println!("Let's solve AOC-13!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("aoc-13-input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

/* --------------- */
/* Data Structures */
/* --------------- */
#[derive(Debug, Clone)]
struct RockMap {
    width: usize,
    height: usize,
    map: HashMap<(usize, usize), Element>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Element {
    Ash,
    Rock,
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> Vec<RockMap> {
    // Things we build up:
    let mut rock_maps: Vec<RockMap> = vec![];
    let mut current_map: HashMap<(usize, usize), Element> = HashMap::new();

    // Parser state:
    let mut cursor = 0;
    let mut line = 0;
    let mut width: Option<usize> = None;
    for c in input.chars() {
        match c {
            '\n' => {
                if cursor == 0 {
                    let rock_map = RockMap {
                        width: width.expect("Failed to find width"),
                        height: line,
                        map: current_map,
                    };
                    rock_maps.push(rock_map);
                    current_map = HashMap::new();
                    line = 0;
                    cursor = 0;
                    width = None;
                } else {
                    line += 1;
                    if width.is_none() {
                        width = Some(cursor);
                    }
                    cursor = 0;
                }
            }
            '.' => {
                current_map.insert((cursor, line), Element::Ash);
                cursor += 1;
            }
            '#' => {
                current_map.insert((cursor, line), Element::Rock);
                cursor += 1;
            }
            _ => {
                panic!("Illegal character parsed")
            }
        }
    }
    if !current_map.is_empty() {
        let rock_map = RockMap {
            width: width.expect("Failed to find width"),
            height: line,
            map: current_map,
        };
        rock_maps.push(rock_map);
    }
    return rock_maps;
}

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> usize {
    let rock_maps = problem_input(input);
    let mut total = 0;
    for rock_map in rock_maps.iter() {
        if let Some(pos) = find_vertical_reflection(rock_map) {
            total += pos;
        } else if let Some(pos) = find_horizontal_reflection(rock_map) {
            total += pos * 100;
        }
    }
    return total;
}

fn find_vertical_reflection(rock_map: &RockMap) -> Option<usize> {
    let mut remaining_slots: BTreeSet<usize> = (1..rock_map.width).collect();
    let mut row = 0;
    loop {
        remaining_slots.retain(|r| could_be_vert_reflection(*r, row, rock_map));
        if remaining_slots.len() == 0 {
            return None;
        }
        row += 1; // Now `row` == the number of rows processed
        if row == rock_map.height {
            break;
        }
    }
    match remaining_slots.len() {
        0 => {
            return None;
        }
        1 => {
            return Some(*remaining_slots.first().unwrap());
        }
        _ => panic!("More than one possible reflection"),
    }
}

fn could_be_vert_reflection(position: usize, row: usize, rock_map: &RockMap) -> bool {
    for (inc, idx) in (0..position).rev().enumerate() {
        let left_guy = rock_map.map.get(&(idx, row)).unwrap(); // Guaranteed to exist
        if position + inc < rock_map.width {
            let right_guy = rock_map.map.get(&(position + inc, row)).unwrap(); // We are in bounds
            if !(*left_guy == *right_guy) {
                return false;
            } else {
                continue;
            }
        }
        // We made it to the end
        else {
            return true;
        }
    }
    return true;
}

fn find_horizontal_reflection(rock_map: &RockMap) -> Option<usize> {
    let mut remaining_slots: BTreeSet<usize> = (1..rock_map.height).collect();
    let mut column = 0;
    loop {
        remaining_slots.retain(|r| could_be_horz_reflection(*r, column, rock_map));
        if remaining_slots.len() == 0 {
            return None;
        }
        column += 1; // Now `row` == the number of rows processed
        if column == rock_map.width {
            break;
        }
    }
    match remaining_slots.len() {
        0 => {
            return None;
        }
        1 => {
            return Some(*remaining_slots.first().unwrap());
        }
        _ => panic!("More than one possible reflection"),
    }
}

fn could_be_horz_reflection(position: usize, column: usize, rock_map: &RockMap) -> bool {
    for (inc, idx) in (0..position).rev().enumerate() {
        let top_guy = rock_map.map.get(&(column, idx)).unwrap(); // Guaranteed to exist
        if position + inc < rock_map.height {
            let bottom_guy = rock_map.map.get(&(column, position + inc)).unwrap(); // We are in bounds
            if !(*top_guy == *bottom_guy) {
                return false;
            } else {
                continue;
            }
        }
        // We made it to the end
        else {
            return true;
        }
    }
    return true;
}
