use hashbrown::HashMap;
use std::{collections::VecDeque, fs};

fn main() {
    println!("Let's solve AOC-21!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> RockMap {
    // Things we build up:
    let mut map: HashMap<Coord, Element> = HashMap::new();

    // Parser state:
    let mut cursor = 0;
    let mut line = 0;
    let mut width: Option<isize> = None;
    let mut start: Option<Coord> = None;
    for c in input.chars() {
        match c {
            '\n' => {
                line += 1;
                if width.is_none() {
                    width = Some(cursor);
                }
                cursor = 0;
            }
            '#' => {
                map.insert((cursor, line), Element::Rock);
                cursor += 1;
            }
            'S' => {
                start = Some((cursor, line));
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
        start: start.expect("Failed to find start"),
    };
}

/* --------------- */
/* Data Structures */
/* --------------- */

#[derive(Debug, Clone)]
struct RockMap {
    width: isize,
    height: isize,
    map: HashMap<Coord, Element>,
    start: Coord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Element {
    Rock,
}

#[derive(Debug, Clone, Copy)]
struct NodeData {
    time_reached: isize,
    color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    White,
    Black,
}

type Coord = (isize, isize);
type NodeDataMap = HashMap<Coord, NodeData>;

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> isize {
    let rock_map = problem_input(input);

    // Basic idea: Let's just do a BFS and find all tiles reachable
    // within 64 steps. The ones that can be reached in exactly 64
    // steps are just the ones with the same parity as the start.

    let mut tile_queue: VecDeque<(Coord, isize)> = VecDeque::default();
    let mut node_data: NodeDataMap = HashMap::default();

    // One invariant is that we never enqueue a tile that we have
    // already enqueued before. We also need a "time" parameter so
    // that we can store the distance at which a node was reached.

    tile_queue.push_back((rock_map.start, 0));
    node_data.insert(
        rock_map.start,
        NodeData {
            time_reached: 0,
            color: Color::White,
        },
    );

    // We will track the total number of white tiles (those with the
    // same coordinate-parity as the start) so that we don't have to
    // compute this again later.
    let mut white_tiles = 1;

    while !tile_queue.is_empty() {
        let (this_tile, tile_time) = tile_queue.pop_front().unwrap();

        if tile_time == 64 {
            break;
        }

        // Get the adjacent tiles that we have never recorded data about
        let new_tiles: Vec<_> = adjacent_tiles(&rock_map, this_tile)
            .into_iter()
            .filter(|tile| !node_data.contains_key(tile))
            .collect();

        // Record the data about these new tiles, updating our count of
        // white tiles along the way
        for adj_tile in new_tiles.iter() {
            let adj_color = match same_color(*adj_tile, rock_map.start) {
                true => Color::White,
                false => Color::Black,
            };
            node_data.insert(
                *adj_tile,
                NodeData {
                    time_reached: tile_time,
                    color: adj_color,
                },
            );
            if adj_color == Color::White {
                white_tiles += 1;
            }
        }

        // The queue contains tiles whose adjacent nodes are unexplored
        tile_queue.extend(
            new_tiles
                .into_iter()
                .map(|tile| (tile, tile_time + 1))
                .collect::<Vec<_>>(),
        );
    }

    return white_tiles;
}

// If these were two coordinates on a chessboard, would they have
// the same color?
fn same_color(first: Coord, second: Coord) -> bool {
    let (x1, y1) = first;
    let (x2, y2) = second;
    ((x2 - x1) + (y2 - y1)) % 2 == 0
}

// This function filters out adjacent tiles that are out of bounds
// or where there is a rock.
fn adjacent_tiles(rock_map: &RockMap, coord: Coord) -> Vec<Coord> {
    let (x, y) = coord;
    [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
        .into_iter()
        .filter(|c| in_bounds(rock_map, *c) && !rock_map.map.contains_key(c))
        .collect()
}

fn in_bounds(rock_map: &RockMap, coord: Coord) -> bool {
    let (x, y) = coord;
    (0..rock_map.width).contains(&x) && (0..rock_map.height).contains(&y)
}
