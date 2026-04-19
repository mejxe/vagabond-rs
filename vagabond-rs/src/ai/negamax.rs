use std::{
    char::MAX,
    f32::MIN,
    i16,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use crate::{
    ai::evaluation::Evaluation,
    board::{
        bitboard::Square,
        board::{Board, Color, Piece, PieceType},
    },
    engine::{make_move, make_move_non_generic, undo_move, undo_move_non_generic},
    format_with_commas,
    moves::{
        move_generator::{self, MoveGenerator, MoveList},
        move_structs::{ExtMove, Move, MoveType},
        traits::{Black, Castle, PawnDirection, Side, White},
    },
    tt::transposition_table::{NodeType, TT, TTEntry},
    uci::handler::StopFlag,
};

use super::{TimeLimit, evaluation::PestoEvaluation};

pub const MAX_SEARCH_DEPTH: usize = 64;

#[derive(Clone)]
pub struct AI<T: TimeLimit> {
    aborted: bool,
    stop: StopFlag,
    killer_moves: [[Option<Move>; 2]; 64],
    time_limit: T,
    nodes_searched: u32,
    move_generator: MoveGenerator,
    tt: Arc<Mutex<TT>>,
    pv_array: PvArray,
}
impl<T: TimeLimit> AI<T> {
    pub fn new(
        aborted: bool,
        stop: StopFlag,
        time_limit: T,
        nodes_searched: u32,
        move_generator: MoveGenerator,
        tt: Arc<Mutex<TT>>,
    ) -> Self {
        Self {
            aborted,
            stop,
            killer_moves: [[None; 2]; 64],
            time_limit,
            nodes_searched,
            move_generator,
            tt,
            pv_array: PvArray::default(),
        }
    }
    pub fn reset_fields(&mut self) {
        self.killer_moves = [[None; 2]; 64];
        self.nodes_searched = 0;
        self.time_limit.restart()
    }
    pub fn make_decision(&mut self, current_depth: u8, board: &mut Board) -> (Option<Move>, i16) {
        // init variables
        let mut best_mv: Option<Move> = None;
        let mut max = -32000;
        let mut alpha = max; // lower bound of our search window - the score that we are guaranteed
        let beta = -alpha; // higher -||- - the score that opponent is guaranteed, we can't surpass it or opponent won't allow us the move
        let ply = 0;
        self.pv_array.init_node(0);

        // spawn the move fetcher
        let mut move_list = MoveList::default();

        // lock the tt's mutex (single threaded so good enough for now)
        let passdown_tt = Arc::clone(&self.tt);
        let mut tt = passdown_tt.lock().unwrap();
        let mut tt_move: Option<Move> = None;

        // get the tt move at root
        if let Some(entry) = tt.get(board.zobrist) {
            tt_move = Some(entry.best_move());
        }

        // generate unsorted pseudo legal moves
        self.move_generator.generate_moves(&mut move_list, board);

        // if there is a 3-fold draw at root return a random move
        if board.is_draw(3) {
            self.aborted = true;
            //return (Some(move_list.moves.first().unwrap().mv), 0);

            return (None, 0);
        }
        // score moves
        move_list.score_moves(board, 0, &self.killer_moves, tt_move);
        // find the best move in the array [i..n], put it at ith position and increment i
        let moves = move_list.move_fetcher();
        for mv in moves {
            // break if stop flag is on
            if current_depth > 1 && (self.stop.load(Ordering::Relaxed) || self.aborted) {
                self.aborted = true;
                break;
            };
            // make move
            let undo = make_move_non_generic(board, mv.mv);
            self.nodes_searched += 1;
            if !self
                .move_generator
                .is_king_in_check(board, board.side_to_move)
            {
                // if not in check score the position recursively

                /*
                our guaranteed score so far becomes enemy's beta
                enemy's guaranteed score so far becomes enemies alpha

                We negate the score & bounds - the function returns the score from the opponents perspective
                which is opposite to ours.
                 */
                let score = match board.side_to_move {
                    Color::White => -self.negamax::<Black>(
                        current_depth - 1,
                        board,
                        -beta,
                        -alpha,
                        ply + 1,
                        &mut tt,
                        mv,
                        current_depth,
                    ),
                    Color::Black => -self.negamax::<White>(
                        current_depth - 1,
                        board,
                        -beta,
                        -alpha,
                        ply + 1,
                        &mut tt,
                        mv,
                        current_depth,
                    ),
                };

                if score > max {
                    max = score;
                    best_mv = Some(mv.mv);
                    self.pv_array.propagate(ply, mv.mv);
                };
                alpha = i16::max(alpha, max);
            }

            undo_move_non_generic(board, mv.mv, undo);
        }
        if max != 0 {
            if let Some(mv) = best_mv {
                tt.put(TTEntry::new(
                    mv,
                    NodeType::Exact,
                    board.zobrist,
                    current_depth,
                    max,
                ));
            }
        }
        return (best_mv, max);
    }
    fn negamax<S: Side + Castle + PawnDirection + Evaluation>(
        &mut self,
        mut depth: u8,
        board: &mut Board,
        mut alpha: i16,
        beta: i16,
        ply: u8,
        tt: &mut TT,
        root_move: ExtMove,
        max_depth: u8,
    ) -> i16 {
        let mut best_score = -32000;
        let mut best_move: Option<Move> = None;
        self.pv_array.init_node(ply);

        let mut null_window_search = false;
        // early stop conditions
        if self.aborted {
            return 0;
        }
        if self.nodes_searched % 2048 == 0 {
            if self.stop.load(Ordering::Relaxed) || self.time_limit.should_stop() {
                self.aborted = true;
                return 0;
            };
        };
        if board.is_draw(3) {
            self.pv_array.init_node(ply);
            return 0;
        }

        let mut tt_move: Option<Move> = None;
        let mut tt_node_bound = NodeType::Upperbound;

        // probe tt (only if the move is not a repeat to avoid cycles)
        if let Some(entry) = tt.get(board.zobrist) {
            if depth <= entry.depth() {
                if *entry.node_type() == NodeType::Exact && !(beta > alpha + 1) {
                    // search ended on this move and it's score was in the <alpha; beta> window, the move is a pv variation
                    return entry.score();
                } else if *entry.node_type() == NodeType::Lowerbound && entry.score() >= beta {
                    // score was better than opponent would allow, if it's still better then dont search it
                    return entry.score();
                } else if *entry.node_type() == NodeType::Upperbound && entry.score() <= alpha {
                    // score was worse than guaranteed score, if it's still worse then dont search it
                    return entry.score();
                }
            }
            // save the move otherwise, it is likely to tighthen the alpha beta window
            tt_move = Some(entry.best_move());
        }

        if depth == 0 {
            if self.move_generator.is_king_in_check(board, S::COLOR) {
                // if we are currently in a check we cannot start qs,
                // find a way to escape it by deepening the search by 1 ply
                depth += 1;
            } else {
                // if not then it's safe to start qs
                return self.quiescence_search::<S>(board, alpha, beta, ply);
            }
        }
        // null move pruning
        if ply > 0 && depth >= 3 && !self.move_generator.is_king_in_check(board, S::COLOR) {
            let null_undo = board.make_null_move();
            let nmp_score = -self.negamax::<S::Opposite>(
                depth - 3,
                board,
                -beta,
                -beta + 1,
                ply + 1,
                tt,
                root_move,
                max_depth,
            );
            board.undo_null_move(null_undo);
            if nmp_score >= beta {
                return beta;
            }
        }

        let mut move_list = MoveList::default();

        self.move_generator
            .generate_moves_generic::<S>(&mut move_list, board);
        move_list.score_moves(board, ply, &self.killer_moves, tt_move);
        let mut legal_moves = 0;
        let moves = move_list.move_fetcher();
        for mv in moves {
            self.nodes_searched += 1;
            let undo = make_move::<S>(board, mv.mv);
            if !self.move_generator.is_king_in_check(board, S::COLOR) {
                legal_moves += 1;
                let mut score;
                if null_window_search {
                    /* PVS search - we search with a null window to force cut off's
                     *
                     * opponent's search is bound by [-alpha - 1; -alpha],
                     * if it finds a better move than our alpha it fails high and we prune it
                     * if it doesn't find a better move than -alpha-1 it fails low
                     * if it fails low we need to search it with normal window as it may be a valid move
                     */
                    score = -self.negamax::<S::Opposite>(
                        depth - 1,
                        board,
                        -alpha - 1, // this creates the [alpha, alpha+1] window,
                        -alpha,
                        ply + 1,
                        tt,
                        mv,
                        max_depth,
                    );
                    if score > alpha && score < beta {
                        // failed low, research if it doesn't fail high in the current search
                        score = -self.negamax::<S::Opposite>(
                            depth - 1,
                            board,
                            -beta,
                            -alpha,
                            ply + 1,
                            tt,
                            mv,
                            max_depth,
                        );
                    }
                } else {
                    score = -self.negamax::<S::Opposite>(
                        depth - 1,
                        board,
                        -beta,
                        -alpha,
                        ply + 1,
                        tt,
                        mv,
                        max_depth,
                    );
                }
                if score > best_score {
                    best_score = score;
                    best_move = Some(mv.mv);
                }

                if score >= beta {
                    // score is better than opponent's best forced move, so fail high
                    if !mv.mv.move_type().is_capture() {
                        // if a move causes a beta cut off in this branch, save it as prioritized move for sister branches at this depth
                        if self.killer_moves[ply as usize][0] != Some(mv.mv) {
                            self.killer_moves[ply as usize][1] = self.killer_moves[ply as usize][0];
                            self.killer_moves[ply as usize][0] = Some(mv.mv);
                        }
                    }
                    undo_move::<S>(mv.mv, board, undo);
                    tt.put(TTEntry::new(
                        mv.mv,
                        NodeType::Lowerbound,
                        board.zobrist,
                        depth,
                        score,
                    ));
                    return score;
                }
                null_window_search = true; // first move that doesn't fail high turns on the pvs
            }
            if best_score > alpha {
                //the move raised the alpha bound so it's a potential pv move
                alpha = best_score;
                tt_node_bound = NodeType::Exact;
                if ply < max_depth {
                    self.pv_array.propagate(ply, best_move.unwrap());
                }
            }
            undo_move::<S>(mv.mv, board, undo);
        }
        // checkmate check
        if legal_moves == 0 && self.move_generator.is_king_in_check(board, S::COLOR) {
            best_score = -30000 + ply as i16;

        // stalemate check
        } else if legal_moves == 0 {
            best_score = 0;
        }
        if let Some(best_move) = best_move {
            tt.put(TTEntry::new(
                best_move,
                tt_node_bound,
                board.zobrist,
                depth,
                best_score,
            ));
        }
        return best_score;
    }
    fn quiescence_search<S: Side + Castle + PawnDirection + Evaluation>(
        &mut self,
        board: &mut Board,
        mut alpha: i16,
        beta: i16,
        ply: u8,
    ) -> i16 {
        /*
         qs search searches the position until no captures are possible,
         ensuring that there are no surprises in the considered line
        */
        self.pv_array.init_node(ply);
        if self.aborted {
            return 0;
        }
        if self.nodes_searched % 2048 == 0 {
            if self.stop.load(Ordering::Relaxed) || self.time_limit.should_stop() {
                // never stop depth_wise
                self.aborted = true;
                return 0;
            }
        };

        // get current score
        let static_score = board.evaluate() * S::MULTIPLIER;

        // if the score before making moves is higher than opponent's best option,
        // we stop the search for the branch
        if static_score >= beta {
            return static_score;
        }
        if alpha < static_score {
            alpha = static_score;
        }
        let mut best_score = static_score;
        let mut move_list = MoveList::default();
        self.move_generator
            .generate_captures::<S>(&mut move_list, board);
        move_list.score_captures(board);

        let moves = move_list.move_fetcher();
        for mv in moves {
            let victim_piece = board
                .get_piece_at_square(mv.mv.to())
                .unwrap_or_else(|| Piece {
                    piece_type: PieceType::Pawn,
                    color: Color::White,
                })
                .piece_type;
            let victim_value = board.evaluate_piece(victim_piece) as i16;

            // delta pruning - if the score gained by capturing is not enought to raise the alpha it's not worth it
            let safety_margin = 200;
            if static_score + victim_value + safety_margin < alpha {
                continue;
            }

            self.nodes_searched += 1;
            let undo = make_move::<S>(board, mv.mv);
            if !self.move_generator.is_king_in_check(board, S::COLOR) {
                let score = -self.quiescence_search::<S::Opposite>(board, -beta, -alpha, ply + 1);
                if score > best_score {
                    best_score = score;
                }

                if score >= beta {
                    undo_move::<S>(mv.mv, board, undo);
                    return score;
                }
                if best_score > alpha {
                    alpha = best_score;
                }
            }
            undo_move::<S>(mv.mv, board, undo);
        }
        return best_score;
    }

    pub fn nodes_searched(&self) -> u32 {
        self.nodes_searched
    }

    pub fn aborted(&self) -> bool {
        self.aborted
    }

    pub fn stop(&self) -> &AtomicBool {
        &self.stop
    }

    pub fn pv_array(&self) -> &PvArray {
        &self.pv_array
    }
}
#[derive(Debug, Clone)]
pub struct PvArray {
    pv: [[Option<Move>; MAX_SEARCH_DEPTH]; MAX_SEARCH_DEPTH],
    pv_length: [usize; MAX_SEARCH_DEPTH],
}
impl PvArray {
    pub fn propagate(&mut self, ply: u8, new_move: Move) {
        let ply = ply as usize;
        self.pv[ply][0] = Some(new_move);
        let child_len = self.pv_length[ply + 1];
        for i in 0..child_len {
            self.pv[ply][i + 1] = self.pv[ply + 1][i];
        }
        self.pv_length[ply] = child_len + 1;
    }
    pub fn get_pv(&self) -> Vec<Option<Move>> {
        let pv_len = self.pv_length[0];
        self.pv[0][0..pv_len as usize].to_vec()
    }
    pub fn init_node(&mut self, ply: u8) {
        self.pv_length[ply as usize] = 0;
    }
}
impl Default for PvArray {
    fn default() -> Self {
        PvArray {
            pv: [[None; MAX_SEARCH_DEPTH]; MAX_SEARCH_DEPTH],
            pv_length: [0; MAX_SEARCH_DEPTH],
        }
    }
}

mod tests;
