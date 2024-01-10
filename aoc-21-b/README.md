# Day 21 (Part 2)
([AoC link](https://adventofcode.com/2023/day/21))
For Part 2 there was a lot of subtlety to handle, even with a number of assumptions; this is a problem with a lot of extra structure in the input. For example:
- The border of the garden pattern is free of rocks (this is true in the example)
- The start is in the center of the garden pattern, which is a square with odd sidelengths (this is true in the example)
- There is a free path to the border in each cardinal direction (untrue in the example)
- There is a "circle" (in the taxicab metric) empty of rocks at fixed distance from the start (untrue in the example)
- There are no long spirals in the interior of the garden pattern (so that nothing is further away from the start than the corners, for example)

The third and fifth assumptions are the most salient to me, since I exploited them implicitly in making my decomposition. Broadly, this is as follows:
- Tiles filled completely within distance N.
- Tiles filled partially within distance N:
    - Tiles along the 'inside' of each edge of the diamond
    - Tiles along the 'outside' of each edge of the diamond
    - Tiles at the points of the diamond, capping off the 'inside' edges
    - Tiles at the points of the diamond, capping off the 'outside' edges

The contribution of the filled tiles (`interior_volume` in `solve_problem`) is computed by computing the filled volume of the garden as a whole and performing some math on the results (there is some parity stuff going on that I am not mentioning).

The contribution for the unfilled tiles also benefits greatly from symmetry, so a similar procedure is applied for each 'kind' of boundary contribution — running a simulation on the unrepeated garden pattern starting at some fixed point and then doing some math to figure out the total contribution from tiles of that 'kind'.