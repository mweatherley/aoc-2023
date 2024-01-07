use hashbrown::HashMap;
use std::{
    collections::{BTreeMap, VecDeque},
    fs,
};

fn main() {
    println!("Let's solve AOC-21!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

const PROBLEM_DISTANCE: isize = 26501365;

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
type NodeDataMap = BTreeMap<Coord, NodeData>;

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> isize {
    let rock_map = problem_input(input);

    // We assume that the map is square and that it has an odd side length,
    // along with the fact that the starting point is in its center and
    // the center has an unobstructed path to each side. Also, we assume
    // that the edge of the block is unobstructed.
    let distance = PROBLEM_DISTANCE;
    let block_size = rock_map.width;
    let half_size = block_size / 2;

    let (one_block_white, one_block_black) =
        tiles_within_range(&rock_map, rock_map.start, None, false);

    let filled_block_inc = (distance - half_size * 2) / block_size;
    let uninverted_block_inc = filled_block_inc / 2;
    let inverted_block_inc = if filled_block_inc % 2 == 1 {
        filled_block_inc / 2 + 1
    } else {
        filled_block_inc / 2
    };
    let uninverted_blocks = 1 + 4 * uninverted_block_inc * (uninverted_block_inc + 1);
    let inverted_blocks = 4 * inverted_block_inc * inverted_block_inc;

    let interior_volume = match distance % 2 {
        // Counting white:
        0 => uninverted_blocks * one_block_white + inverted_blocks * one_block_black,
        // Counting black:
        1 => uninverted_blocks * one_block_black + inverted_blocks * one_block_white,
        _ => unreachable!(),
    };

    println!("Interior volume: {:?}", interior_volume);

    // `filled_block_distance` records the distance at which the maximum fill is
    // attained.
    let filled_block_distance = filled_block_inc * block_size + half_size * 2;
    let remaining_for_point = distance - (filled_block_distance - half_size);
    let remaining_for_inner_edge = distance - (filled_block_distance - block_size);
    let remaining_for_outer_edge = distance - filled_block_distance;

    let border_volume;
    if filled_block_inc % 2 == 1 {
        // Inner edge and points are uninverted; outer edge is inverted
        match distance % 2 {
            // Counting white
            0 => {
                let (point_total, _) = point_contributions(&rock_map, remaining_for_point, false);
                let (inner_edge_total, _) =
                    edge_contributions(&rock_map, remaining_for_inner_edge, false);
                let (outer_edge_total, _) =
                    edge_contributions(&rock_map, remaining_for_outer_edge, true);
                border_volume = point_total
                    + filled_block_inc * inner_edge_total
                    + (filled_block_inc + 1) * outer_edge_total;
            }
            // Counting black
            1 => {
                let (_, point_total) = point_contributions(&rock_map, remaining_for_point, false);
                let (_, inner_edge_total) =
                    edge_contributions(&rock_map, remaining_for_inner_edge, false);
                let (_, outer_edge_total) =
                    edge_contributions(&rock_map, remaining_for_outer_edge, true);
                border_volume = point_total
                    + filled_block_inc * inner_edge_total
                    + (filled_block_inc + 1) * outer_edge_total;
            }
            _ => unreachable!(),
        }
    } else {
        // Inner edge and points are inverted; outer edge is uninverted
        match distance % 2 {
            // Counting white
            0 => {
                let (point_total, _) = point_contributions(&rock_map, remaining_for_point, true);
                let (inner_edge_total, _) =
                    edge_contributions(&rock_map, remaining_for_inner_edge, true);
                let (outer_edge_total, _) =
                    edge_contributions(&rock_map, remaining_for_outer_edge, false);
                border_volume = point_total
                    + filled_block_inc * inner_edge_total
                    + (filled_block_inc + 1) * outer_edge_total;
            }
            // Counting black
            1 => {
                let (_, point_total) = point_contributions(&rock_map, remaining_for_point, true);
                let (_, inner_edge_total) =
                    edge_contributions(&rock_map, remaining_for_inner_edge, true);
                let (_, outer_edge_total) =
                    edge_contributions(&rock_map, remaining_for_outer_edge, false);
                border_volume = point_total
                    + filled_block_inc * inner_edge_total
                    + (filled_block_inc + 1) * outer_edge_total;
            }
            _ => unreachable!(),
        }
    }

    println!("Border volume: {}", border_volume);

    return interior_volume + border_volume;
}

// Helper function for computing border contributions for components where the
// remaining stretch begins in the middle of the tile's edge. Counted once per
// direction because of the problem's symmetry.
// Set `inverted` to true when the start point itself is black instead of white
fn point_contributions(rock_map: &RockMap, remaining: isize, inverted: bool) -> (isize, isize) {
    let size = rock_map.width;
    let (x, y) = rock_map.start;

    // Is the middle of a side the same color as the center?
    // This also inverts the answer.
    let off_color = (size / 2) % 2 == 1;

    let mut w_total = 0;
    let mut b_total = 0;

    // All of these have an offset of 1 in their distance because the
    // first move is actually taken to enter the border of the tile.
    let distance = Some(remaining - 1);
    let (nw, nb) = tiles_within_range(rock_map, (x, 0), distance, false);
    let (sw, sb) = tiles_within_range(rock_map, (x, rock_map.height - 1), distance, false);
    let (ew, eb) = tiles_within_range(rock_map, (rock_map.width - 1, y), distance, false);
    let (ww, wb) = tiles_within_range(rock_map, (0, y), distance, false);

    let w_this = nw + sw + ew + ww;
    let b_this = nb + sb + eb + wb;
    match off_color ^ inverted {
        true => {
            w_total += b_this;
            b_total += w_this;
        }
        false => {
            w_total += w_this;
            b_total += b_this;
        }
    }

    // Sometimes, there is enough room to go one tile further, so we
    // call this function again, but with a smaller remaining distance
    // and with the colors inverted.
    if remaining > size {
        let (w_ext, b_ext) = point_contributions(rock_map, remaining - size, !inverted);
        w_total += w_ext;
        b_total += b_ext;
    }

    (w_total, b_total)
}

// Helper function for computing border contributions for components where the
// remaining stretch begins in a corner of the pattern. Counted once per direction
// because of the problem's symmetry.
fn edge_contributions(rock_map: &RockMap, remaining: isize, inverted: bool) -> (isize, isize) {
    // Ditto, but with offsets of two, since we are going from corner to corner
    let distance = Some(remaining - 2);
    let (nw_w, nw_b) = tiles_within_range(
        rock_map,
        (rock_map.width - 1, rock_map.height - 1),
        distance,
        false,
    );
    let (ne_w, ne_b) = tiles_within_range(rock_map, (0, rock_map.height - 1), distance, false);
    let (se_w, se_b) = tiles_within_range(rock_map, (0, 0), distance, false);
    let (sw_w, sw_b) = tiles_within_range(rock_map, (rock_map.width - 1, 0), distance, false);

    let w_total = ne_w + nw_w + se_w + sw_w;
    let b_total = ne_b + nw_b + se_b + sw_b;
    if !inverted {
        (w_total, b_total)
    } else {
        (b_total, w_total)
    }
}

fn tiles_within_range(
    rock_map: &RockMap,
    start: Coord,
    distance: Option<isize>,
    wrap: bool,
) -> (isize, isize) {
    if let Some(dist) = distance {
        if dist < 0 {
            return (0, 0);
        }
    }

    // Basic idea: Let's just do a BFS and find all tiles reachable
    // within 64 steps. The ones that can be reached in exactly 64
    // steps are just the ones with the same parity as the start.

    let mut tile_queue: VecDeque<(Coord, isize)> = VecDeque::default();
    let mut node_data: NodeDataMap = BTreeMap::default();

    // One invariant is that we never enqueue a tile that we have
    // already enqueued before. We also need a "time" parameter so
    // that we can store the distance at which a node was reached.

    tile_queue.push_back((start, 0));
    node_data.insert(
        start,
        NodeData {
            time_reached: 0,
            color: Color::White,
        },
    );

    // We will track the total number of white tiles (those with the
    // same coordinate-parity as the start) so that we don't have to
    // compute this again later.
    let mut white_tiles = 1;
    let mut black_tiles = 0;

    while !tile_queue.is_empty() {
        let (this_tile, tile_time) = tile_queue.pop_front().unwrap();

        if let Some(max) = distance {
            if tile_time >= max {
                break;
            }
        }

        // Get the adjacent tiles that we have never recorded data about
        let new_tiles: Vec<_> = match wrap {
            true => adjacent_tiles_repeated(rock_map, this_tile)
                .into_iter()
                .filter(|tile| !node_data.contains_key(tile))
                .collect(),
            false => adjacent_tiles(&rock_map, this_tile)
                .into_iter()
                .filter(|tile| !node_data.contains_key(tile))
                .collect(),
        };

        // Record the data about these new tiles, updating our count of
        // white tiles along the way
        for adj_tile in new_tiles.iter() {
            let adj_color = match same_color(*adj_tile, start) {
                true => Color::White,
                false => Color::Black,
            };
            node_data.insert(
                *adj_tile,
                NodeData {
                    time_reached: tile_time + 1,
                    color: adj_color,
                },
            );
            if adj_color == Color::White {
                white_tiles += 1;
            } else {
                black_tiles += 1;
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
    // if wrap {
    //     for node in node_data.iter() {
    //         println!("{:?}", node);
    //     }
    // }

    return (white_tiles, black_tiles);
}

fn adjacent_tiles_repeated(rock_map: &RockMap, coord: Coord) -> Vec<Coord> {
    let (x, y) = coord;
    [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
        .into_iter()
        .filter(|(x, y)| {
            let x_n = x.rem_euclid(rock_map.width);
            let y_n = y.rem_euclid(rock_map.height);
            !rock_map.map.contains_key(&(x_n, y_n))
        })
        .collect()
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
