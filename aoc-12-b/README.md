# Day 12 (Part 2)
([AoC link](https://adventofcode.com/2023/day/12))
In my solution for Part 2, I employed a completely different algorithm, taking the dual approach of looking at all the ways of dropping the blocks into place among the symbols. 

To do this I started by listing, for each block, the legal locations within the token string where it could be placed *a priori*, taking into account only the local symbols and some naïve bounds based on the total length of the blocks (`generate_location_sets`). 

Then, for each pair of adjacent blocks, I generated a matrix with entries corresponding to pairs of locations, indicating (with a 1 or 0) whether the two placements could legally coincide (`allowance_matrix`, `causality_matrices`). This is determined by their spacing and whether or not they leave over a '#' between them.

The number of legal placements is then computed by the sum of the entries of the product matrix of all of these (`total_solutions`). 