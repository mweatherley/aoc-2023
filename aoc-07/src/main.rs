use regex::Regex;
use std::cmp::Ordering;
use std::fs;

fn main() {
    println!("Let's solve AOC-07!");
    let input = fs::read_to_string("aoc-07-input.txt").expect("Unable to read input");
    let solution = solve_problem(&input);
    println!("Solution: {}", solution);
}

// Data
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum Card {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Hand {
    cards: [Card; 5],
    hand_type: HandType,
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        let hands_cmp = self.hand_type.cmp(&other.hand_type);
        match hands_cmp {
            Ordering::Equal => self.cards.cmp(&other.cards),
            _ => hands_cmp,
        }
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(other));
    }
}

// Parsing
fn parse_input(input: &str) -> Vec<(Hand, u64)> {
    let mut output: Vec<(Hand, u64)> = vec![];
    let re = Regex::new(r"^(?<cards>(A|K|Q|J|T|9|8|7|6|5|4|3|2){5})\s(?<bid>\d+)$").unwrap();
    let lines = input.split('\n');

    for line in lines.into_iter() {
        if line == "" {
            break;
        }
        let caps = re.captures(line).unwrap();

        let cards_str = &caps["cards"];
        let card_vec: Vec<char> = cards_str.chars().collect();
        let card_arr: [char; 5] = card_vec.try_into().ok().unwrap();
        let cards = card_arr.map(|c| char_to_card(&c).unwrap());
        let hand_type = get_hand_type(&cards);

        let bid_str = &caps["bid"];
        let bid: u64 = bid_str.parse().ok().unwrap();

        let hand = Hand {
            cards: cards,
            hand_type: hand_type,
        };
        output.push((hand, bid));
    }

    return output;
}

// Non-parsing functions
fn solve_problem(input: &str) -> u64 {
    let mut hands_and_bids = parse_input(input);
    hands_and_bids.sort_by_key(proj);

    let mut rank = 1;
    let mut total = 0;
    for (_h, v) in hands_and_bids.iter() {
        total += rank * v;
        rank += 1;
    }
    return total;
}

fn get_hand_type(cards: &[Card; 5]) -> HandType {
    let mut cards_sorted = *cards;
    cards_sorted.sort();
    match cards_sorted {
        [a, b, c, d, e] if (a == b) && (b == c) && (c == d) && (d == e) => HandType::FiveOfAKind,
        [a, b, c, d, _] if (a == b) && (b == c) && (c == d) => HandType::FourOfAKind,
        [_, a, b, c, d] if (a == b) && (b == c) && (c == d) => HandType::FourOfAKind,
        [a, b, c, d, e] if (a == b) && (b == c) && (d == e) => HandType::FullHouse,
        [a, b, c, d, e] if (a == b) && (c == d) && (d == e) => HandType::FullHouse,
        [a, b, c, _, _] if (a == b) & (b == c) => HandType::ThreeOfAKind,
        [_, a, b, c, _] if (a == b) & (b == c) => HandType::ThreeOfAKind,
        [_, _, a, b, c] if (a == b) & (b == c) => HandType::ThreeOfAKind,
        [a, b, c, d, _] if (a == b) & (c == d) => HandType::TwoPair,
        [a, b, _, c, d] if (a == b) & (c == d) => HandType::TwoPair,
        [_, a, b, c, d] if (a == b) & (c == d) => HandType::TwoPair,
        [a, b, _, _, _] if (a == b) => HandType::OnePair,
        [_, a, b, _, _] if (a == b) => HandType::OnePair,
        [_, _, a, b, _] if (a == b) => HandType::OnePair,
        [_, _, _, a, b] if (a == b) => HandType::OnePair,
        _ => HandType::HighCard,
    }
}

fn char_to_card(c: &char) -> Option<Card> {
    match c {
        'A' => Some(Card::Ace),
        'K' => Some(Card::King),
        'Q' => Some(Card::Queen),
        'J' => Some(Card::Jack),
        'T' => Some(Card::Ten),
        '9' => Some(Card::Nine),
        '8' => Some(Card::Eight),
        '7' => Some(Card::Seven),
        '6' => Some(Card::Six),
        '5' => Some(Card::Five),
        '4' => Some(Card::Four),
        '3' => Some(Card::Three),
        '2' => Some(Card::Two),
        _ => None,
    }
}

fn proj(vals: &(Hand, u64)) -> Hand {
    let (h, _) = vals;
    return *h;
}
