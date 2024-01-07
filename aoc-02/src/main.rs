use std::cmp::max;
use std::fs;
use std::ops::Add;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{newline, space0, space1, u32};
use nom::combinator::value;
use nom::multi::{fold_many0, many1, many_m_n};
use nom::sequence::{delimited, pair, separated_pair, terminated};
use nom::IResult;

type Rgb = (u32, u32, u32);
const LIMIT_CUBES: Rgb = (12, 13, 14);

#[derive(Copy, Clone)]
enum Color {
    Red,
    Green,
    Blue,
}

fn incorp(rgb: Rgb, blocks: &(u32, Color)) -> Rgb {
    let (mut r, mut g, mut b) = rgb;
    match blocks {
        (v, Color::Red) => {
            r += v;
        }
        (v, Color::Green) => {
            g += v;
        }
        (v, Color::Blue) => {
            b += v;
        }
    }
    return (r, g, b);
}

fn sup(rgb1: Rgb, rgb2: Rgb) -> Rgb {
    let (r1, g1, b1) = rgb1;
    let (r2, g2, b2) = rgb2;
    return (max(r1, r2), max(g1, g2), max(b1, b2));
}

fn is_possible(rgb: &Rgb) -> bool {
    let (r, g, b) = rgb;
    let (R, G, B) = LIMIT_CUBES;
    return r <= &R && g <= &G && b <= &B;
}

fn power(rgb: &Rgb) -> u32 {
    let (r, g, b) = rgb;
    return r * g * b;
}

fn main() {
    println!("Let's solve AOC-02!");
    let input = fs::read_to_string("aoc-02-input").expect("Unable to read input");
    let (_, output) = many_lines(&input).unwrap();
    println!("Game total: {}", output)
}

// Parse many lines, adding together the results
fn many_lines(input: &str) -> IResult<&str, u32> {
    fold_many0(one_line, || 0, Add::add)(input)
}

// Parses one line, returning the power of the minimum cube set
fn one_line(input: &str) -> IResult<&str, u32> {
    let (rest, (_n, rgbs)) = terminated(pair(game_header, several_tests), newline)(input)?;
    let min_cubes = rgbs.into_iter().fold((0, 0, 0), sup);
    return Ok((rest, power(&min_cubes)));
}

// Old version for first part of the problem; instead, just checks
// whether a game is possible
fn _one_line(input: &str) -> IResult<&str, u32> {
    let (rest, (n, rgbs)) = terminated(pair(game_header, several_tests), newline)(input)?;
    if rgbs.iter().all(is_possible) {
        Ok((rest, n))
    } else {
        Ok((rest, 0))
    }
}

fn game_header(input: &str) -> IResult<&str, u32> {
    delimited(tag("Game "), u32, tag(":"))(input)
}

fn several_tests(input: &str) -> IResult<&str, Vec<Rgb>> {
    many1(several_block_exprs)(input)
}

fn several_block_exprs(input: &str) -> IResult<&str, Rgb> {
    let (rest, outputs) = terminated(
        many_m_n(1, 3, block_expr_wrapped),
        after_several_block_exprs,
    )(input)?;
    let output = outputs.iter().fold((0, 0, 0), incorp);
    return Ok((rest, output));
}

// " 5 red," or " 5 red" -> (5, Color::Red) etc.
fn block_expr_wrapped(input: &str) -> IResult<&str, (u32, Color)> {
    delimited(space0, block_expr, after_block_expr)(input)
}

// "5 red" -> (5, Color::Red) etc.
fn block_expr(input: &str) -> IResult<&str, (u32, Color)> {
    separated_pair(u32, space1, color_word)(input)
}

// zero or more commas
fn after_block_expr(input: &str) -> IResult<&str, &str> {
    take_while(|c| c == ',')(input)
}

// zero or more semicolons
fn after_several_block_exprs(input: &str) -> IResult<&str, &str> {
    take_while(|c| c == ';')(input)
}

// "red" -> Color::Red etc.
fn color_word(input: &str) -> IResult<&str, Color> {
    alt((
        value(Color::Red, tag("red")),
        value(Color::Green, tag("green")),
        value(Color::Blue, tag("blue")),
    ))(input)
}
