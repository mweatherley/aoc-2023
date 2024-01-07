use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
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
        if let Some(pos) = find_secret_vert_line(rock_map) {
            total += pos;
        } else if let Some(pos) = find_secret_horz_line(rock_map) {
            total += pos * 100;
        } else {
            panic!("Didn't solve the problem :(")
        }
    }
    return total;
}

// Note: The actual input of the problem is such that multiple candidates never actually happens;
// it's clearly possible to produce examples where there are multiple candidates, but it is unclear
// to me whether such examples that still have valid solutions exist. Anyway, I added handling for
// these cases because I could not disprove their existence.
fn find_secret_vert_line(rock_map: &RockMap) -> Option<usize> {
    let mut vert_reflection_multiplicities: BTreeMap<usize, usize> = BTreeMap::new();
    let mut near_reflection_index: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
    for row in 0..rock_map.height {
        let (reflections_for_row, near_reflections_for_row) =
            vert_reflections_in_row(row, rock_map);
        for pos in near_reflections_for_row {
            near_reflection_index
                .entry(pos)
                .and_modify(|set| {
                    set.insert(row);
                })
                .or_insert([row].into());
        }
        for refl in reflections_for_row.iter() {
            vert_reflection_multiplicities
                .entry(*refl)
                .and_modify(|v| *v += 1)
                .or_insert(1);
        }
    }
    let mut candidates: Vec<usize> = vec![];
    for (pos, mult) in vert_reflection_multiplicities.iter() {
        if *mult == rock_map.height - 1 {
            candidates.push(*pos);
        }
    }
    if candidates.len() == 1 {
        return Some(*candidates.first().unwrap());
    } else if candidates.len() == 0 {
        return None;
    }
    // We found more than one candidate, but at most one is actually near a row;
    // we locate it with the index. If it doesn't exist, then we didn't find anything
    else {
        for pos in candidates.iter() {
            match near_reflection_index.get(pos) {
                Some(set) => {
                    if !set.is_empty() {
                        return Some(*pos);
                    } else {
                        continue;
                    }
                }
                None => {
                    continue;
                }
            }
        }
        return None;
    }
}

fn find_secret_horz_line(rock_map: &RockMap) -> Option<usize> {
    let mut horz_reflection_multiplicities: BTreeMap<usize, usize> = BTreeMap::new();
    let mut near_reflection_index: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
    for column in 0..rock_map.width {
        let (reflections_for_column, near_reflections_for_column) =
            horz_reflections_in_column(column, rock_map);
        for pos in near_reflections_for_column {
            near_reflection_index
                .entry(pos)
                .and_modify(|set| {
                    set.insert(column);
                })
                .or_insert([column].into());
        }
        for refl in reflections_for_column.iter() {
            horz_reflection_multiplicities
                .entry(*refl)
                .and_modify(|v| *v += 1)
                .or_insert(1);
        }
    }
    let mut candidates: Vec<usize> = vec![];
    for (pos, mult) in horz_reflection_multiplicities.iter() {
        if *mult == rock_map.width - 1 {
            candidates.push(*pos);
        }
    }
    if candidates.len() == 1 {
        return Some(*candidates.first().unwrap());
    } else if candidates.len() == 0 {
        return None;
    } else {
        for pos in candidates.iter() {
            match near_reflection_index.get(pos) {
                Some(set) => {
                    if !set.is_empty() {
                        return Some(*pos);
                    } else {
                        continue;
                    }
                }
                None => {
                    continue;
                }
            }
        }
        return None;
    }
}

// These functions tell us how close each potential reflection is to being a reflection;
// i.e. how many symbols would have to change in order to make the position one of reflection
fn vert_reflection_dist(position: usize, row: usize, rock_map: &RockMap) -> usize {
    let mut distance = 0;
    for (inc, idx) in (0..position).rev().enumerate() {
        let left_guy = rock_map.map.get(&(idx, row)).unwrap(); // Guaranteed to exist
        if position + inc < rock_map.width {
            let right_guy = rock_map.map.get(&(position + inc, row)).unwrap(); // We are in bounds
            if !(*left_guy == *right_guy) {
                distance += 1;
                continue;
            } else {
                continue;
            }
        }
        // We made it to the end
        else {
            return distance;
        }
    }
    return distance;
}

fn horz_reflection_dist(position: usize, column: usize, rock_map: &RockMap) -> usize {
    let mut distance = 0;
    for (inc, idx) in (0..position).rev().enumerate() {
        let top_guy = rock_map.map.get(&(column, idx)).unwrap(); // Guaranteed to exist
        if position + inc < rock_map.height {
            let bottom_guy = rock_map.map.get(&(column, position + inc)).unwrap(); // We are in bounds
            if !(*top_guy == *bottom_guy) {
                distance += 1;
                continue;
            } else {
                continue;
            }
        }
        // We made it to the end
        else {
            return distance;
        }
    }
    return distance;
}

// These functions return a pair; the first element is the set of legal reflections, while the second
// is the set of near-reflections (i.e. those with distance 1).
fn vert_reflections_in_row(row: usize, rock_map: &RockMap) -> (BTreeSet<usize>, BTreeSet<usize>) {
    let mut near_reflections: BTreeSet<usize> = (1..rock_map.width).collect();
    let mut reflections: BTreeSet<usize> = (1..rock_map.width).collect();
    near_reflections.retain(|pos| vert_reflection_dist(*pos, row, rock_map) == 1);
    reflections.retain(|pos| vert_reflection_dist(*pos, row, rock_map) == 0);
    return (reflections, near_reflections);
}

fn horz_reflections_in_column(
    column: usize,
    rock_map: &RockMap,
) -> (BTreeSet<usize>, BTreeSet<usize>) {
    let mut near_reflections: BTreeSet<usize> = (1..rock_map.height).collect();
    let mut reflections: BTreeSet<usize> = (1..rock_map.height).collect();
    near_reflections.retain(|pos| horz_reflection_dist(*pos, column, rock_map) == 1);
    reflections.retain(|pos| horz_reflection_dist(*pos, column, rock_map) == 0);
    return (reflections, near_reflections);
}
