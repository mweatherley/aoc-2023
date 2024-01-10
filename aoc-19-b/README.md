# Day 19 (Part 2)
([AoC link](https://adventofcode.com/2023/day/19))
Part 2 for this was pretty fun. The point is that each of the conditions defines a hyperplane in xmas-space; in light of Day 5, this is actually pretty nice, since we never need to worry about being left with multiple intervals when we start modifying our given hypercube (e.g. `pare_range` returns a single range).

The main algorithm in `acceptance_total` just follows the hyper-volumes as they move through the workflows and recursively builds up the total volume of accepted part-space.