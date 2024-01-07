use std::fs;

use nom::bytes::complete::take_until;
use nom::character::complete::{char, space1, u32};
use nom::multi::{many0, many1};
use nom::sequence::{preceded, terminated};
use nom::IResult;

fn main() {
    println!("Let's solve AOC-04!");
    let input = fs::read_to_string("aoc-04-input.txt").expect("Unable to read file");
    let output = solve_problem(&input);
    println!("Solution: {}", output);
}

pub struct Card {
    winning_numbers: Vec<u32>,
    had_numbers: Vec<u32>,
}

impl Card {
    fn value(&self) -> u32 {
        let mut matches = 0;
        for n in self.winning_numbers.iter() {
            for m in self.had_numbers.iter() {
                if n == m {
                    matches += 1;
                }
            }
        }
        if matches == 0 {
            return 0;
        } else {
            return 2_u32.pow(matches - 1);
        }
    }
}

fn solve_problem(input: &str) -> u32 {
    let (_, cards) = parse_input(input).ok().unwrap();
    return cards.iter().map(|c| c.value()).sum();
}

fn parse_input(input: &str) -> IResult<&str, Vec<Card>> {
    many0(terminated(card, char('\n')))(input)
}

fn card(input: &str) -> IResult<&str, Card> {
    // Strip out the card number
    let (rest, _) = terminated(take_until(":"), char(':'))(input)?;

    // Winning numbers separated by space
    let (rest, winners) = many1(preceded(space1, u32))(rest)?;

    // Separator between winning numbers and the ones that are obtained
    let (rest, _) = preceded(space1, char('|'))(rest)?;

    // Had numbers separated by spaces again
    let (rest, had) = many1(preceded(space1, u32))(rest)?;

    let card = Card {
        winning_numbers: winners,
        had_numbers: had,
    };

    return Ok((rest, card));
}
