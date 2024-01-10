# Day 14 (Part 1)
([AoC link](https://adventofcode.com/2023/day/14))
This could be accomplished by just running a simulation, but the value of interest is computed just by the knowing how many round rocks are between each barrier (the end of the grid or a '#') and the subsequent one below it, so I just computed that for each column (`column_sum`).