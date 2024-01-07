use std::cell::Cell;
use std::collections::BTreeMap;
use std::fs;

use nom::bytes::complete::tag;
use nom::character::complete::{char, newline, space0, space1, u32};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::IResult;

fn main() {
    println!("Let's solve AOC-04!");
    let input = fs::read_to_string("aoc-04-input.txt").expect("Unable to read file");
    let output = solve_problem(&input);
    println!("Solution: {}", output);
}

pub struct Card {
    no: u32,
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
        return matches;
    }

    fn flatten(self) -> CardAbstract {
        let vals = TreeVals {
            value: self.value(),
            count: 1,
        };
        return (self.no, Cell::new(vals));
    }
}

// CardAbstract is used when we know we don't care about the actual numbers any more
// The first u32 is the number, the second is the value
type CardAbstract = (u32, Cell<TreeVals>);

#[derive(Debug, Copy, Clone)]
pub struct TreeVals {
    value: u32,
    count: u32,
}

fn solve_problem(input: &str) -> u32 {
    let (_, cards) = parse_input(input).ok().unwrap();
    let card_map: BTreeMap<u32, Cell<TreeVals>> = cards.into_iter().map(|c| c.flatten()).collect();
    let mut total_cards = 0;
    for (k, v) in card_map.iter() {
        println!("{:?}: {:?}", k, v);
        total_cards += v.get().count;
        let cards_won = v.get().value;
        for offset in 1..(cards_won + 1) {
            let target_key = k + offset;
            if let Some(vals) = card_map.get(&target_key) {
                let mut new_vals = vals.get();
                new_vals.count += v.get().count;
                vals.set(new_vals);
            }
        }
    }
    return total_cards;
}

fn parse_input(input: &str) -> IResult<&str, Vec<Card>> {
    many0(terminated(card, newline))(input)
}

fn card(input: &str) -> IResult<&str, Card> {
    // Extract the card number
    let (rest, card_no) = delimited(pair(tag("Card"), space0), u32, char(':'))(input)?;

    // Winning numbers separated by space
    let (rest, winners) = many1(preceded(space1, u32))(rest)?;

    // Separator between winning numbers and the ones that are obtained
    let (rest, _) = preceded(space1, char('|'))(rest)?;

    // Had numbers separated by spaces again
    let (rest, had) = many1(preceded(space1, u32))(rest)?;

    let card = Card {
        no: card_no,
        winning_numbers: winners,
        had_numbers: had,
    };

    return Ok((rest, card));
}
