# Day 5 (Part 1)
([AoC link](https://adventofcode.com/2023/day/5))
This is one of many examples where the first part of the problem essentially consists of writing a correct simulation, while the second part contains the meat. Here, each line of input corresponds to a `FunctionPiece` and a map is encoded as a vector of these — a `CompositeFunction`. These can be turned into honest-to-god functions by `composite_fn`. 

I remember struggling here for a while just with the fact that functions are not really first-class objects in Rust (unsurprisingly); probably, I tried to compose the unnamed functions given as output of `composite_fn` in order to apply them to the seeds in one fell swoop. You can see that, instead, I just had `solve_problem` apply them sequentially to the inputs. 

(Note that this has two solutions for Part 2.)