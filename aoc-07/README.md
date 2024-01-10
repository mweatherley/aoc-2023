# Day 7 (Part 1)
([AoC link](https://adventofcode.com/2023/day/7))
All the meat of this solution is really in implementing the hand-order described, which is like the order of hands in poker, except that ties between hands of the same type are resolved by looking at the constituent cards in lexicographic order. This is in the `Order` implementation for `Hand`.

For the first part, I decided that I would just decide hand types by pattern matching (`get_hand_type`), which is awful. The approach in Part 2 is a lot more natural (and much more closely matches how people think about hands, probably).