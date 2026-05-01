# Overview 
My attempt at making a performant chess engine in Rust.
## Why/What for?
- I always wanted to program a machine that would play games (well).
- I always wanted to build something that relies on high performance.
- I like chess.
- It's an interesting topic for a semester project. 
# Technical Info
## Board Representation
- BitBoards
- Mailbox for instant field lookups
## Move generation
- Bitboard based move generator
- PEXT for bishops, rooks and queens (therefore no support for some CPUs)
## Board Evaluation
- Material scores
- [PeSTO](https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function) PST tables
## Move tree traversal
- Negamax with Alpha-Beta pruning
- Principal Variation Search
## Optimizations
- TT's with Zobrist Hashing
- Move Ordering (move type & MVV/LVA)
- Null Move Pruning
- Killer Moves
- Delta Pruning
## Communication/Logging
- UCI compatible
- Info string's (nodes, nps, time, depth, PV)
- MultiPV
## Misc
- No external dependecies
- Above 2k ELO
- PERFT

# Goals / What's next
- NNUE
- LMR

# Acknowledgements/Resources
- https://www.chessprogramming.org/Main_Page - a lot of useful information for FREE
- https://www.youtube.com/@chessprogramming591 - especially the "Bitboard CHESS ENGINE in C" series.
