# Advent of Code 2023
## Overview
In this repository, you will find the code I produced in solving [Advent of Code 2023](https://adventofcode.com/2023). Note that, per the request in the [Aoc FAQ](https://adventofcode.com/2023/about), the actual puzzle inputs and text are not provided as part of this repo. As such, if you wish to run the code on something, you will have to retrieve inputs of your own from the Advent of Code website (or whip them up yourself).

## Goals
This year, I treated Advent of Code as an opportunity to improve with Rust, mostly because I viewed it as a very promising language with a lot of desirable features but a high upfront learning cost. Furthermore, I aimed to solve all of the problems without falling back on help or hints from others, just because I thought it would be more rewarding that way. On the other hand, I did consult at times with outside materials pertaining to mathematics and algorithms, which I considered fair game. 

## Takeaways
Well, I had fun and I improved at Rust a lot. I think if you look at the code, you'll find a lot more awkwardness in the earlier solutions — e.g., it took some time before I learned a lot of the ins and outs of the Iterator API, the Entry API for maps, and so on. My comfort with ownership and borrowing semantics also definitely improved over the course of doing this, and that's probably the biggest gain for me going forward in terms of using Rust.

Another wondferful thing is that I learned (and designed?) a number of interesting algorithms in the course of solving these problems. I am particularly fond of Day 12 in this regard, where I dreamt up a non-memoizing polynomial-time reduction of the problem (which can otherwise be solved by dynamic programming) and Day 25, where I learned the Ford-Fulkerson method for computing max-flow between two points in a graph and employed it in a rather interesting way. The power of priority queues also really shone through in Day 22, where they rapidly trivialize any concerns that might arise in the relevant computations. 

The 'non-programming' parts I was a lot less fond of. A couple of the problems essentially required analyzing the given problem input directly instead of employing computer algorithms to produce an answer, and to me this sort of undermines the whole endeavor. I think I would have enjoyed those more if they appeared in a different context rather than being intermixed within what is ostensibly a series of programming exercises. 

## Continued...
Each problem folder contains a README where I have written about solving that problem in particular.
