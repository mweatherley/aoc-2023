use hashbrown::HashMap;
use std::cmp::max;
use std::collections::BTreeSet;

fn main() {
    println!("Let's solve AOC-16!");
    let now = std::time::Instant::now();
    let input = std::fs::read_to_string("input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

/* --------------- */
/* Data Structures */
/* --------------- */

type Coord = (isize, isize);

#[derive(Debug, Clone)]
struct SplitterMap {
    width: isize,
    height: isize,
    map: HashMap<Coord, Element>,
}

#[derive(Debug, Clone, Copy)]
enum Element {
    Splitter(Splitter),
    Mirror(Mirror),
}

#[derive(Debug, Clone, Copy)]
enum Splitter {
    NorthSouth,
    EastWest,
}

#[derive(Debug, Clone, Copy)]
enum Mirror {
    NorthWest,
    NorthEast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Direction {
    N,
    S,
    E,
    W,
}

const EACH_DIRECTION: [Direction; 4] = [Direction::N, Direction::S, Direction::E, Direction::W];

type BeamCache = HashMap<Coord, BTreeSet<Direction>>;

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> SplitterMap {
    // Things we build up:
    let mut map: HashMap<Coord, Element> = HashMap::new();

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
            '|' => {
                map.insert((cursor, line), Element::Splitter(Splitter::NorthSouth));
                cursor += 1;
            }
            '-' => {
                map.insert((cursor, line), Element::Splitter(Splitter::EastWest));
                cursor += 1;
            }
            '\\' => {
                map.insert((cursor, line), Element::Mirror(Mirror::NorthWest));
                cursor += 1;
            }
            '/' => {
                map.insert((cursor, line), Element::Mirror(Mirror::NorthEast));
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
    return SplitterMap {
        width: width.expect("Failed to find width"),
        height: line,
        map: map,
    };
}

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> usize {
    let splitter_map = problem_input(input);
    let north_max = (0..splitter_map.width)
        .map(|idx| beam_total(&splitter_map, Direction::N, idx))
        .reduce(max)
        .unwrap();
    let east_max = (0..splitter_map.height)
        .map(|idx| beam_total(&splitter_map, Direction::E, idx))
        .reduce(max)
        .unwrap();
    let south_max = (0..splitter_map.width)
        .map(|idx| beam_total(&splitter_map, Direction::S, idx))
        .reduce(max)
        .unwrap();
    let west_max = (0..splitter_map.height)
        .map(|idx| beam_total(&splitter_map, Direction::W, idx))
        .reduce(max)
        .unwrap();
    *vec![north_max, south_max, east_max, west_max]
        .iter()
        .reduce(max)
        .unwrap()
}

fn beam_total(splitter_map: &SplitterMap, direction: Direction, index: isize) -> usize {
    // To avoid issues with the first tile, we start "off-screen".
    // To avoid creating a bunch of code to avoid updating the
    // cache for the start location, I decided to just subtract 1
    // at the end.

    let start = match direction {
        Direction::E => (-1, index),
        Direction::W => (splitter_map.width, index),
        Direction::S => (index, -1),
        Direction::N => (index, splitter_map.height),
    };
    let mut cache: BeamCache = HashMap::default();
    run_beam(&mut cache, &splitter_map, start, direction, None);
    return cache.len() - 1;
}

fn run_beam(
    beam_cache: &mut BeamCache,
    splitter_map: &SplitterMap,
    start: Coord,
    start_direction: Direction,
    old_direction: Option<Direction>,
) {
    let mut old_dir = old_direction;
    let mut current_coords = start;
    let mut current_direction = start_direction;
    loop {
        // Update the cache with the direction where we just entered
        let mut update_dir = current_direction;
        if old_dir.is_some() {
            update_dir = old_dir.unwrap();
            old_dir = None;
        }
        update_cache(beam_cache, current_coords, update_dir);

        match coord_in_direction(splitter_map, current_coords, current_direction) {
            // Out of bounds => This beam dies
            None => {
                return;
            }

            // In bounds => Examine the next tile
            Some(next_coords) => {
                // If the cache indicates we've already done the next tile,
                // then this beam just dies
                if let Some(taken_dirs) = beam_cache.get(&next_coords) {
                    if taken_dirs.contains(&current_direction) {
                        return;
                    }
                }

                // Otherwise, actually look at the type of the next tile
                match splitter_map.map.get(&next_coords) {
                    // Empty tile, so we just update our position
                    None => {
                        current_coords = next_coords;
                        continue;
                    }

                    // A tile with an element in it
                    Some(el) => {
                        match el {
                            // Mirror => Change direction and update position
                            Element::Mirror(mirror) => {
                                // Old direction temporarily maintained for next write
                                old_dir = Some(current_direction);

                                current_coords = next_coords;
                                current_direction = mirror_direction(*mirror, current_direction);
                                continue;
                            }

                            // Splitter => Depends on whether it actually splits the beam
                            Element::Splitter(splitter) => {
                                // The splitter actually split the beam, so we run the two subprocesses
                                // corresponding to the split directions and then die
                                if let Some(new_directions) =
                                    splitter_directions(*splitter, current_direction)
                                {
                                    inundate(beam_cache, next_coords);
                                    for dir in new_directions {
                                        run_beam(
                                            beam_cache,
                                            splitter_map,
                                            next_coords,
                                            dir,
                                            Some(current_direction),
                                        );
                                    }
                                    return;
                                }
                                // Otherwise, the splitter is parallel to the direction of travel,
                                // so we just keep going
                                else {
                                    current_coords = next_coords;
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Returns None if the splitter is parallel to the direction of travel;
// otherwise returns the list of new directions for the beam out of the splitter.
fn splitter_directions(splitter: Splitter, direction: Direction) -> Option<Vec<Direction>> {
    match splitter {
        Splitter::EastWest => match direction {
            Direction::N | Direction::S => Some(vec![Direction::E, Direction::W]),
            _ => None,
        },
        Splitter::NorthSouth => match direction {
            Direction::E | Direction::W => Some(vec![Direction::N, Direction::S]),
            _ => None,
        },
    }
}

fn mirror_direction(mirror: Mirror, direction: Direction) -> Direction {
    match mirror {
        Mirror::NorthEast => match direction {
            Direction::N => Direction::E,
            Direction::S => Direction::W,
            Direction::E => Direction::N,
            Direction::W => Direction::S,
        },
        Mirror::NorthWest => match direction {
            Direction::N => Direction::W,
            Direction::S => Direction::E,
            Direction::E => Direction::S,
            Direction::W => Direction::N,
        },
    }
}

// Finds the next coordinate in a direction, returning None if that coordinate is out of bounds
fn coord_in_direction(map: &SplitterMap, start: Coord, direction: Direction) -> Option<Coord> {
    let (x, y) = start;
    let prospective_coord = match direction {
        Direction::E => (x + 1, y),
        Direction::W => (x - 1, y),
        Direction::N => (x, y - 1),
        Direction::S => (x, y + 1),
    };
    maybe_coord(map, prospective_coord)
}

fn maybe_coord(map: &SplitterMap, coord: Coord) -> Option<Coord> {
    match coord_in_bounds(map, coord) {
        true => Some(coord),
        false => None,
    }
}

fn coord_in_bounds(map: &SplitterMap, coord: Coord) -> bool {
    let (x, y) = coord;
    (0..map.width).contains(&x) && (0..map.height).contains(&y)
}

fn update_cache(cache: &mut BeamCache, coord: Coord, incoming_dir: Direction) {
    cache
        .entry(coord)
        .and_modify(|set| {
            set.insert(incoming_dir);
        })
        .or_insert_with(|| {
            let mut set = BTreeSet::new();
            set.insert(incoming_dir);
            set
        });
}

// Fill the coordinate with all directions, effectively killing it for beam paths
fn inundate(cache: &mut BeamCache, coord: Coord) {
    cache
        .entry(coord)
        .and_modify(|set| {
            for dir in EACH_DIRECTION {
                set.insert(dir);
            }
        })
        .or_insert_with(|| {
            let mut set = BTreeSet::new();
            for dir in EACH_DIRECTION {
                set.insert(dir);
            }
            set
        });
}

/* --------- */
/* Debugging */
/* --------- */

fn grid_string(cache: &BeamCache, splitter_map: &SplitterMap) -> String {
    let mut grid_string = String::new();
    for y in 0..splitter_map.height {
        for x in 0..splitter_map.width {
            grid_string.push(coord_to_char(cache, splitter_map, (x, y)));
        }
        grid_string.push('\n');
    }
    grid_string
}

fn fill_string(cache: &BeamCache, splitter_map: &SplitterMap) -> String {
    let mut fill_string = String::new();
    for y in 0..splitter_map.height {
        for x in 0..splitter_map.width {
            if cache.contains_key(&(x, y)) {
                fill_string.push('#');
            } else {
                fill_string.push('.');
            }
        }
        fill_string.push('\n');
    }
    fill_string
}

fn coord_to_char(cache: &BeamCache, splitter_map: &SplitterMap, coord: Coord) -> char {
    if splitter_map.map.contains_key(&coord) {
        match splitter_map.map.get(&coord).unwrap() {
            Element::Mirror(Mirror::NorthEast) => '/',
            Element::Mirror(Mirror::NorthWest) => '\\',
            Element::Splitter(Splitter::EastWest) => '-',
            Element::Splitter(Splitter::NorthSouth) => '|',
        }
    } else if cache.contains_key(&coord) {
        let dirs = cache.get(&coord).unwrap();
        if dirs.len() > 1 {
            match dirs.len() {
                2 => '2',
                3 => '3',
                4 => '4',
                _ => unreachable!(),
            }
        } else {
            match dirs.first().unwrap() {
                Direction::N => '^',
                Direction::E => '>',
                Direction::W => '<',
                Direction::S => 'v',
            }
        }
    } else {
        '.'
    }
}
