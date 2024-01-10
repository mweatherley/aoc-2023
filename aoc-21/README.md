# Day 21 (Part 1)
([AoC link](https://adventofcode.com/2023/day/21))
This problem was pretty interesting. The main thing to notice is that the parity of the number of steps taken matters a lot: the set of tiles reachable in exactly N steps is exactly the set of tiles reachable in at most N steps and at a distance with the same parity as N. In other words, it's like a checkerboard in that whenever you take a step, the color of the tile changes.

With that in mind, this is basically just a breadth-first search that tracks the number of tiles of the same color as the startig tile reached as it goes.