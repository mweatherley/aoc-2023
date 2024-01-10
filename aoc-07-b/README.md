# Day 7 (Part 2)
([AoC link](https://adventofcode.com/2023/day/7))
With the introduction of Jokers, the pattern-matching approach used in the first part would be vastly too laborious, so I decided to actually implement a good function for computing the type of the poker hands instead (`get_hand_type`). The key observations are these:
- Since straights are excluded, the types of these poker hands are determined completely by the highest two multiplicities for cards in the hand. This is in the `match` block at the end of the function:

    match mults_max {
        [5, 0] => HandType::FiveOfAKind,
        [4, 1] => HandType::FourOfAKind,
        [3, 2] => HandType::FullHouse,
        [3, 1] => HandType::ThreeOfAKind,
        [2, 2] => HandType::TwoPair,
        [2, 1] => HandType::OnePair,
        [1, 1] => HandType::HighCard,
        _ => HandType::HighCard,
    }

- The best possible hand is always obtained by using jokers in conjunction with the card of highest multiplicity.