# Day 24 (Part 1)
([AoC link](https://adventofcode.com/2023/day/24))
This one is kind of interesting; my approach was to notice that we can use the bounding box together with the lines to reduce the problem to counting pairwise intersections of (possibly degenerate) line segments. The `points_of_interest` function performs this association: the idea is just that the rays have to intersect the bounding box to be relevant, and we use either the two points of intersection (if it started outside) or the ray's starting point plus its one intersection (if it started inside).

Collision between the line segments themselves is detected using an orientation-based algorithm. This has a lot of corner-cases, which turned out to be pretty annoying.