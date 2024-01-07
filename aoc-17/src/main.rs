use hashbrown::HashMap;
use priority_queue::PriorityQueue;

fn main() {
    println!("Let's solve AOC-17!");
    let now = std::time::Instant::now();
    let input = std::fs::read_to_string("input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> CityMap {
    // Things we build up:
    let mut map: Vec<Vec<isize>> = vec![vec![]];

    // Parser state:
    let mut line = 0;
    for c in input.chars() {
        match c {
            '\n' => {
                line += 1;
                map.push(vec![]);
            }
            '1' => {
                map[line].push(1);
            }
            '2' => {
                map[line].push(2);
            }
            '3' => {
                map[line].push(3);
            }
            '4' => {
                map[line].push(4);
            }
            '5' => {
                map[line].push(5);
            }
            '6' => {
                map[line].push(6);
            }
            '7' => {
                map[line].push(7);
            }
            '8' => {
                map[line].push(8);
            }
            '9' => {
                map[line].push(9);
            }
            _ => {
                panic!("Illegal character parsed");
            }
        }
    }
    return CityMap {
        width: map[0].len(),
        height: line,
        heats: map,
    };
}

/* --------------- */
/* Data Structures */
/* --------------- */
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Direction {
    N,
    S,
    E,
    W,
}

impl Direction {
    fn can_turn_toward(&self, other: Direction) -> bool {
        match self {
            Direction::N => match other {
                Direction::E | Direction::W => true,
                _ => false,
            },
            Direction::S => match other {
                Direction::E | Direction::W => true,
                _ => false,
            },
            Direction::E => match other {
                Direction::N | Direction::S => true,
                _ => false,
            },
            Direction::W => match other {
                Direction::N | Direction::S => true,
                _ => false,
            },
        }
    }

    fn opposite(&self) -> Direction {
        match self {
            Direction::N => Direction::S,
            Direction::S => Direction::N,
            Direction::E => Direction::W,
            Direction::W => Direction::E,
        }
    }
}

const EACH_DIRECTION: [Direction; 4] = [Direction::N, Direction::S, Direction::E, Direction::W];

struct CityMap {
    width: usize,
    height: usize,
    heats: Vec<Vec<Heat>>,
}

type Coord = (isize, isize);
type Heat = isize;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct CrucibleState {
    last_dir: Option<Direction>,
    consecs: usize,
    coord: Coord,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct NodeData {
    estimate: Heat,
    previous: Option<Node>,
}

// Nodes are augmented by a special END node which the states representing the
// finish location all have a special 0-weight map to. This means that we only need
// to find the optimal path from the starting node to END instead of to all of the
// possible states that coincide with the finish location geometrically.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Node {
    Normal(CrucibleState),
    END,
}

type EstimateMap = HashMap<Node, NodeData>;

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> Heat {
    let city_map = problem_input(input);
    let start_node = Node::Normal(CrucibleState {
        last_dir: None,
        consecs: 0,
        coord: (0, 0),
    });
    let estimates = dijkstra(&city_map, start_node);
    estimates.get(&Node::END).unwrap().estimate
}

fn dijkstra(city_map: &CityMap, start_node: Node) -> EstimateMap {
    let start_data = NodeData {
        estimate: 0,
        previous: None,
    };
    let mut estimates: EstimateMap = HashMap::new();
    estimates.insert(start_node, start_data);

    // Note: This is a max-priority queue, so we need to negate node distance estimates
    // when we use them as priority.
    let mut pri_queue: PriorityQueue<Node, isize> = PriorityQueue::new();
    pri_queue.push(start_node, -start_data.estimate);

    while !pri_queue.is_empty() {
        let (node, _) = pri_queue.pop().unwrap();
        for edge in outgoing_edges(city_map, node) {
            if let Some(new_estimate) = relax(&mut estimates, node, edge) {
                let (_, other_node) = edge;
                pri_queue.push_increase(other_node, -new_estimate);
            }
        }
    }

    return estimates;
}

// Given a Node and an outgoing weighted edge, we update our estimate for the target
// of that edge:
// > if an estimate exists and traversing the edge improves it, then we update it
// > if no estimate exists (estimate infinity) then we instantiate one based on this edge
// Returns the estimate of the target when an update occurs; otherwise returns None
fn relax(estimates: &mut EstimateMap, state: Node, outgoing_edge: (Heat, Node)) -> Option<Heat> {
    let (heat, next_state) = outgoing_edge;
    if !estimates.contains_key(&state) {
        return None;
        // panic!("Tried to relax based on null data");
    }
    let state_data = *estimates.get(&state).unwrap();
    let next_state_data = estimates
        .entry(next_state)
        .and_modify(|next_state_data| {
            if state_data.estimate + heat < next_state_data.estimate {
                next_state_data.estimate = state_data.estimate + heat;
                next_state_data.previous = Some(state);
            }
        })
        .or_insert(NodeData {
            estimate: state_data.estimate + heat,
            previous: Some(state),
        });

    if next_state_data.previous == Some(state) {
        Some(next_state_data.estimate)
    } else {
        None
    }
}

// This function gives the weighted adjacency list for a node in our graph.
fn outgoing_edges(city_map: &CityMap, current_node: Node) -> Vec<(Heat, Node)> {
    let mut outgoing_edges = vec![];
    match current_node {
        Node::END => {}
        Node::Normal(current_state) => {
            if is_terminus(city_map, current_state) {
                outgoing_edges.push((0, Node::END));
            }
            for dir in EACH_DIRECTION.iter() {
                match coord_in_direction(city_map, current_state.coord, *dir) {
                    // This direction runs off a boundary
                    None => {
                        continue;
                    }

                    // This is in bounds
                    Some(new_coord) => {
                        let outgoing_heat = heat_at_coord(city_map, new_coord);
                        match current_state.last_dir {
                            None => {
                                let next_node = Node::Normal(CrucibleState {
                                    last_dir: Some(*dir),
                                    consecs: 1,
                                    coord: new_coord,
                                });
                                outgoing_edges.push((outgoing_heat, next_node));
                                continue;
                            }
                            Some(last_dir) => {
                                // We have to turn:
                                if current_state.consecs == 3 && !last_dir.can_turn_toward(*dir) {
                                    continue;
                                }

                                // Reversing is not allowed either
                                if *dir == last_dir.opposite() {
                                    continue;
                                }

                                // Unless we go the same direction, the number of times we have gone this way
                                // consecutively is just 1
                                let mut new_consecs = 1;

                                // But if we go the same direction, this obviously increments instead
                                if last_dir == *dir {
                                    new_consecs = current_state.consecs + 1;
                                }

                                let next_node = Node::Normal(CrucibleState {
                                    last_dir: Some(*dir),
                                    consecs: new_consecs,
                                    coord: new_coord,
                                });
                                outgoing_edges.push((outgoing_heat, next_node));
                                continue;
                            }
                        }
                    }
                }
            }
        }
    }

    return outgoing_edges;
}

// Check to see if a node is one of the special nodes requiring a 0-weight sink edge
fn is_terminus(city_map: &CityMap, state: CrucibleState) -> bool {
    let x = (city_map.width as isize) - 1;
    let y = (city_map.height as isize) - 1;
    state.coord == (x, y)
}

fn heat_at_coord(city_map: &CityMap, coord: Coord) -> Heat {
    let (x, y) = coord;
    let x = x as usize;
    let y = y as usize;
    city_map.heats[y][x]
}

fn coord_in_direction(city_map: &CityMap, start: Coord, direction: Direction) -> Option<Coord> {
    let (x, y) = start;
    match direction {
        Direction::N => bounded_coord(city_map, (x, y - 1)),
        Direction::S => bounded_coord(city_map, (x, y + 1)),
        Direction::E => bounded_coord(city_map, (x + 1, y)),
        Direction::W => bounded_coord(city_map, (x - 1, y)),
    }
}

fn bounded_coord(city_map: &CityMap, coord: Coord) -> Option<Coord> {
    if coord_in_bounds(city_map, coord) {
        Some(coord)
    } else {
        None
    }
}

fn coord_in_bounds(city_map: &CityMap, coord: Coord) -> bool {
    let (x, y) = coord;
    (0..(city_map.width as isize)).contains(&x) && (0..(city_map.height as isize)).contains(&y)
}
