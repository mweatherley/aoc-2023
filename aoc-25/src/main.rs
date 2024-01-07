use hashbrown::{HashMap, HashSet};
use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, newline, space0},
    combinator::{map, map_res},
    multi::many0,
    sequence::{preceded, separated_pair, terminated},
    IResult,
};
use std::{
    collections::{BTreeMap, VecDeque},
    fs,
};

fn main() {
    println!("Let's solve AOC-25!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> IResult<&str, Vec<ProtoNode>> {
    many0(terminated(proto_node, newline))(input)
}

fn proto_node(input: &str) -> IResult<&str, ProtoNode> {
    map(separated_pair(label, tag(":"), label_list), |(l, ls)| {
        ProtoNode(l, ls)
    })(input)
}

fn label_list(input: &str) -> IResult<&str, Vec<Label>> {
    many0(preceded(space0, label))(input)
}

fn label(input: &str) -> IResult<&str, Label> {
    map_res(alpha1, |s: &str| s.chars().collect::<Vec<_>>().try_into())(input)
}

/* --------------- */
/* Data Structures */
/* --------------- */

// ProtoNode is a node-model parsed directly from
// the problem input; i.e. it is missing reverse
// edges.
struct ProtoNode(Label, Vec<Label>);

struct NodeData {
    adjacent_nodes: Vec<Label>,
}

// `EdgeData` tracks flow across and edge and also
// its residual; i.e. it is used for both the flow
// and the flow's associated residual graph
#[derive(Debug, Clone, Copy)]
struct EdgeData {
    flow: Quantity,
}
impl EdgeData {
    // Is this edge part of the residual graph?
    fn admissible(&self) -> bool {
        self.flow < 1 // Rearrangement of capacity - flow = 1 - flow > 0
    }
}

type Quantity = i32;
type Label = [char; 3];

type FlowMap = HashMap<Label, BTreeMap<Label, EdgeData>>;
type SimplePath = VecDeque<Label>;

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> u64 {
    let (_, proto_nodes) = problem_input(input).expect("Failed to parse problem input");
    let mut node_map: HashMap<Label, NodeData> = HashMap::default();

    // Populate a map of adjacency data based on the given presentation
    for proto_node in proto_nodes.into_iter() {
        absorb(&mut node_map, proto_node);
    }
    let blank_map: FlowMap = node_map
        .into_iter()
        .map(|(l, data)| {
            let tree = data
                .adjacent_nodes
                .into_iter()
                .map(|adj_label| empty_flow(&adj_label))
                .collect::<BTreeMap<_, _>>();
            (l, tree)
        })
        .collect();

    let mut ins = 1;
    let mut outs = 0;

    if blank_map.is_empty() {
        panic!();
    }

    let first = blank_map.keys().next().unwrap();
    for other in blank_map.keys().skip(1) {
        match connected(blank_map.clone(), first, other) {
            true => {
                ins += 1;
            }
            false => {
                outs += 1;
            }
        }
    }

    ins * outs
}

// Edge data for an empty flow from an anonymous node
// into a specified one
fn empty_flow(label: &Label) -> (Label, EdgeData) {
    let null_flow_data = EdgeData { flow: 0 };
    (*label, null_flow_data)
}

// Here we use the Ford-Fulkerson method to tell whether two nodes would
// be in the same component after the three-edge cut is performed.
// If the max-flow between the nodes exceeds 3, then they must be in the
// same component. Otherwise, they are disconnected by a 3-edge cut.
fn connected(mut flow_map: FlowMap, source: &Label, sink: &Label) -> bool {
    let mut total_flow = 0;
    loop {
        match find_path(&flow_map, source, sink) {
            Some(path) => {
                update(&mut flow_map, path);
                total_flow += 1;
                if total_flow > 3 {
                    return true;
                }
            }
            None => {
                if total_flow <= 3 {
                    return false;
                } else {
                    return true;
                }
            }
        }
    }
}

// We just always assume that the weight of the path is 1. This is not
// technically always the case, but for it to not be the case, every single
// path piece would have to reverse flow, so it's okay. In that case, we
// would just find the same path twice anyway.
fn update(flow_map: &mut FlowMap, path: SimplePath) {
    for (preceding, following) in path.iter().zip(path.iter().skip(1)) {
        update_one(flow_map, preceding, following);
    }
}

fn update_one(flow_map: &mut FlowMap, preceding: &Label, following: &Label) {
    // Outgoing side
    let prec_data = flow_map.get_mut(preceding).unwrap();
    prec_data
        .entry(*following)
        .and_modify(|data| data.flow += 1);

    // Incoming side
    let foll_data = flow_map.get_mut(following).unwrap();
    foll_data
        .entry(*preceding)
        .and_modify(|data| data.flow -= 1);
}

// Finds a path in the residual graph inferred from `flow_map`
// using a depth-first search.
fn find_path(flow_map: &FlowMap, source: &Label, sink: &Label) -> Option<SimplePath> {
    let mut path = VecDeque::default();
    visit(
        &mut path,
        &mut HashSet::default(),
        &mut false,
        source,
        sink,
        flow_map,
    );
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

fn visit(
    path: &mut SimplePath,
    visited_nodes: &mut HashSet<Label>,
    done: &mut bool,
    node: &Label,
    goal: &Label,
    flow_map: &FlowMap,
) {
    visited_nodes.insert(*node);
    if node == goal {
        *done = true;
    }
    if !*done {
        let data = flow_map.get(node).unwrap();
        for (l, _) in data.iter().filter(|(_, edge_data)| edge_data.admissible()) {
            if !visited_nodes.contains(l) {
                visit(path, visited_nodes, done, l, goal, flow_map);

                // Stop looking, we're done (and those nodes would get
                // the wrong idea about being part of the path)
                if *done {
                    break;
                }
            }
        }
    }
    // This node is part of the path; we are falling down the stack
    if *done {
        path.push_front(*node);
    }
}

fn absorb(node_map: &mut HashMap<Label, NodeData>, new_node: ProtoNode) {
    let ProtoNode(label, mut adjacent_labels) = new_node;

    // For every discovered adjacent node, create a stub entry
    // that just knows about this node. If the entry exists,
    // then just add this node to its adjacencies.
    for adj_label in adjacent_labels.iter() {
        node_map
            .entry(*adj_label)
            .and_modify(|adj_data| adj_data.adjacent_nodes.push(label))
            .or_insert(NodeData {
                adjacent_nodes: vec![label],
            });
    }

    // If the entry for the given label doesn't exist, create it
    // using the list of adjacent labels. Otherwise, append the
    // labels we found here to its data.
    node_map
        .entry(label)
        .and_modify(|data| data.adjacent_nodes.append(&mut adjacent_labels))
        .or_insert(NodeData {
            adjacent_nodes: adjacent_labels,
        });
}
