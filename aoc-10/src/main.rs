use nom::character::complete::{anychar, char, newline};
use nom::combinator::{map, map_opt};
use nom::error::ParseError;
use nom::multi::many0;
use nom::sequence::{preceded, tuple};
use nom::{IResult, Offset, Parser};
use std::cell::RefCell;
use std::sync::Mutex;
use std::thread;
use std::{collections::HashMap, fs};

fn main() {
    println!("Let's solve AOC-10!");
    let input = fs::read_to_string("aoc-10-input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Solution: {}", solution);
}

/* --------------- */
/* Data Structures */
/* --------------- */
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum PipeSegment {
    NS,
    EW,
    NW,
    NE,
    SE,
    SW,
    Start,
}

impl PipeSegment {
    fn outgoing_dirs(&self) -> Vec<Direction> {
        match self {
            PipeSegment::NS => vec![Direction::North, Direction::South],
            PipeSegment::EW => vec![Direction::East, Direction::West],
            PipeSegment::NW => vec![Direction::North, Direction::West],
            PipeSegment::NE => vec![Direction::North, Direction::East],
            PipeSegment::SE => vec![Direction::South, Direction::East],
            PipeSegment::SW => vec![Direction::South, Direction::West],
            PipeSegment::Start => vec![
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
            ],
        }
    }
    fn incoming_dirs(&self) -> Vec<Direction> {
        match self {
            PipeSegment::NS => vec![Direction::North, Direction::South],
            PipeSegment::EW => vec![Direction::East, Direction::West],
            PipeSegment::NW => vec![Direction::South, Direction::East],
            PipeSegment::NE => vec![Direction::South, Direction::West],
            PipeSegment::SE => vec![Direction::North, Direction::West],
            PipeSegment::SW => vec![Direction::North, Direction::East],
            PipeSegment::Start => vec![
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
            ],
        }
    }
}

#[derive(Debug, Clone)]
struct PipeMap {
    map: HashMap<(i64, i64), PipeSegment>,
    start: Option<(i64, i64)>,
}

// State necessary for a process running along the pipe in one direction
#[derive(Debug, Clone, Copy)]
struct PipeRunnerState {
    steps_travelled: i64,
    current_location: (i64, i64),
    current_segment: PipeSegment,
    last_direction: Direction,
}

#[derive(Debug)]
struct SharedState {
    points_tested: Mutex<HashMap<(i64, i64), i64>>,
    answer: Mutex<Option<i64>>,
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> IResult<&str, PipeMap> {
    let parser_data = RefCell::new(ProblemParserData {
        current_line: 0,
        current_cursor: 0,
        pipe_map: PipeMap {
            map: HashMap::new(),
            start: None,
        },
    });

    let parser = ProblemParser { data: parser_data };
    let (rest, ()) = parser.parse_input()(input)?;
    let data = parser.data.into_inner();
    Ok((rest, data.pipe_map))
}

/// We need to pass around shared mutable state between our parsers, which we do by
/// 1) Using a struct ("parser object") to maintain the shared state of all of these parsers
/// 2) Using methods that return parsers, allowing them to interoperate with `nom`
/// 3) Locking the shared state in a `Cell` so that it can be "simultaneously" used by many parsers in a combinator
/// (Note: It is not actually used simultaneously at all, but the function calls cannot know that; they would just otherwise
/// see something being mutably borrowed by the same function several times and freak out.)
struct ProblemParser {
    data: RefCell<ProblemParserData>,
}

#[derive(Clone)]
struct ProblemParserData {
    current_line: usize,
    current_cursor: usize,
    pipe_map: PipeMap,
}

impl ProblemParser {
    fn next_line(&self) {
        let mut data = self.data.borrow_mut();
        data.current_line = data.current_line + 1;
        data.current_cursor = 0;
    }

    fn parse_input<'a>(&'a self) -> impl FnMut(&str) -> IResult<&str, ()> + 'a {
        |input| {
            map(
                many0(tuple((
                    many0(self.parse_pipe_segment()),
                    self.parse_blank_space(),
                    self.parse_newline(),
                ))),
                |_| (),
            )(input) // Literally kill the output
        }
    }

    fn parse_pipe_segment<'a>(&'a self) -> impl FnMut(&str) -> IResult<&str, ()> + 'a {
        |input| {
            let (rest, (offset, segment)) =
                with_offset(map_opt(pipe_segment_char, char_to_segment))(input)?;
            let mut data = self.data.borrow_mut();
            data.current_cursor += offset;
            let y_position: i64 = data
                .current_line
                .try_into()
                .expect("y position out of i64 bounds");
            let x_position: i64 = data
                .current_cursor
                .try_into()
                .expect("x position out of i64 bounds");
            let x_position = x_position - 1; // Just doing subtraction after converting to a signed thing for hygiene
            data.pipe_map.map.insert((x_position, y_position), segment);
            if segment == PipeSegment::Start {
                data.pipe_map.start = Some((x_position, y_position))
            }
            Ok((rest, ()))
        }
    }

    fn parse_blank_space<'a>(&'a self) -> impl FnMut(&str) -> IResult<&str, ()> + 'a {
        |input| {
            let (rest, (offset, _)) = with_offset(many0(char('.')))(input)?;
            let mut data = self.data.borrow_mut();
            data.current_cursor += offset;
            Ok((rest, ()))
        }
    }

    fn parse_newline<'a>(&'a self) -> impl FnMut(&str) -> IResult<&str, ()> + 'a {
        |input| {
            let (rest, _) = newline(input)?;
            self.next_line();
            Ok((rest, ()))
        }
    }
}

