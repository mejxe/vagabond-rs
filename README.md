# Overview 
My attempt at making a performant chess engine in Rust.

Still Work In Progress...

# Technical Info
- BitBoards for board representation
- [PeSTO](https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function) for evaluation
- Negamax with Alpha-Beta pruning
- UCI compatible
- No external dependecies
- Roughly estimateed to be around 2k elo

# Goals / What's next
- Transposition Tables
- Some pruning optimizations
- Multi PV
- NNUE (someday, maybe?)

# Acknowledgements/Resources
- https://www.chessprogramming.org/Main_Page - a lot of useful information for FREE
- https://www.youtube.com/@chessprogramming591 - especially the "Bitboard CHESS ENGINE in C" series.
