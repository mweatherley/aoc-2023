use nom::branch::alt;
use nom::bytes::complete::{tag, take_until1};
use nom::character::complete::{alpha1, anychar, char, newline, u32};
use nom::combinator::{map, map_opt, map_res, opt, value};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, separated_pair, terminated, tuple};
use nom::IResult;
use std::collections::HashMap;
use std::fs;
use std::ops::Range;

fn main() {
    println!("Let's solve AOC-19!");
    let now = std::time::Instant::now();
    let input = fs::read_to_string("input.txt").expect("Unable to read file");
    let solution = solve_problem(&input);
    println!("Elapsed: {:?}", now.elapsed());
    println!("Solution: {}", solution);
}

/* ------- */
/* Parsers */
/* ------- */

fn problem_input(input: &str) -> IResult<&str, Vec<Workflow>> {
    let (rest, workflows) = many1(terminated(workflow, newline))(input)?;
    Ok((rest, workflows))
}

fn workflow(input: &str) -> IResult<&str, Workflow> {
    map(
        pair(
            take_until1("{"),
            delimited(char('{'), instructions, char('}')),
        ),
        |(name, instructions)| Workflow {
            name: name.to_string(),
            instructions,
        },
    )(input)
}

fn instructions(input: &str) -> IResult<&str, Vec<Instruction>> {
    many0(terminated(instruction, opt(char(','))))(input)
}

fn instruction(input: &str) -> IResult<&str, Instruction> {
    alt((
        map(
            separated_pair(condition, tag(":"), outcome),
            |(condition, outcome)| Instruction::Conditional(condition, outcome),
        ),
        map(outcome, |outcome| Instruction::Unconditional(outcome)),
    ))(input)
}

fn condition(input: &str) -> IResult<&str, Condition> {
    map(
        tuple((characteristic, comparator, threshold)),
        |(characteristic, comparator, threshold)| Condition {
            characteristic,
            comparator,
            threshold,
        },
    )(input)
}

fn characteristic(input: &str) -> IResult<&str, Characteristic> {
    map_opt(anychar, |c| match c {
        'x' => Some(Characteristic::X),
        'm' => Some(Characteristic::M),
        'a' => Some(Characteristic::A),
        's' => Some(Characteristic::S),
        _ => None,
    })(input)
}

fn comparator(input: &str) -> IResult<&str, Comparator> {
    alt((
        value(Comparator::LessThan, char('<')),
        value(Comparator::GreaterThan, char('>')),
    ))(input)
}

fn outcome(input: &str) -> IResult<&str, Outcome> {
    map(alpha1, string_to_outcome)(input)
}

fn threshold(input: &str) -> IResult<&str, Value> {
    map_res(u32, |x| x.try_into())(input)
}

fn string_to_outcome(s: &str) -> Outcome {
    match s {
        "R" => Outcome::REJECT,
        "A" => Outcome::ACCEPT,
        st => Outcome::GOTO(st.to_string()),
    }
}

/* --------------- */
/* Data Structures */
/* --------------- */
#[derive(Clone, Debug)]
struct Workflow {
    name: String,
    instructions: Vec<Instruction>,
}
impl Workflow {
    fn into_pair(self) -> (String, Vec<Instruction>) {
        (self.name, self.instructions)
    }
}

type WorkflowMap = HashMap<String, Vec<Instruction>>;

