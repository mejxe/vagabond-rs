use std::{char::MAX, f32::MIN, i16, sync::atomic::Ordering, time::Instant};

use crate::{
    ai::evaluation::Evaluation,
    board::{
        bitboard::Square,
        board::{Board, Color, Piece, PieceType},
    },
    engine::{make_move, make_move_non_generic, undo_move, undo_move_non_generic},
    format_with_commas,
    moves::{
        move_generator::{MoveGenerator, MoveList},
        move_structs::{ExtMove, Move},
        traits::{Black, Castle, PawnDirection, Side, White},
    },
    uci::handler::StopFlag,
};

use super::{TimeLimit, evaluation::PestoEvaluation};

#[derive(Clone)]
pub struct AI<T: TimeLimit> {
    aborted: bool,
    stop: StopFlag,
    killer_moves: [[Option<Move>; 2]; 64],
    time_limit: T,
    nodes_searched: u32,
    move_generator: MoveGenerator,
    pub pv_table: PvArray,
}
impl<T: TimeLimit> AI<T> {
    pub fn new(
        aborted: bool,
        stop: StopFlag,
        time_limit: T,
        nodes_searched: u32,
        move_generator: MoveGenerator,
    ) -> Self {
        Self {
            aborted,
            stop,
            killer_moves: [[None; 2]; 64],
            time_limit,
            nodes_searched,
            move_generator,
            pv_table: PvArray::new(),
        }
    }
    pub fn reset_fields(&mut self) {
        self.killer_moves = [[None; 2]; 64];
        self.nodes_searched = 0;
        self.time_limit.restart()
    }
    pub fn make_decision(
        &mut self,
        current_depth: u8,
        board: &mut Board,
        pv: Option<Move>,
    ) -> (Option<Move>, i16) {
        let mut best_mv: Option<Move> = pv;
        // reset
        let mut max = -31000;
        let mut move_list = MoveList::default();
        let mut alpha = max;
        let beta = -alpha;
        self.move_generator.generate_moves(&mut move_list, board);
        move_list.score_moves(board, 0, &self.killer_moves, &self.pv_table);
        let moves = move_list.move_fetcher();
        for mv in moves {
            if current_depth > 1 && self.stop.load(Ordering::Relaxed) {
                break;
            };
            let undo = make_move_non_generic(board, mv.mv);
            self.nodes_searched += 1;
            if !self
                .move_generator
                .is_king_in_check(board, board.side_to_move)
            {
                let score = match board.side_to_move {
                    Color::White => -self.nega_max::<Black>(
                        current_depth - 1,
                        board,
                        -beta,
                        -alpha,
                        current_depth,
                    ),
                    Color::Black => -self.nega_max::<White>(
                        current_depth - 1,
                        board,
                        -beta,
                        -alpha,
                        current_depth,
                    ),
                };

                if score > max {
                    max = score;
                    best_mv = Some(mv.mv);
                    self.pv_table.put_pv(0, mv.mv);
                };
                alpha = i16::max(alpha, max);
            }
            undo_move_non_generic(board, mv.mv, undo, board.side_to_move);
        }
        return (best_mv, max);
    }
    fn nega_max<S: Side + Castle + PawnDirection + Evaluation>(
        &mut self,
        depth: u8,
        board: &mut Board,
        mut alpha: i16,
        beta: i16,
        max_depth: u8,
    ) -> i16 {
        if self.aborted {
            return 0;
        }
        if self.nodes_searched % 2048 == 0 {
            if self.stop.load(Ordering::Relaxed) || self.time_limit.should_stop() {
                self.aborted = true;
                return 0;
            };
        };
        if depth == 0 {
            return self.quiescence_search::<S>(board, alpha, beta);
            //return board.evaluate() * S::MULTIPLIER;
        }
        let mut best_score = i16::MIN;
        let ply = max_depth - depth;

        let mut move_list = MoveList::default();

        self.move_generator
            .generate_moves_generic::<S>(&mut move_list, board);
        move_list.score_moves(board, ply, &self.killer_moves, &self.pv_table);
        let mut legal_moves = 0;
        let moves = move_list.move_fetcher();
        for mv in moves {
            self.nodes_searched += 1;
            let undo = make_move::<S>(board, mv.mv);
            if !self.move_generator.is_king_in_check(board, S::COLOR) {
                legal_moves += 1;
                let score =
                    -self.nega_max::<S::Opposite>(depth - 1, board, -beta, -alpha, max_depth);
                if score > best_score {
                    best_score = score;
                }

                if score >= beta {
                    if !mv.mv.move_type().is_capture() {
                        if self.killer_moves[ply as usize][0] != Some(mv.mv) {
                            self.killer_moves[ply as usize][1] = self.killer_moves[ply as usize][0];
                            self.killer_moves[ply as usize][0] = Some(mv.mv);
                        }
                    }
                    undo_move::<S>(mv.mv, board, undo);
                    return best_score;
                }
                if best_score > alpha {
                    alpha = best_score;
                    self.pv_table.put_pv(ply, mv.mv);
                }
            }
            undo_move::<S>(mv.mv, board, undo);
        }
        if legal_moves == 0 && self.move_generator.is_king_in_check(board, S::COLOR) {
            return -30000 + ply as i16;
        } else if legal_moves == 0 {
            return 0;
        }
        return best_score;
    }
    fn quiescence_search<S: Side + Castle + PawnDirection + Evaluation>(
        &mut self,
        board: &mut Board,
        mut alpha: i16,
        beta: i16,
    ) -> i16 {
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

        let static_score = board.evaluate() * S::MULTIPLIER;

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

            // delta pruning
            let safety_margin = 200;
            if static_score + victim_value + safety_margin < alpha {
                continue;
            }

            self.nodes_searched += 1;
            let undo = make_move::<S>(board, mv.mv);
            if !self.move_generator.is_king_in_check(board, S::COLOR) {
                let score = -self.quiescence_search::<S::Opposite>(board, -beta, -alpha);
                if score > best_score {
                    best_score = score;
                }

                if score >= beta {
                    undo_move::<S>(mv.mv, board, undo);
                    return beta;
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
}
const MAX_DEPTH_NUM: u8 = 15;
#[derive(Clone, Copy, Debug)]
pub struct PvArray {
    moves: [Option<Move>; ((MAX_DEPTH_NUM * MAX_DEPTH_NUM + MAX_DEPTH_NUM) / 2) as usize],
}
impl PvArray {
    pub fn new() -> PvArray {
        PvArray {
            moves: [None; ((MAX_DEPTH_NUM * MAX_DEPTH_NUM + MAX_DEPTH_NUM) / 2) as usize],
        }
    }
    pub fn get_pv_index(ply: u8) -> usize {
        let index = MAX_DEPTH_NUM * ply - (ply * (ply.saturating_sub(1))) / 2;
        index as usize
    }
    pub fn put_pv(&mut self, ply: u8, mv: Move) {
        let current_index = PvArray::get_pv_index(ply);
        self.moves[current_index] = Some(mv);
        let next_index = PvArray::get_pv_index(ply + 1);
        for i in 0..(MAX_DEPTH_NUM - ply - 1) {
            self.moves[current_index + 1 + i as usize] = self.moves[next_index + i as usize];
        }
    }
    pub fn get_pv(&self) -> &[Option<Move>] {
        // TODO: Rewrite to an array of max_depth_num
        let index = {
            let mut i = 0;
            for _ in 0..MAX_DEPTH_NUM {
                if self.moves[i as usize].is_none() {
                    break;
                };
                i += 1;
            }
            i
        };
        &self.moves[0..index as usize]
    }
    #[inline(always)]
    pub fn get_move(&self, index: usize) -> Option<Move> {
        self.moves[index]
    }
}

mod tests {
    use std::sync::{Arc, atomic::AtomicBool};

    use crate::{
        ai::NoLimit,
        board::{
            bitboard::Square,
            board::{Board, Color},
        },
        engine::make_move,
        moves::{
            move_generator::MoveGenerator,
            move_structs::Move,
            traits::{Black, White},
        },
    };

    use super::AI;

    #[test]
    fn nega_max_test() {
        //let mut board = Board::default();
        let mut board = Board::from_FEN(
            "r1bq2r1/b4pk1/p1pp1p2/1p2pP2/1P2P1PB/3P4/1PPQ2P1/R3K2R w KQ - 0 1".to_string(),
        );
        let mvg = MoveGenerator::default();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let nodes = 0;
        let time_limit = NoLimit;
        let mut ai = AI::new(false, stop_flag, time_limit, nodes, mvg);
        let (move_made, _) = ai.make_decision(2, &mut board, None);
        make_move::<White>(&mut board, move_made.unwrap());
        board.side_to_move = board.side_to_move ^ 1;
        println!("{:?}", move_made.unwrap());
        println!("{}", board);
        assert!(false)
    }
    #[test]
    fn nega_max_mate_test() {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let nodes = 0;
        let mut board = Board::from_FEN(
            "r1bq2r1/b4pk1/p1pp1p2/1p2pP2/1P2P1PB/3P4/1PPQ2P1/R3K2R w KQ - 0 1".to_string(),
        );
        println! {"{}", board};
        let mvg = MoveGenerator::default();
        let time_limit = NoLimit;
        let mut ai = AI::new(false, stop_flag, time_limit, nodes, mvg.clone());
        let (move_made, _) = ai.make_decision(4, &mut board, None);
        println!("{:?}", move_made.unwrap());
        make_move::<White>(&mut board, move_made.unwrap());
        println! {"{}", board};
        board.side_to_move = board.side_to_move ^ 1;
        ai.reset_fields();
        let (move_made, _) = ai.make_decision(4, &mut board, None);
        println!("{:?}", move_made.unwrap());
        make_move::<Black>(&mut board, move_made.unwrap());
        println! {"{}", board};
        board.side_to_move = board.side_to_move ^ 1;
        ai.reset_fields();
        let (move_made, _) = ai.make_decision(4, &mut board, None);
        make_move::<White>(&mut board, move_made.unwrap());
        println!("{:?}", move_made.unwrap());
        println! {"{}", board};
        assert!(false)
    }
}
