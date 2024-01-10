# Day 23 (Part 2)
([AoC link](https://adventofcode.com/2023/day/23))
Part 2 basically removes all of the interesting features that made the problem more tractable, so for this I just implemented a depth-first search to test all paths to the exit. The main optimization to this would be incorporating backward paths from the end and linking them up with forward paths to produce legal routes faster.