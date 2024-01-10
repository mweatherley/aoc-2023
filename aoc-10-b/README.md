# Day 10 (Part 2)
([AoC link](https://adventofcode.com/2023/day/10))
Part 2 is one of the more interesting parts of Advent of Code for this year, since there are so many different approaches one can take to finding the area inside of the curve — for instance, one could do some kind of flood-fill or apply the shoestring formula in combination with Pick's theorem. 

I chose to do something quite close to a flood-fill, coloring in tiles on each side of the pipe as it is traversed (in `run_pipe`, using the logic of `paint`) and using these to seed expanding regions that color in the entirety of the area enclosed by the pipe. At the end, I use a ray-casting algorithm (in `is_in_loop`) to determine which of the two colors is actually the one inside the pipe loop.