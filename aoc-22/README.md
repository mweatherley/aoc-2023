# Day 22 (Part 1)
([AoC link](https://adventofcode.com/2023/day/22))
Once again, Part 1 is mostly about providing a correct simulation. The only real decision made here was to use a priority queue to process the blocks' falling, assigning each block a priority associated to its height so that the lowest ones would fall first. 