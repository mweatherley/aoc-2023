# Day 25
([AoC link](https://adventofcode.com/2023/day/25))
This problem is interesting. I decided to use a sort of homespun cut-finding algorithm. Since we know that there is a unique 3-edge-cut of the graph by assumption, we also know from the max-flow min-cut theorem that two nodes are on the same side of that 3-edge-cut if and only if they have a max-flow between them exceeding 3.

With that in mind, I used the Ford-Fulkerson method (together with DFS) to classify the nodes as being between the two components (by comparing the max-flow between them and some arbitrarily chosen starting node). This has relatively nice characteristics — normally, a method like this has worst-case runtime proportional to E|f*| for each node, but here we can just terminate after we have found a max-flow of value more than 3, which means it's O(E) for each node. 

The most obvious improvement would just be to use the fact that we already know the 3-cut as soon as we've found any two nodes in different components (since that 3-cut also separates the source and sink nodes of the residual graph in the Ford-Fulkerson method), searching each component for its constituent nodes directly after that point.