#[derive(Clone, Debug)]
enum Instruction {
    Conditional(Condition, Outcome),
    Unconditional(Outcome),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Condition {
    characteristic: Characteristic,
    comparator: Comparator,
    threshold: Value,
}
impl Condition {
    fn opposite(&self) -> Self {
        let new_comparator = match self.comparator {
            Comparator::LessThan => Comparator::GreaterThanEq,
            Comparator::LessThanEq => Comparator::GreaterThan,
            Comparator::GreaterThan => Comparator::LessThanEq,
            Comparator::GreaterThanEq => Comparator::LessThan,
        };

        Condition {
            characteristic: self.characteristic,
            comparator: new_comparator,
            threshold: self.threshold,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Outcome {
    GOTO(String),
    ACCEPT,
    REJECT,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Comparator {
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Characteristic {
    X,
    M,
    A,
    S,
}

#[derive(Clone, Debug)]
struct Parts {
    x_range: Range<Value>,
    m_range: Range<Value>,
    a_range: Range<Value>,
    s_range: Range<Value>,
    empty: bool,
}
impl Parts {
    fn size(&self) -> Value {
        (self.x_range.len() * self.m_range.len() * self.a_range.len() * self.s_range.len()) as Value
    }

    fn pare_to_condition(&mut self, condition: &Condition) {
        let range = match condition.characteristic {
            Characteristic::X => &self.x_range,
            Characteristic::M => &self.m_range,
            Characteristic::A => &self.a_range,
            Characteristic::S => &self.s_range,
        };

        let new_range = pare_range(range, condition.comparator, condition.threshold);
        if new_range.is_empty() {
            self.empty = true;
        }

        match condition.characteristic {
            Characteristic::X => {
                self.x_range = new_range;
            }
            Characteristic::M => {
                self.m_range = new_range;
            }
            Characteristic::A => {
                self.a_range = new_range;
            }
            Characteristic::S => {
                self.s_range = new_range;
            }
        }
    }
}

type Value = isize;

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> Value {
    let (_, workflows) = problem_input(input).expect("Failed to parse problem input");
    let workflow_map: WorkflowMap = workflows.into_iter().map(|wf| wf.into_pair()).collect();
    let starting_parts = Parts {
        x_range: 1..4001,
        m_range: 1..4001,
        a_range: 1..4001,
        s_range: 1..4001,
        empty: false,
    };

    acceptance_total(&workflow_map, &starting_parts, "in")
}

fn acceptance_total(workflow_map: &WorkflowMap, parts: &Parts, label: &str) -> Value {
    let mut total = 0;

    // `remaining_parts` tracks the remaining part-space in the current iteration
    let mut remaining_parts = parts.clone();
    let instructions = workflow_map.get(label).unwrap();
    for instruction in instructions.iter() {
        match instruction {
            // When we happen upon a condition, we branch based on its conditions
            Instruction::Conditional(condition, outcome) => {
                let mut positive_parts = remaining_parts.clone();

                // `positive_parts` are the ones where the condition succeeds;
                // these are used for the outcome of the condition, and either
                // branch into another workflow or die immediately
                positive_parts.pare_to_condition(condition);

                // The remaining parts stay in this branch of execution
                remaining_parts.pare_to_condition(&condition.opposite());

                if !positive_parts.empty {
                    match outcome {
                        Outcome::ACCEPT => {
                            total += positive_parts.size();
                        }
                        Outcome::REJECT => {
                            continue;
                        }
                        Outcome::GOTO(label) => {
                            total += acceptance_total(workflow_map, &positive_parts, &label);
                        }
                    }
                }
            }

            // When we reach an unconditional outcome, we use our remaining part-space
            // to either begin a new branch of execution or to just tally results
            Instruction::Unconditional(outcome) => {
                if !remaining_parts.empty {
                    match outcome {
                        Outcome::ACCEPT => {
                            total += remaining_parts.size();
                        }
                        Outcome::REJECT => {
                            continue;
                        }
                        Outcome::GOTO(label) => {
                            total += acceptance_total(workflow_map, &remaining_parts, &label)
                        }
                    }
                }
            }
        }
    }
    return total;
}

fn pare_range(range: &Range<Value>, comparator: Comparator, threshold: Value) -> Range<Value> {
    let (x, y) = (range.start, range.end);

    // If the range contains the threshold then we usually have to actually pare it down
    if range.contains(&threshold) {
        match comparator {
            Comparator::LessThan => x..threshold,
            Comparator::LessThanEq => x..(threshold + 1),
            Comparator::GreaterThan => (threshold + 1)..y,
            Comparator::GreaterThanEq => threshold..y,
        }
    }
    // If it doesn't, then we're definitely returning either the whole thing or nothing.
    // (Note: The distinction between e.g. < and <= doesn't matter if you're outside the range)
    else {
        match comparator {
            Comparator::LessThan | Comparator::LessThanEq => {
                if threshold >= y {
                    range.clone()
                } else {
                    x..x
                }
            }
            Comparator::GreaterThan | Comparator::GreaterThanEq => {
                if threshold < x {
                    range.clone()
                } else {
                    y..y
                }
            }
        }
    }
}
