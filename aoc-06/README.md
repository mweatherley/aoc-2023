# Day 6 (Part 1)
([AoC link](https://adventofcode.com/2023/day/6))
The problem basically just asks you to write a function for finding the number of combinations `(n,m)` such that `n + m = T` and `n * M > D`. I just do a linear search for the smallest `n` such that `n * (T - n) > D` and then compute the answer directly from that.