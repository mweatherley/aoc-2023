use nom::bytes::complete::take_until;
use nom::character::complete::{anychar, char, newline};
use nom::combinator::{map, map_opt};
use nom::error::ParseError;
use nom::multi::many0;
use nom::sequence::{preceded, terminated, tuple};
use nom::{IResult, Offset, Parser};
use std::cell::RefCell;
use std::collections::VecDeque;
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TileType {
    Pipe,
    Red,
    Blue,
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

#[derive(Clone)]
pub struct TileFillData {
    index: HashMap<(i64, i64), TileType>,
    red_count: i64,
    blue_count: i64,
    some_red: Option<(i64, i64)>,
    some_blue: Option<(i64, i64)>,
}

// State necessary for a process running along the pipe in one direction
#[derive(Debug, Clone, Copy)]
struct PipeRunnerState {
    current_location: (i64, i64),
    current_segment: PipeSegment,
    last_direction: Direction,
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> IResult<&str, ((usize, usize), PipeMap)> {
    let parser_data = RefCell::new(ProblemParserData {
        current_line: 0,
        current_cursor: 0,
        pipe_map: PipeMap {
            map: HashMap::new(),
            start: None,
        },
    });

    let (_, firstline) = terminated(take_until("\n"), newline)(input)?;

    let parser = ProblemParser { data: parser_data };
    let (rest, ()) = parser.parse_input()(input)?;
    let data = parser.data.into_inner();
    let total_lines = data.current_line;
    let total_width = firstline.len();
    Ok((rest, ((total_width, total_lines), data.pipe_map)))
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
    let (_, ((width, height), pipe_map)) =
        problem_input(input).expect("Failed to parse problem input");
    let start = pipe_map
        .start
        .expect("Failed to find the starting position");
    let mut starting_data: Vec<PipeRunnerState> = vec![];
    let mut starting_dirs: Vec<Direction> = vec![];

    // Look at each direction; the first time we find a valid direction, go that way.
    for starting_dir in EACH_DIRECTION.iter() {
        let next_coord = coord_in_dir(start, *starting_dir);
        match pipe_map.map.get(&next_coord) {
            Some(segment) => {
                if segment.incoming_dirs().contains(starting_dir) {
                    let new_state = PipeRunnerState {
                        current_location: next_coord,
                        current_segment: *segment,
                        last_direction: *starting_dir,
                    };
                    starting_data.push(new_state);
                    starting_dirs.push(*starting_dir);
                }
            }
            None => {
                continue;
            }
        }
    }
    // We ensure that the actual type of the starting block is recorded properly; this can matter
    // for computing intersection numbers later on, so this is to avoid corner cases
    let starting_data = starting_data
        .first()
        .expect("Failed to find a starting direction");
    let start_type =
        find_type_of_start(&starting_dirs).expect("Failed to find type of starting location");

    // Now, we set our guy running along the loop and updating all these wonderful mutable things:
    // > The loop history and the loop index record very similar information, searchable in different ways
    //   (i.e. one of them is actually chronological and the other is fast)
    let mut loop_history = vec![(start, start_type)];
    let mut index = HashMap::new();
    index.insert(start, TileType::Pipe);

    // > The red and blue queues store locations of tiles to the left and right of the path in its direction
    //   of movement. They are the seeds for the later algorithms that actually count all of the points in the
    //   red and blue groups.
    let mut red_to_check: VecDeque<(i64, i64)> = VecDeque::new();
    let mut blue_to_check: VecDeque<(i64, i64)> = VecDeque::new();
    run_pipe(
        *starting_data,
        &pipe_map,
        &mut loop_history,
        &mut index,
        &mut red_to_check,
        &mut blue_to_check,
    );

    // Now, it's time to process our queues and fill in the regions.
    let mut fill_data = TileFillData {
        index: index,
        red_count: 0,
        blue_count: 0,
        some_red: None,
        some_blue: None,
    };

    let width_max: i64 = width.try_into().unwrap();
    let height_max: i64 = height.try_into().unwrap();
    while !red_to_check.is_empty() {
        let reddish_tile = red_to_check.pop_back().unwrap();

        // If the pipe ended up in this tile (or if it was otherwise checked already), continue
        // (Note: The starting items of the queue will contain "unchecked" tiles; ones that are
        // added later on in this routine are "pre-checked")
        if fill_data.index.contains_key(&reddish_tile)
            || !within_max(width_max, height_max, reddish_tile)
        {
            continue;
        }

        // The pipe did not end up here, so we are definitely red.
        fill_data.index.insert(reddish_tile, TileType::Red);
        fill_data.red_count += 1;
        if fill_data.some_red.is_none() {
            fill_data.some_red = Some(reddish_tile);
        }

        // Additionally, add other adjacent tiles to the front of the queue to check.
        for dir in EACH_DIRECTION.iter() {
            let adj_tile = coord_in_dir(reddish_tile, *dir);
            if !fill_data.index.contains_key(&adj_tile)
                && within_max(width_max, height_max, adj_tile)
            {
                red_to_check.push_front(adj_tile);
            }
        }
    }
    while !blue_to_check.is_empty() {
        let bluish_tile = blue_to_check.pop_back().unwrap();

        if fill_data.index.contains_key(&bluish_tile) {
            continue;
        }

        fill_data.index.insert(bluish_tile, TileType::Blue);
        fill_data.blue_count += 1;
        if fill_data.some_blue.is_none() {
            fill_data.some_blue = Some(bluish_tile);
        }

        for dir in EACH_DIRECTION.iter() {
            let adj_tile = coord_in_dir(bluish_tile, *dir);
            if !fill_data.index.contains_key(&adj_tile)
                && within_max(width_max, height_max, adj_tile)
            {
                blue_to_check.push_front(adj_tile);
            }
        }
    }

    // Now, it remains to determine which of the two is the inside of the loop.
    if let Some(pt) = fill_data.some_red {
        if is_in_loop(&pt, &loop_history) {
            return fill_data.red_count;
        } else {
            return fill_data.blue_count;
        }
    } else if let Some(pt) = fill_data.some_blue {
        if is_in_loop(&pt, &loop_history) {
            return fill_data.blue_count;
        } else {
            return fill_data.red_count;
        }
    } else {
        return 0;
    }
}

fn run_pipe(
    start_datum: PipeRunnerState,
    pipe_map: &PipeMap,
    history: &mut Vec<((i64, i64), PipeSegment)>,
    index: &mut HashMap<(i64, i64), TileType>,
    red_queue: &mut VecDeque<(i64, i64)>,
    blue_queue: &mut VecDeque<(i64, i64)>,
) {
    let mut state = start_datum;
    'main: loop {
        // Update shared information:
        history.push((state.current_location, state.current_segment));
        index.insert(state.current_location, TileType::Pipe);

        // We color the tiles to the left and right of the path blue and red respectively;
        // these are not checked immediately for validity (e.g. the path can later intersect these locations)
        // However, we are guaranteed that a "potential" blue tile enqueued now is not actually red (and vice versa)
        let (red_locs, blue_locs) = paint(
            state.current_location,
            state.current_segment,
            state.last_direction,
        )
        .expect("Failed to acquire painting instructions");
        for r in red_locs.iter() {
            red_queue.push_front(*r);
        }
        for b in blue_locs.iter() {
            blue_queue.push_front(*b);
        }

        // Move to the next spot in the pipe:
        for dir in state.current_segment.outgoing_dirs().iter() {
            if *dir != state.last_direction.opposite() {
                let next_coord = coord_in_dir(state.current_location, *dir);
                match pipe_map.map.get(&next_coord) {
                    Some(segment) => {
                        if segment.incoming_dirs().contains(dir) {
                            if *segment == PipeSegment::Start {
                                return;
                            } else {
                                state.current_segment = *segment;
                                state.current_location = next_coord;
                                state.last_direction = *dir;
                                continue 'main;
                            }
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

fn within_max(width: i64, height: i64, pt: (i64, i64)) -> bool {
    let (x, y) = pt;
    x >= 0 && x < width && y >= 0 && y < height
}

const EACH_DIRECTION: [Direction; 4] = [
    Direction::North,
    Direction::South,
    Direction::East,
    Direction::West,
];

// Ray-casting algorithm for testing whether a point is inside the loop;
// Chose a diagonal direction so that the line is never tangent to the loop --
// here we go southeast (the line x=y in my coordinates) and notice that the
// NE and SW loop segments double-cross that line.
fn is_in_loop(test_pt: &(i64, i64), history: &Vec<((i64, i64), PipeSegment)>) -> bool {
    let (x, y) = test_pt;
    let mut total = 0;
    for ((x_p, y_p), segment) in history.iter() {
        if x_p - x == y_p - y && x_p - x >= 0 {
            if *segment == PipeSegment::NE || *segment == PipeSegment::SW {
                continue;
            } else {
                total += 1;
            }
        }
    }
    return total % 2 != 0;
}

// Extracts local tile locations to color based on the shape of the current segment
// and the last direction that was traversed (to obtain orientation)
fn paint(
    loc: (i64, i64),
    seg: PipeSegment,
    last_dir: Direction,
) -> Option<(Vec<(i64, i64)>, Vec<(i64, i64)>)> {
    let (x, y) = loc;
    match seg {
        PipeSegment::NS => match last_dir {
            Direction::North => Some((vec![(x - 1, y)], vec![(x + 1, y)])),
            Direction::South => Some((vec![(x + 1, y)], vec![(x - 1, y)])),
            _ => None,
        },
        PipeSegment::EW => match last_dir {
            Direction::East => Some((vec![(x, y - 1)], vec![(x, y + 1)])),
            Direction::West => Some((vec![(x, y + 1)], vec![(x, y - 1)])),
            _ => None,
        },
        PipeSegment::NE => match last_dir {
            Direction::South => Some((
                vec![(x + 1, y - 1)],
                vec![(x, y + 1), (x - 1, y + 1), (x - 1, y)],
            )),
            Direction::West => Some((
                vec![(x, y + 1), (x - 1, y + 1), (x - 1, y)],
                vec![(x + 1, y - 1)],
            )),
            _ => None,
        },
        PipeSegment::NW => match last_dir {
            Direction::South => Some((
                vec![(x, y + 1), (x + 1, y + 1), (x + 1, y)],
                vec![(x - 1, y - 1)],
            )),
            Direction::East => Some((
                vec![(x - 1, y - 1)],
                vec![(x, y + 1), (x + 1, y + 1), (x + 1, y)],
            )),
            _ => None,
        },
        PipeSegment::SE => match last_dir {
            Direction::North => Some((
                vec![(x, y - 1), (x - 1, y - 1), (x - 1, y)],
                vec![(x + 1, y + 1)],
            )),
            Direction::West => Some((
                vec![(x + 1, y + 1)],
                vec![(x, y - 1), (x - 1, y - 1), (x - 1, y)],
            )),
            _ => None,
        },
        PipeSegment::SW => match last_dir {
            Direction::North => Some((
                vec![(x - 1, y + 1)],
                vec![(x, y - 1), (x + 1, y - 1), (x + 1, y)],
            )),
            Direction::East => Some((
                vec![(x, y - 1), (x + 1, y - 1), (x + 1, y)],
                vec![(x - 1, y + 1)],
            )),
            _ => None,
        },
        PipeSegment::Start => None, // Should never be called
    }
}

fn find_type_of_start(start_dirs: &Vec<Direction>) -> Option<PipeSegment> {
    if start_dirs.contains(&Direction::North) && start_dirs.contains(&Direction::South) {
        return Some(PipeSegment::NS);
    }
    if start_dirs.contains(&Direction::East) && start_dirs.contains(&Direction::West) {
        return Some(PipeSegment::EW);
    }
    if start_dirs.contains(&Direction::North) && start_dirs.contains(&Direction::East) {
        return Some(PipeSegment::NE);
    }
    if start_dirs.contains(&Direction::North) && start_dirs.contains(&Direction::West) {
        return Some(PipeSegment::NW);
    }
    if start_dirs.contains(&Direction::South) && start_dirs.contains(&Direction::East) {
        return Some(PipeSegment::SE);
    }
    if start_dirs.contains(&Direction::South) && start_dirs.contains(&Direction::West) {
        return Some(PipeSegment::SW);
    }
    return None;
}
