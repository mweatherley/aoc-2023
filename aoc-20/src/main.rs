use hashbrown::HashMap;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, newline},
    combinator::{map, opt},
    multi::{many0, many1},
    sequence::{preceded, separated_pair, terminated},
    IResult,
};
use std::{
    collections::{BTreeMap, VecDeque},
    fs,
};

fn main() {
    println!("Let's solve AOC-20!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> IResult<&str, Vec<Node>> {
    many1(terminated(node, newline))(input)
}

fn node(input: &str) -> IResult<&str, Node> {
    alt((flipflop, conjunction, broadcaster))(input)
}

fn flipflop(input: &str) -> IResult<&str, Node> {
    map(
        separated_pair(preceded(tag("%"), name), tag(" -> "), targets),
        |(name, targets)| Node {
            name,
            data: NodeData::FlipFlop(FlipFlop {
                state: PulseType::LO,
                targets,
            }),
        },
    )(input)
}

fn conjunction(input: &str) -> IResult<&str, Node> {
    map(
        separated_pair(preceded(tag("&"), name), tag(" -> "), targets),
        |(name, targets)| Node {
            name,
            data: NodeData::Conjunction(Conjunction {
                state: BTreeMap::default(),
                targets,
            }),
        },
    )(input)
}

fn broadcaster(input: &str) -> IResult<&str, Node> {
    map(
        separated_pair(tag("broadcaster"), tag(" -> "), targets),
        |(name, targets)| Node {
            name: name.to_string(),
            data: NodeData::Broadcaster(Broadcaster { targets }),
        },
    )(input)
}

fn targets(input: &str) -> IResult<&str, Vec<String>> {
    many0(terminated(name, opt(tag(", "))))(input)
}

fn name(input: &str) -> IResult<&str, String> {
    map(alpha1, |s: &str| s.to_string())(input)
}

/* --------------- */
/* Data Structures */
/* --------------- */
#[derive(Debug, Clone)]
struct Node {
    name: String,
    data: NodeData,
}
impl Node {
    fn into_pair(self) -> (String, NodeData) {
        (self.name, self.data)
    }
}

#[derive(Debug, Clone)]
enum NodeData {
    FlipFlop(FlipFlop),
    Conjunction(Conjunction),
    Broadcaster(Broadcaster),
}
impl NodeData {
    fn targets(&self) -> &Vec<String> {
        match self {
            NodeData::FlipFlop(inner) => &inner.targets,
            NodeData::Conjunction(inner) => &inner.targets,
            NodeData::Broadcaster(inner) => &inner.targets,
        }
    }

    fn is_conjunction(&self) -> bool {
        match self {
            NodeData::Conjunction(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
struct FlipFlop {
    state: PulseType,
    targets: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PulseType {
    HI,
    LO,
}

#[derive(Debug, Clone)]
struct Conjunction {
    state: BTreeMap<String, PulseType>,
    targets: Vec<String>,
}

#[derive(Debug, Clone)]
struct Broadcaster {
    targets: Vec<String>,
}

type NodeMap = HashMap<String, NodeData>;

#[derive(Debug, Clone)]
struct Pulse {
    source: String,
    target: String,
    hilo: PulseType,
}

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> usize {
    let (_, nodes) = problem_input(input).expect("Failed to parse problem input");
    let mut node_map: NodeMap = nodes.into_iter().map(|n| n.into_pair()).collect();
    initialize_conjunctions(&mut node_map);

    let mut hi_pulses = 0;
    let mut lo_pulses = 0;
    for _i in 0..1000 {
        let (more_hi, more_lo) = push_button(&mut node_map);
        hi_pulses += more_hi;
        lo_pulses += more_lo;
    }

    return hi_pulses * lo_pulses;
}

// Pushes the button, mutating the network of nodes and returning the number of
// HI and LO pulses produced, respectively
fn push_button(node_map: &mut NodeMap) -> (usize, usize) {
    let mut hi_pulses = 0;
    let mut lo_pulses = 0;
    let button_pulse = Pulse {
        source: String::from("button"),
        target: String::from("broadcaster"),
        hilo: PulseType::LO,
    };

    let mut pulse_queue: VecDeque<Pulse> = VecDeque::default();
    pulse_queue.push_back(button_pulse);

    while !pulse_queue.is_empty() {
        let pulse = pulse_queue.pop_front().unwrap();

        if pulse.hilo == PulseType::HI {
            hi_pulses += 1;
        } else {
            lo_pulses += 1;
        }

        let new_pulses = process(node_map, pulse);
        pulse_queue.extend(new_pulses.into_iter());
    }

    return (hi_pulses, lo_pulses);
}

fn process(node_map: &mut NodeMap, pulse: Pulse) -> Vec<Pulse> {
    if let Some(target_data) = node_map.get_mut(&pulse.target) {
        match target_data {
            NodeData::FlipFlop(ref mut data) => process_flipflop(data, pulse),
            NodeData::Conjunction(ref mut data) => process_conjunction(data, pulse),
            NodeData::Broadcaster(ref mut data) => process_broadcaster(data, pulse),
        }
    } else {
        vec![]
    }
}

fn process_flipflop(data: &mut FlipFlop, pulse: Pulse) -> Vec<Pulse> {
    if pulse.hilo == PulseType::LO {
        let new_state = match data.state {
            PulseType::HI => PulseType::LO,
            PulseType::LO => PulseType::HI,
        };
        data.state = new_state;
        return targets_to_pulses(&pulse.target, &data.targets, new_state);
    } else {
        return vec![];
    }
}

fn process_conjunction(data: &mut Conjunction, pulse: Pulse) -> Vec<Pulse> {
    data.state.insert(pulse.source, pulse.hilo);
    let output_hilo = if data.state.values().all(|x| *x == PulseType::HI) {
        PulseType::LO
    } else {
        PulseType::HI
    };
    targets_to_pulses(&pulse.target, &data.targets, output_hilo)
}

fn process_broadcaster(data: &mut Broadcaster, pulse: Pulse) -> Vec<Pulse> {
    targets_to_pulses(&pulse.target, &data.targets, pulse.hilo)
}

fn targets_to_pulses(source: &str, targets: &[String], hilo: PulseType) -> Vec<Pulse> {
    targets
        .iter()
        .map(|target| Pulse {
            source: source.to_string(),
            target: target.to_string(),
            hilo,
        })
        .collect()
}

fn initialize_conjunctions(node_map: &mut NodeMap) {
    let mut linkages: Vec<(String, String)> = vec![];
    for (name, data) in node_map.iter() {
        let targets = data.targets();
        for target in targets.iter() {
            if let Some(data) = node_map.get(target) {
                if data.is_conjunction() {
                    linkages.push((name.to_string(), target.to_string()));
                }
            }
        }
    }

    for (source, target) in linkages.into_iter() {
        add_source(node_map, source, target);
    }
}

fn add_source(node_map: &mut NodeMap, source: String, conj: String) {
    node_map.entry(conj).and_modify(|data| {
        if let NodeData::Conjunction(conj) = data {
            conj.state.insert(source, PulseType::LO);
        }
    });
}
