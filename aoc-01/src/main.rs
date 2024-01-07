use std::{collections::VecDeque, fs};

fn main() {
    println!("Let's start solving AOC-01!");
    let input = fs::read_to_string("aoc-01-input").expect("Unable to read file");
    let total = get_num(&input);
    println!("Total: {}", total);
}

fn get_num(txt: &str) -> u32 {
    let mut word_buffer: VecDeque<char> = VecDeque::new();
    let mut first_char: Option<char> = None;
    let mut last_char: Option<char> = None;
    let mut total: u32 = 0;

    let mut print_buffer: String = String::new();

    // Iterate over characters in the sample text
    for c in txt.chars() {
        // Add to the word buffer; we never need to store more than 5 characters
        word_buffer.push_back(c);
        if word_buffer.len() > 5 {
            word_buffer.pop_front();
        }

        print_buffer.push(c);

        // New digit found!
        if c.is_ascii_digit() {
            // first_char == None when we haven't encountered a number yet
            if first_char.is_none() {
                first_char = Some(c);
            }

            // last_char is updated every time we encounter a number
            last_char = Some(c);

            // Clean up the word buffer
            word_buffer.clear();
        }

        // The word buffer spells out the name of a number
        if let Some(c) = read_deque(&word_buffer) {
            if first_char.is_none() {
                first_char = Some(c);
            }
            last_char = Some(c);
            // Note: Do NOT clear the word buffer in this situation; we are not allowed to assume
            // that the names of numbers don't overlap, and this would mess up "twone" for instance
        }

        // Newline found!
        if c == '\n' {
            // Provided we succeeded in getting two digits, add their value to the total
            if let Some(val) = assemble_number(&first_char, &last_char) {
                total += val;
                println!("{} -> {}", print_buffer, val) // debug
            }
            // Reset the first_char and last_char for the next line
            first_char = None;
            last_char = None;
            word_buffer.clear();
            print_buffer.clear(); // debug
        }
    }
    // Input always ends with a newline; if it didn't, we would put end-of-file code here

    return total;
}

fn assemble_number(c1: &Option<char>, c2: &Option<char>) -> Option<u32> {
    let d1 = c1.and_then(|c| c.to_digit(10));
    let d2 = c2.and_then(|c| c.to_digit(10));
    match (d1, d2) {
        (Some(n1), Some(n2)) => Some(n1 * 10 + n2),
        _otherwise => None,
    }
}

fn read_deque(buffer: &VecDeque<char>) -> Option<char> {
    let mut partial: String;
    for i in 0..buffer.len() {
        partial = buffer.range(i..).collect();
        if let Some(c) = read_one(&partial) {
            return Some(c);
        }
    }
    return None;
}

fn read_one(s: &String) -> Option<char> {
    match s.as_str() {
        "one" => Some('1'),
        "two" => Some('2'),
        "three" => Some('3'),
        "four" => Some('4'),
        "five" => Some('5'),
        "six" => Some('6'),
        "seven" => Some('7'),
        "eight" => Some('8'),
        "nine" => Some('9'),
        _ => None,
    }
}
