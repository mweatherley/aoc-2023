# Day 11 (Part 1)
([AoC link](https://adventofcode.com/2023/day/11))
This problem was pretty straightforward, but there is one interesting thing in this solution, which is that the sum of pairwise distances is computed with an O(n) algorithm instead of a naïve O(n^2) one. This is possible because the taxicab distance between two points is decomposable into its vertical and horizontal components, and the sum of pairwise distances of points on a line can be computed using only the associated mass function in time O(n); the idea is to look at how the sum changes as individual points are added.

My algorithm for that is actually a little bit silly, since it didn't occur to me to "build up" rather than to "tear down" — the function `linear_distance_total` takes parameters `total_mass` and `weighted_mass` which would be completely unnecessary if I had just done it the other way. 

We also see that I decided to reuse that totally over-engineered "state machine parser" from Day 10, once again in a context where just parsing individual characters would have done the trick. Interesting.