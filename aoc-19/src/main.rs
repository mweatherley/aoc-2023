use nom::branch::alt;
use nom::bytes::complete::{tag, take_until1};
use nom::character::complete::{alpha1, anychar, char, newline, u32};
use nom::combinator::{map, map_opt, map_res, opt, value};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, separated_pair, terminated, tuple};
use nom::IResult;
use std::collections::HashMap;
use std::fs;

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

fn problem_input(input: &str) -> IResult<&str, (Vec<Workflow>, Vec<Part>)> {
    let (rest, workflows) = many1(terminated(workflow, newline))(input)?;
    let (rest, _) = newline(rest)?;
    let (rest, parts) = many1(terminated(part, newline))(rest)?;
    Ok((rest, (workflows, parts)))
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

fn part(input: &str) -> IResult<&str, Part> {
    map(
        delimited(
            char('{'),
            tuple((one_value, one_value, one_value, one_value)),
            char('}'),
        ),
        |(x, m, a, s)| Part { x, m, a, s },
    )(input)
}

fn one_value(input: &str) -> IResult<&str, Value> {
    map_res(
        delimited(pair(anychar, tag("=")), u32, opt(char(','))),
        |v| v.try_into(),
    )(input)
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct Condition {
    characteristic: Characteristic,
    comparator: Comparator,
    threshold: Value,
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
    GreaterThan,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Characteristic {
    X,
    M,
    A,
    S,
}

#[derive(Clone, Copy, Debug)]
struct Part {
    x: Value,
    m: Value,
    a: Value,
    s: Value,
}
impl Part {
    fn sum(&self) -> Value {
        self.x + self.m + self.a + self.s
    }
}

type Value = isize;

/* ----- */
/* Logic */
/* ----- */

fn solve_problem(input: &str) -> Value {
    let (_, (workflows, parts)) = problem_input(input).expect("Failed to parse problem input");
    let workflow_map: WorkflowMap = workflows.into_iter().map(|wf| wf.into_pair()).collect();

    let mut total = 0;
    for part in parts.iter() {
        if accepts(&workflow_map, *part, "in") {
            total += part.sum()
        }
    }
    return total;
}

fn accepts(workflow_map: &WorkflowMap, part: Part, start: &str) -> bool {
    let mut current_workflow_name = start.to_string();
    let mut instructions;
    'entire_path: loop {
        instructions = workflow_map.get(&current_workflow_name).unwrap().clone();
        'reading_instructions: for instruction in instructions.into_iter() {
            match follow_instruction(part, instruction) {
                Some(outcome) => match outcome {
                    Outcome::ACCEPT => {
                        return true;
                    }
                    Outcome::REJECT => {
                        return false;
                    }
                    Outcome::GOTO(st) => {
                        current_workflow_name = st;
                        continue 'entire_path;
                    }
                },
                None => {
                    continue 'reading_instructions;
                }
            }
        }
    }
}

// Returns Some<> if the instruction is conclusive for the part, and None
// otherwise, indicating that we should continue reading input from the same workflow
// Right now this consumes the instruction for purposes of ownership
// over the string it might contain.
fn follow_instruction(part: Part, instruction: Instruction) -> Option<Outcome> {
    match instruction {
        Instruction::Conditional(condition, outcome) => {
            if matches_condition(part, condition) {
                Some(outcome)
            } else {
                None
            }
        }
        Instruction::Unconditional(outcome) => Some(outcome),
    }
}

fn matches_condition(part: Part, condition: Condition) -> bool {
    let part_value = match condition.characteristic {
        Characteristic::X => part.x,
        Characteristic::M => part.m,
        Characteristic::A => part.a,
        Characteristic::S => part.s,
    };

    match condition.comparator {
        Comparator::LessThan => part_value < condition.threshold,
        Comparator::GreaterThan => part_value > condition.threshold,
    }
}
