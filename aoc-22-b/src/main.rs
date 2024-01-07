use hashbrown::{HashMap, HashSet};
use nom::{
    bytes::complete::tag,
    character::complete::{i64, newline},
    combinator::{map, map_res},
    multi::many0,
    sequence::{separated_pair, terminated, tuple},
    IResult,
};
use priority_queue::PriorityQueue;
use std::fs;

fn main() {
    println!("Let's solve AOC-22!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> IResult<&str, Vec<Block>> {
    many0(terminated(block, newline))(input)
}

fn block(input: &str) -> IResult<&str, Block> {
    map(separated_pair(coord, tag("~"), coord), |(c1, c2)| {
        Block::from_coords(c1, c2)
    })(input)
}

fn coord(input: &str) -> IResult<&str, Coord> {
    tuple((
        terminated(value, tag(",")),
        terminated(value, tag(",")),
        value,
    ))(input)
}

fn value(input: &str) -> IResult<&str, isize> {
    map_res(i64, |v| v.try_into())(input)
}

/* --------------- */
/* Data Structures */
/* --------------- */
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct Range<T>
where
    T: Ord + Copy,
{
    start: T,
    end: T,
}
impl<T> Range<T>
where
    T: Ord + Copy,
{
    fn contains(&self, other: &T) -> bool {
        *other >= self.start && *other <= self.end
    }
}

impl Range<isize> {
    fn len(&self) -> isize {
        self.end - self.start + 1
    }
}

impl IntoIterator for Range<isize> {
    type Item = isize;
    type IntoIter = std::ops::RangeInclusive<isize>;
    fn into_iter(self) -> Self::IntoIter {
        self.start..=self.end
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IdBlock {
    id: usize,
    block: Block,
}
impl IdBlock {
    fn into_pair(self) -> (usize, Block) {
        (self.id, self.block)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Block {
    x_range: Range<isize>,
    y_range: Range<isize>,
    z_range: Range<isize>,
}
impl Block {
    fn contains(&self, coord: &Coord) -> bool {
        let (x, y, z) = coord;
        self.x_range.contains(x) && self.y_range.contains(y) && self.z_range.contains(z)
    }
    fn from_coords(start: Coord, end: Coord) -> Self {
        let (x1, y1, z1) = start;
        let (x2, y2, z2) = end;
        let x_range = Range { start: x1, end: x2 };
        let y_range = Range { start: y1, end: y2 };
        let z_range = Range { start: z1, end: z2 };
        Block {
            x_range,
            y_range,
            z_range,
        }
    }
    fn height(&self) -> isize {
        self.z_range.start
    }
    fn projection(&self) -> BlockProjection {
        BlockProjection {
            x_range: self.x_range,
            y_range: self.y_range,
        }
    }
    fn fall_to(&mut self, height: isize) {
        let length = self.z_range.len();
        self.z_range.start = height;
        self.z_range.end = height + length - 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockProjection {
    x_range: Range<isize>,
    y_range: Range<isize>,
}
impl BlockProjection {
    fn contains(&self, coord: &PlaneCoord) -> bool {
        let (x, y) = coord;
        self.x_range.contains(x) && self.y_range.contains(y)
    }
    fn coords(&self) -> Vec<PlaneCoord> {
        self.x_range
            .into_iter()
            .flat_map(|x| self.y_range.into_iter().map(move |y| (x, y)))
            .collect()
    }
}

#[derive(Debug, Clone, Copy)]
struct ElevationDatum {
    max: isize,
    arg: Id,
}

type Coord = (isize, isize, isize);
type PlaneCoord = (isize, isize);
type Id = usize;

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> usize {
    let (_, blocks) = problem_input(input).expect("Failed to parse problem input");

    // Let's give each block a number since their coordinates will change
    // when they fall.
    let id_blocks = blocks
        .into_iter()
        .enumerate()
        .map(|(id, block)| IdBlock { id, block })
        .collect::<Vec<_>>();

    let (height_map, support_map, supported_map) = blockfall(&id_blocks);

    // Only things in `support_map` can possibly cause anything to fall,
    // so we only need to iterate over those.

    support_map
        .keys()
        .map(|id| reaction_length(*id, &height_map, &support_map, &supported_map))
        .sum()
}

fn reaction_length(
    id: Id,
    height_map: &HashMap<Id, isize>,
    support_map: &HashMap<Id, Vec<Id>>,
    supported_map: &HashMap<Id, HashSet<Id>>,
) -> usize {
    let mut disturbed_block_ids: HashSet<Id> = HashSet::default();
    let mut queue: PriorityQueue<Id, isize> = PriorityQueue::default();
    let start_height = height_map.get(&id).unwrap();
    queue.push(id, -start_height);

    // The queue contains blocks that are going to be disturbed.
    while !queue.is_empty() {
        let (next_id, _) = queue.pop().unwrap();

        // When a block is dequeued, it is considered disturbed
        disturbed_block_ids.insert(next_id);

        // Look at the blocks it supports:
        if let Some(supported_ids) = support_map.get(&next_id) {
            for supported_id in supported_ids.iter() {
                let support_set = supported_map.get(supported_id).unwrap();

                // If all of the block's supports are now disturbed, then it
                // will become disturbed itself, so we enqueue it.
                if support_set.is_subset(&disturbed_block_ids) {
                    let height = height_map.get(supported_id).unwrap();
                    queue.push(*supported_id, -height);
                }
            }
        }
    }

    return disturbed_block_ids.len() - 1;
}

fn blockfall(
    id_blocks: &[IdBlock],
) -> (
    HashMap<Id, isize>,
    HashMap<Id, Vec<Id>>,
    HashMap<Id, HashSet<Id>>,
) {
    let mut queue: PriorityQueue<IdBlock, isize> = PriorityQueue::default();
    let mut elevation_map: HashMap<PlaneCoord, ElevationDatum> = HashMap::default();
    let mut height_map: HashMap<Id, isize> = HashMap::default();
    let mut support_map: HashMap<Id, Vec<Id>> = HashMap::default();
    let mut supported_map: HashMap<Id, HashSet<Id>> = HashMap::default();

    // Enqueue everything, prioritizing low-height blocks.
    // This is a max-priority queue, so we negate height as our priority.
    queue.extend(
        id_blocks
            .iter()
            .map(|id_block| (id_block.clone(), -id_block.block.height())),
    );

    while !queue.is_empty() {
        let (id_block, _) = queue.pop().unwrap();
        one_fall(
            &id_block,
            &mut elevation_map,
            &mut height_map,
            &mut support_map,
            &mut supported_map,
        );
    }

    (height_map, support_map, supported_map)
}

fn one_fall(
    id_block: &IdBlock,
    elevation_map: &mut HashMap<PlaneCoord, ElevationDatum>,
    height_map: &mut HashMap<Id, isize>,
    support_map: &mut HashMap<Id, Vec<Id>>,
    supported_map: &mut HashMap<Id, HashSet<Id>>,
) {
    // Collect the elevation data lying below this block.
    let shadow = id_block.block.projection();
    let things_below: Vec<_> = shadow
        .coords()
        .iter()
        .flat_map(|coord| elevation_map.get(coord))
        .collect();

    // Using this, find the highest elevation; i.e. the height of the
    // block(s) this one will rest on. This will be None if nothing
    // is below this block.
    let maybe_max_height = things_below.iter().map(|datum| datum.max).max();

    let peaks: HashSet<usize>;
    let new_height: isize;
    match maybe_max_height {
        // There is actually something below this
        Some(max_height) => {
            peaks = things_below
                .into_iter()
                .filter(|datum| datum.max == max_height)
                .map(|datum| datum.arg)
                .collect();
            new_height = max_height + 1;
        }

        // There is nothing below this
        None => {
            peaks = HashSet::default();
            new_height = 1;
        }
    }

    let (id, mut block) = id_block.clone().into_pair();
    let vert_length = block.z_range.len();

    // Record the new height of blocks since we use this for priority
    // queuing when we compute chain reactions
    height_map.insert(id, new_height);

    // Change the block's height to the correct value after falling
    block.fall_to(new_height);

    // Update the elevation map with the new block. It is guaranteed
    // to be the maximum in all of the tiles it shadows
    for coord in shadow.coords().iter() {
        elevation_map.insert(
            *coord,
            ElevationDatum {
                max: new_height + (vert_length - 1),
                arg: id,
            },
        );
    }

    // Now, using `peaks`, which contains the list of blocks supporting
    // this block, we update the support map. This time, we include all
    // the blocks that support this one, even if they aren't alone.
    for sup_id in peaks.iter() {
        support_map
            .entry(*sup_id)
            .and_modify(|v| v.push(id))
            .or_insert(vec![id]);
    }

    // Record the inverse information as well in `supported_map`
    supported_map.insert(id, peaks);
}