fn pipe_segment_char(input: &str) -> IResult<&str, char> {
    preceded(many0(char('.')), anychar)(input)
}

fn char_to_segment(c: char) -> Option<PipeSegment> {
    match c {
        '|' => Some(PipeSegment::NS),
        '-' => Some(PipeSegment::EW),
        'F' => Some(PipeSegment::SE),
        'J' => Some(PipeSegment::NW),
        '7' => Some(PipeSegment::SW),
        'L' => Some(PipeSegment::NE),
        'S' => Some(PipeSegment::Start),
        _ => None,
    }
}

fn with_offset<F, I, O, E>(mut parser: F) -> impl FnMut(I) -> IResult<I, (usize, O), E>
where
    F: Parser<I, O, E>,
    I: Clone + Offset,
    E: ParseError<I>,
{
    move |input: I| {
        let i = input.clone();
        match parser.parse(i) {
            Ok((rest, output)) => {
                let offset = input.offset(&rest);
                return Ok((rest, (offset, output)));
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> i64 {
    let (_, pipe_map) = problem_input(input).expect("Failed to parse problem input");
    let start = pipe_map
        .start
        .expect("Failed to find the starting position");
    let mut starting_data: Vec<PipeRunnerState> = vec![];

    // Look at each direction and wherever we find a pipe connection, create data to
    // instantiate a subprocess
    for starting_dir in EACH_DIRECTION.iter() {
        let next_coord = coord_in_dir(start, *starting_dir);
        match pipe_map.map.get(&next_coord) {
            Some(segment) => {
                if segment.incoming_dirs().contains(starting_dir) {
                    let new_state = PipeRunnerState {
                        steps_travelled: 1,
                        current_location: next_coord,
                        current_segment: *segment,
                        last_direction: *starting_dir,
                    };
                    starting_data.push(new_state);
                }
            }
            None => {
                continue;
            }
        }
    }
    let shared = SharedState {
        points_tested: Mutex::new(HashMap::new()),
        answer: Mutex::new(None),
    };
    thread::scope(|s| {
        for start_datum in starting_data.iter() {
            s.spawn(|| run_off(*start_datum, &pipe_map, &shared));
        }
    });

    let solution = shared
        .answer
        .into_inner()
        .expect("Answer mutex was poisoned")
        .expect("Failed to set answer in shared data");
    return solution;
}

fn run_off(start_datum: PipeRunnerState, pipe_map: &PipeMap, shared: &SharedState) {
    let mut state = start_datum;
    let mut entire_history = vec![state.clone()];
    'main: loop {
        // Update shared information:
        let mut pad = shared
            .points_tested
            .lock()
            .expect("Failed to lock shared pad");
        match pad.get(&state.current_location) {
            None => {
                pad.insert(state.current_location, state.steps_travelled);
            }
            Some(steps) => {
                let mut answer = shared.answer.lock().expect("Failed to lock shared answer");
                *answer = Some((steps + state.steps_travelled) / 2);
                return;
            }
        }

        // Move to the next spot in the pipe:
        for dir in state.current_segment.outgoing_dirs().iter() {
            if *dir != state.last_direction.opposite() {
                let next_coord = coord_in_dir(state.current_location, *dir);
                match pipe_map.map.get(&next_coord) {
                    Some(segment) => {
                        if segment.incoming_dirs().contains(dir) {
                            state.current_segment = *segment;
                            state.current_location = next_coord;
                            state.steps_travelled += 1;
                            state.last_direction = *dir;
                            entire_history.push(state.clone());
                            continue 'main;
                        } else {
                            panic!("Tried to go down an illegal pipe");
                        }
                    }
                    None => {
                        continue;
                    }
                }
            }
        }
    }
}

fn coord_in_dir(start: (i64, i64), direction: Direction) -> (i64, i64) {
    let (x, y) = start;
    match direction {
        Direction::North => (x, y - 1),
        Direction::South => (x, y + 1),
        Direction::East => (x + 1, y),
        Direction::West => (x - 1, y),
    }
}

const EACH_DIRECTION: [Direction; 4] = [
    Direction::North,
    Direction::South,
    Direction::East,
    Direction::West,
];
