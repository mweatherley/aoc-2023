use hashbrown::HashMap;
use std::{
    collections::{BTreeSet, VecDeque},
    fs,
};

fn main() {
    println!("Let's solve AOC-23!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> TrailMap {
    // Things we build up:
    let mut map: HashMap<Coord, Element> = HashMap::new();
    let mut start: Option<Coord> = None;
    let mut end: Option<Coord> = None;

    // Parser state:
    let mut cursor = 0;
    let mut line = 0;

    for c in input.chars() {
        match c {
            '\n' => {
                line += 1;
                cursor = 0;
            }
            '.' => {
                map.insert((cursor, line), Element::Path);

                // The start is just the first path token parsed, and
                // the end is the last.
                if start.is_none() {
                    start = Some((cursor, line));
                }
                end = Some((cursor, line));
                cursor += 1;
            }
            '^' => {
                map.insert((cursor, line), Element::Slope(Slope::Up));
                cursor += 1;
            }
            '>' => {
                map.insert((cursor, line), Element::Slope(Slope::Right));
                cursor += 1;
            }
            '<' => {
                map.insert((cursor, line), Element::Slope(Slope::Left));
                cursor += 1;
            }
            'v' => {
                map.insert((cursor, line), Element::Slope(Slope::Down));
                cursor += 1;
            }
            '#' => {
                cursor += 1;
            }
            _ => {
                panic!("Illegal character parsed");
            }
        }
    }
    return TrailMap {
        map: map,
        start: start.expect("Failed to find start"),
        end: end.expect("Failed to find end"),
    };
}

/* --------------- */
/* Data Structures */
/* --------------- */

#[derive(Debug, Clone)]
struct TrailMap {
    map: HashMap<Coord, Element>,
    start: Coord,
    end: Coord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Element {
    Slope(Slope),
    Path,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slope {
    Right,
    Left,
    Up,
    Down,
}

type Coord = (isize, isize);

#[derive(Debug, Clone)]
struct SegmentData {
    length: usize,
    flows_into: Vec<Id>,
}

struct SegmentMap {
    map: HashMap<Id, SegmentData>,
    start: Id,
    end: Id,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum Node {
    Node(NodeData),
    END,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct NodeData {
    segment: Id,
    previous_segments: BTreeSet<Id>,
}

type Id = usize;

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> usize {
    // Parse the trail map from text
    let trail_map = problem_input(input);

    // Parse the trail into its segments
    let segment_map = build_segments(&trail_map);

    let start_length = segment_map.map.get(&segment_map.start).unwrap().length;
    let start_node = Node::Node(NodeData {
        segment: segment_map.start,
        previous_segments: BTreeSet::default(),
    });

    // This is just a naive DFS.
    let mut stack: Vec<(Node, usize)> = Vec::default();
    stack.push((start_node, start_length));
    let mut current_best = 0;

    while !stack.is_empty() {
        let (node, length) = stack.pop().unwrap();

        for adj_node in adjacent_nodes(&segment_map, &node).into_iter() {
            match adj_node {
                // If we find the END node, just update our estimate
                Node::END => {
                    if length > current_best {
                        current_best = length;
                    }
                }

                // Otherwise, queue up the node together with the path length
                Node::Node(ref data) => {
                    let adj_length = segment_map.map.get(&data.segment).unwrap().length;
                    stack.push((adj_node, length + adj_length + 1));
                }
            }
        }
    }

    // Subtract 1 because we never actually leave the last tile
    current_best - 1
}

fn adjacent_nodes(segment_map: &SegmentMap, node: &Node) -> Vec<Node> {
    match node {
        // The end node is terminal
        Node::END => {
            return Vec::default();
        }

        Node::Node(node) => {
            // We impose the constraint that the end segment can only
            // move to the special END node
            if node.segment == segment_map.end {
                return vec![Node::END];
            }

            let segment_data = segment_map.map.get(&node.segment).unwrap();

            // Look at adjacent segments to this one
            segment_data
                .flows_into
                .iter()
                // Discard segments that we have previously visited
                .filter(|id| !node.previous_segments.contains(&id))
                // Add the current segment to the history as we move
                .map(|id| {
                    let mut previous_segments = node.previous_segments.clone();
                    previous_segments.insert(node.segment);
                    Node::Node(NodeData {
                        segment: *id,
                        previous_segments,
                    })
                })
                .collect::<Vec<_>>()
        }
    }
}

// `build_segments` takes the raw grid data and uses it to
// build a graph (stored as a hashmap) of path segments
// and their adjacencies.
fn build_segments(trail_map: &TrailMap) -> SegmentMap {
    // We are only going to check the path tiles
    let mut unchecked_tiles: BTreeSet<Coord> = trail_map
        .map
        .iter()
        .filter(|(_, el)| **el == Element::Path)
        .map(|(k, _)| k)
        .copied()
        .collect();

    // For the slope tiles, we will store them separately.
    // When `occupancy_map` is populated, we will
    // use them to establish the relationships between segments.
    let connections: HashMap<Coord, Slope> = trail_map
        .map
        .iter()
        .flat_map(|(coord, el)| match el {
            Element::Path => None,
            Element::Slope(slope) => Some((*coord, *slope)),
        })
        .collect();

    // `occupancy_map` stores which path-segment each point
    // belongs to as an Id.
    let mut occupancy_map: HashMap<Coord, Id> = HashMap::default();
    let mut segment_map: HashMap<Id, SegmentData> = HashMap::default();
    let mut segment_counter: Id = 0;

    // Consuming `unchecked_tiles`, we populate both `occupancy_map`
    // and `segment_map`. At this stage, the segment data's adjacency
    // information is unpopulated.
    while !unchecked_tiles.is_empty() {
        let start_tile = *unchecked_tiles.first().unwrap();
        let new_segment_data = find_segment(
            &mut unchecked_tiles,
            &mut occupancy_map,
            start_tile,
            segment_counter,
        );
        segment_map.insert(segment_counter, new_segment_data);
        segment_counter += 1;
    }

    // Now, using the locations of the slope tiles, we go back and
    // fill in the adjacency data for the segments.
    for (coord, slope) in connections.iter() {
        let (preceding_tile, following_tile) = precedes_follows(*slope, *coord);
        let preceding_id = occupancy_map
            .get(&preceding_tile)
            .expect("Failed to find preceding tile");
        let following_id = occupancy_map
            .get(&following_tile)
            .expect("Failed to find following tile");

        // This time, treat the slope tiles as two-sided, so that each
        // segment that it points to stores adjacancy for the other.
        let preceding_segment_data = segment_map
            .get_mut(preceding_id)
            .expect("Failed to load segment data");
        preceding_segment_data.flows_into.push(*following_id);
        let following_segment_data = segment_map
            .get_mut(following_id)
            .expect("Failed to load segment data");
        following_segment_data.flows_into.push(*preceding_id);
    }

    // Finally, use the `occupancy_map` to recover the IDs of the
    // starting and ending segments.
    let start = *occupancy_map.get(&trail_map.start).unwrap();
    let end = *occupancy_map.get(&trail_map.end).unwrap();

    SegmentMap {
        map: segment_map,
        start,
        end,
    }
}

fn find_segment(
    unchecked_tiles: &mut BTreeSet<Coord>,
    occupancy_map: &mut HashMap<Coord, Id>,
    start: Coord,
    id: Id,
) -> SegmentData {
    let mut search_queue: VecDeque<Coord> = VecDeque::default();
    search_queue.push_back(start);
    let mut searched = 0;

    while !search_queue.is_empty() {
        let tile = search_queue.pop_front().unwrap();

        // Update data about this tile:
        unchecked_tiles.remove(&tile);
        occupancy_map.insert(tile, id);

        searched += 1;

        // Add adjacent tiles that are unsearched
        // to the queue to process for this region
        search_queue.extend(
            adjacent_coords(tile)
                .iter()
                .filter(|t| unchecked_tiles.contains(t)),
        );
    }

    SegmentData {
        length: searched,
        flows_into: vec![],
    }
}

// Given a slope and its coordinate, recover the coordinates
// on each side of that slope.
fn precedes_follows(slope: Slope, coord: Coord) -> (Coord, Coord) {
    let (x, y) = coord;
    match slope {
        Slope::Down => ((x, y - 1), (x, y + 1)),
        Slope::Up => ((x, y + 1), (x, y - 1)),
        Slope::Left => ((x + 1, y), (x - 1, y)),
        Slope::Right => ((x - 1, y), (x + 1, y)),
    }
}

fn adjacent_coords(coord: Coord) -> Vec<Coord> {
    let (x, y) = coord;
    vec![(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
}
