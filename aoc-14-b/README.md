# Day 14 (Part 2)
([AoC link](https://adventofcode.com/2023/day/14))
This time there wasn't really any getting around simulating the rock movements, so I did that. Broadly speaking, the idea here is to simulate until you find a cycle (by hashing the state) and then interpolate to get the state at the time of interest. 

The main thing of interest in terms of approach here is that I made the choice of double-storing the map in `RockMap` in order to expose both the rows and columns easily to iteration. (Of course, this means that they both have to be updated in tandem anyway.) 

Really, the best thing to do in situations like this would be just to store a "grid" as a vector of entries and do arithmetic on indices (with known bounds) to move between rows.