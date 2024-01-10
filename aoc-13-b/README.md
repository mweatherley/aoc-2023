# Day 13 (Part 2)
([AoC link](https://adventofcode.com/2023/day/13))
To find the almost-reflections, I took an approach very similar to Part 1. The point is that when there is an almost-reflection, it will show up in every row (say) except for one, so we can look for indices that show up in every row except for one.

It turns out that for the given input, that is sufficient, although I wasn't satisfied that it's actually mathematically thorough enough. For instance, I couldn't find a reason that there could not be two such candidates, where one of them is a red herring. To this end, my solution builds an index tracking near-reflections for each row — those indices where only one symbol needs to change to produce a reflection — and uses this to check that candidates are genuine.