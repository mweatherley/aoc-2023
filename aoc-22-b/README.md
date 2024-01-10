# Day 22 (Part 2)
([AoC link](https://adventofcode.com/2023/day/22))
This was pretty interesting to think about! The main wrinkle here in my view is that disintegrating one block can, for instance, lead to two blocks falling, while those two blocks both support a third block which would not fall on its own were only one of them removed.

Once again, a priority queue (now in `reaction_length`) is a handy tool for managing behavior like this, since height is such a powerful invariant in this problem: we know whether or not a block will fall if we have already processed all of the relevant blocks beneath it. 