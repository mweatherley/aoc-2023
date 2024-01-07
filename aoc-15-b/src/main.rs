use std::{
    collections::{HashMap, VecDeque},
    fs,
};

use nom::character::complete::{char, u32};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    combinator::{map, opt},
    multi::many0,
    sequence::{separated_pair, terminated},
    IResult,
};

fn main() {
    println!("Let's solve AOC-15!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("aoc-15-input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

/* --------------- */
/* Data Structures */
/* --------------- */

#[derive(Clone, Debug)]
struct Lens {
    label: String,
    focal_length: u64,
}

#[derive(Debug)]
struct LensBox {
    contents: VecDeque<Lens>,
    label_map: HashMap<String, usize>,
}

impl LensBox {
    fn sum(&self) -> u64 {
        let mut total = 0;
        for (idx, lens) in self.contents.iter().enumerate() {
            let idx = idx as u64;
            total += (idx + 1) * lens.focal_length
        }
        return total;
    }

    fn new() -> Self {
        LensBox {
            contents: VecDeque::new(),
            label_map: HashMap::new(),
        }
    }

    fn perform_instruction(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::Remove(label) => match self.label_map.contains_key(label) {
                true => {
                    let this_idx = *self.label_map.get(label).unwrap();
                    self.contents.remove(this_idx);
                    self.label_map.remove(label);
                    for (_label, idx) in self
                        .label_map
                        .iter_mut()
                        .filter(|(_, idx)| **idx > this_idx)
                    {
                        *idx -= 1;
                    }
                }
                false => {
                    return;
                }
            },
            Instruction::Insert(label, value) => match self.label_map.contains_key(label) {
                true => {
                    self.contents[*self.label_map.get(label).unwrap()].focal_length = *value;
                }
                false => {
                    let lens = Lens {
                        label: label.clone(),
                        focal_length: *value,
                    };
                    let length = self.contents.len();
                    self.contents.push_back(lens);
                    self.label_map.insert(label.to_string(), length);
                }
            },
        }
    }
}

#[derive(Debug)]
enum Instruction {
    Remove(String),
    Insert(String, u64),
}

impl Instruction {
    fn label(&self) -> String {
        match self {
            Instruction::Remove(label) => label.to_string(),
            Instruction::Insert(label, _) => label.to_string(),
        }
    }
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> IResult<&str, Vec<Instruction>> {
    many0(instruction)(input)
}

fn instruction(input: &str) -> IResult<&str, Instruction> {
    let (rest, segment) = terminated(take_while1(|c| c != ','), opt(char(',')))(input)?;
    let (_, instruction) = alt((remove, insert))(segment)?;
    Ok((rest, instruction))
}

fn remove(input: &str) -> IResult<&str, Instruction> {
    map(
        separated_pair(take_until("="), tag("="), u32),
        |(label, val): (&str, u32)| Instruction::Insert(label.to_string(), val as u64),
    )(input)
}

fn insert(input: &str) -> IResult<&str, Instruction> {
    map(terminated(take_until("-"), char('-')), |label: &str| {
        Instruction::Remove(label.to_string())
    })(input)
}

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> u64 {
    let (_, instructions) = problem_input(input).expect("Failed to parse problem input");

    // Build up the box contents from instructions
    let mut box_map: HashMap<u8, LensBox> = HashMap::new();
    for instruction in instructions.iter() {
        let box_no = hash_label(&instruction.label());
        box_map
            .entry(box_no)
            .and_modify(|lens_box| lens_box.perform_instruction(instruction))
            .or_insert_with(|| {
                let mut lens_box = LensBox::new();
                lens_box.perform_instruction(instruction);
                lens_box
            });
    }

    // Sum the results
    box_map
        .iter()
        .map(|(k, lens_box)| ((*k as u64) + 1) * lens_box.sum())
        .sum()
}

fn hash_label(label: &str) -> u8 {
    hash(label.as_bytes())
}

fn hash(xs: &[u8]) -> u8 {
    let mut val: u8 = 0;
    for x in xs.iter() {
        val = val.wrapping_add(*x);
        val = val.wrapping_mul(17);
    }
    return val;
}
