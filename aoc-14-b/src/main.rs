use std::{
    collections::{BTreeMap, HashMap},
    fs,
};

fn main() {
    println!("Let's solve AOC-14!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("aoc-14-input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

const MAX_ITERATIONS: isize = 1000000000;

/* --------------- */
/* Data Structures */
/* --------------- */
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Rock {
    Round,
    Square,
}

// Now, we doubly record the map data; BTreeMap uses lexicographic sort order,
// so one of these will be sorted column-first and the other will be sorted
// row-first. We can update them in tandem, but this makes it easier to do the
// north/south/east/west transformations because it makes it trivial to iterate
// along a single row or column.
#[derive(Clone, PartialEq, Eq, Hash)]
struct RockMap {
    width: isize,
    height: isize,
    map_by_cols: BTreeMap<(isize, isize), Rock>,
    map_by_rows: BTreeMap<(isize, isize), Rock>,
}

impl RockMap {
    // Removes a rock at `old_coords` and places one at `new_coords`
    // `transposed` dictates whether the inputs to this are in transposed coordinates
    fn alter(&mut self, transposed: bool, old_coords: (isize, isize), new_coords: (isize, isize)) {
        let (mut old_x, mut old_y) = old_coords;
        let (mut new_x, mut new_y) = new_coords;
        if transposed {
            (old_x, old_y) = (old_y, old_x);
            (new_x, new_y) = (new_y, new_x);
        }
        self.map_by_cols.remove(&(old_x, old_y));
        self.map_by_rows.remove(&(old_y, old_x));
        self.map_by_cols.insert((new_x, new_y), Rock::Round);
        self.map_by_rows.insert((new_y, new_x), Rock::Round);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Direction {
    N,
    S,
    E,
    W,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Directionality {
    Forward,
    Backward,
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> RockMap {
    // Things we build up:
    let mut map_cols: BTreeMap<(isize, isize), Rock> = BTreeMap::new();
    let mut map_rows: BTreeMap<(isize, isize), Rock> = BTreeMap::new();

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
                map_cols.insert((cursor, line), Rock::Round);
                map_rows.insert((line, cursor), Rock::Round);
                cursor += 1;
            }
            '#' => {
                map_cols.insert((cursor, line), Rock::Square);
                map_rows.insert((line, cursor), Rock::Square);
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
        map_by_cols: map_cols,
        map_by_rows: map_rows,
    };
}

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> isize {
    let mut rock_map = problem_input(input);
    let mut iterations = 0;
    let mut visited: HashMap<RockMap, isize> = HashMap::new();
    let mut additional_iterations: Option<isize> = None;

    // We just start looping our guy and recording all the previous states in a HashMap
    // together with the iteration where they occurred
    loop {
        cycle(&mut rock_map);
        iterations += 1;

        // If we visited this state before, we compute the length of the cycle; then
        // we break out so we can iterate a little more until we agree with `MAX_ITERATIONS`
        // modulo the cycle length of the dynamics.
        if visited.contains_key(&rock_map) {
            let last_visited = visited.get(&rock_map).unwrap();
            let length = iterations - last_visited;
            println!(
                "Cycle length: {}, observed at iteration {}",
                length, iterations
            );
            additional_iterations = Some((MAX_ITERATIONS - iterations) % length);
            break;
        } else {
            visited.insert(rock_map.clone(), iterations);
        }

        // I put this here just so that this theoretically always terminates, even if
        // the runtime would be completely heinous
        if iterations == MAX_ITERATIONS {
            break;
        }
    }

    // We have some remaining iterations to perform to reach the right state in the cycle
    if let Some(mut remaining_iterations) = additional_iterations {
        while remaining_iterations > 0 {
            cycle(&mut rock_map);
            remaining_iterations -= 1;
        }
    }
    return total_load(&rock_map);
}

// All of the shift operations have the same idea, so I tried to reuse code, but it only sort of worked,
// since the iterator's type changes depending on whether you reverse it, which I hadn't accounted for
// at the outset.
fn shift(rock_map: &mut RockMap, direction: Direction) {
    let (map_to_use, directionality, transposed, boundary, orthog_max) = match direction {
        Direction::N => (
            rock_map.map_by_cols.clone(),
            Directionality::Backward,
            false,
            rock_map.height,
            rock_map.width,
        ),
        Direction::S => (
            rock_map.map_by_cols.clone(),
            Directionality::Forward,
            false,
            rock_map.height,
            rock_map.width,
        ),
        Direction::E => (
            rock_map.map_by_rows.clone(),
            Directionality::Forward,
            true,
            rock_map.width,
            rock_map.height,
        ),
        Direction::W => (
            rock_map.map_by_rows.clone(),
            Directionality::Backward,
            true,
            rock_map.width,
            rock_map.height,
        ),
    };

    // Outer iterator moves over lines in the direction orthogonal to the direction where rocks roll
    for line in 0..orthog_max {
        let iterator = map_to_use.range((line, 0)..(line + 1, 0));
        match directionality {
            Directionality::Backward => {
                // `resting_place` records the position of where the next Round rock we encounter
                // should roll.
                let mut resting_place = 0;
                for (coords, rock) in iterator {
                    match rock {
                        // When we encounter a round rock, we move it to the `resting_place`;
                        // it takes up space, so the `resting_place` moves by one position
                        Rock::Round => {
                            rock_map.alter(transposed, *coords, (line, resting_place));
                            resting_place += 1;
                        }

                        // When we encounter a Square rock, we update our `resting_place` so
                        // that it is next to it.
                        Rock::Square => {
                            let (_ln, pos) = coords;
                            resting_place = pos + 1;
                        }
                    }
                }
            }
            Directionality::Forward => {
                let mut resting_place = boundary - 1;
                for (coords, rock) in iterator.rev() {
                    match rock {
                        Rock::Round => {
                            rock_map.alter(transposed, *coords, (line, resting_place));
                            resting_place -= 1;
                        }
                        Rock::Square => {
                            let (_ln, pos) = coords;
                            resting_place = pos - 1;
                        }
                    }
                }
            }
        }
    }
}

// Run one cycle
fn cycle(rock_map: &mut RockMap) {
    shift(rock_map, Direction::N);
    shift(rock_map, Direction::W);
    shift(rock_map, Direction::S);
    shift(rock_map, Direction::E);
}

// Instead of pre-accounting for a transformation, we just operate on the
// transformed version of the map, so this part becomes boring and trivial
fn column_sum(column: isize, rock_map: &RockMap) -> isize {
    let this_column = (column, 0)..(column + 1, 0);
    let height: isize = rock_map.height;
    let mut total = 0;
    for ((_, pos), rock) in rock_map.map_by_cols.range(this_column) {
        if *rock == Rock::Round {
            total += height - *pos;
        }
    }
    return total;
}

fn total_load(rock_map: &RockMap) -> isize {
    (0..rock_map.width)
        .map(|col| column_sum(col, rock_map))
        .sum()
}
