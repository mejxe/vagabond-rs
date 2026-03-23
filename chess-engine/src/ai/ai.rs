use std::{f32::MIN, i16, sync::atomic::Ordering, time::Instant};

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

use super::evaluation::PestoEvaluation;

#[derive(Clone)]
pub struct AI;
impl AI {
    pub fn make_decision<T: TimeLimit>(
        current_depth: u8,
        move_generator: &MoveGenerator,
        board: &mut Board,
        stop: &StopFlag,
        pv: Option<Move>,
        aborted: &mut bool,
        nodes_searched: &mut u32,
        time_limit: &T,
    ) -> Option<Move> {
        let mut best_mv: Option<Move> = pv;
        // reset
        let mut killer_moves: [[Option<Move>; 2]; 64] = [[None; 2]; 64];
        let mut max = -31000;
        let mut move_list = MoveList::default();
        let mut alpha = max;
        let beta = -alpha;
        move_generator.generate_moves(&mut move_list, board);
        move_list.score_moves(board, 0, &killer_moves, best_mv);
        let moves = move_list.move_fetcher();
        for mv in moves {
            if stop.load(Ordering::Relaxed) {
                break;
            };
            let undo = make_move_non_generic(board, mv.mv);
            if !move_generator.is_king_in_check(board, board.side_to_move) {
                let score = match board.side_to_move {
                    Color::White => -AI::nega_max::<Black, T>(
                        current_depth - 1,
                        move_generator,
                        board,
                        -beta,
                        -alpha,
                        nodes_searched,
                        &mut killer_moves,
                        current_depth,
                        &stop,
                        aborted,
                        time_limit,
                    ),
                    Color::Black => -AI::nega_max::<White, T>(
                        current_depth - 1,
                        move_generator,
                        board,
                        -beta,
                        -alpha,
                        nodes_searched,
                        &mut killer_moves,
                        current_depth,
                        &stop,
                        aborted,
                        time_limit,
                    ),
                };

                if score > max {
                    max = score;
                    best_mv = Some(mv.mv);
                };
                alpha = i16::max(alpha, max);
            }
            undo_move_non_generic(board, mv.mv, undo, board.side_to_move);
        }
        return best_mv;
    }
    fn nega_max<S: Side + Castle + PawnDirection + Evaluation, T: TimeLimit>(
        depth: u8,
        move_generator: &MoveGenerator,
        board: &mut Board,
        mut alpha: i16,
        beta: i16,
        nodes_searched: &mut u32,
        killer_moves: &mut [[Option<Move>; 2]; 64],
        max_depth: u8,
        stop: &StopFlag,
        aborted: &mut bool,
        search_limit: &T,
    ) -> i16 {
        if *aborted {
            return 0;
        }
        if *nodes_searched % 2048 == 0 {
            if stop.load(Ordering::Relaxed) || search_limit.should_stop() {
                *aborted = true;
                return 0;
            };
        };
        if depth == 0 {
            return AI::quiescence_search::<S, T>(
                move_generator,
                board,
                alpha,
                beta,
                nodes_searched,
                stop,
                aborted,
                search_limit,
            );
            //return board.evaluate() * S::MULTIPLIER;
        }
        let mut best_score = i16::MIN;
        let ply = max_depth - depth;

        let mut move_list = MoveList::default();

        move_generator.generate_moves_generic::<S>(&mut move_list, board);
        move_list.score_moves(board, ply, killer_moves, None);
        let mut legal_moves = 0;
        let moves = move_list.move_fetcher();
        for mv in moves {
            *nodes_searched += 1;
            let undo = make_move::<S>(board, mv.mv);
            if !move_generator.is_king_in_check(board, S::COLOR) {
                legal_moves += 1;
                let score = -AI::nega_max::<S::Opposite, T>(
                    depth - 1,
                    move_generator,
                    board,
                    -beta,
                    -alpha,
                    nodes_searched,
                    killer_moves,
                    max_depth,
                    stop,
                    aborted,
                    search_limit,
                );
                if score > best_score {
                    best_score = score;
                }

                if score >= beta {
                    if !mv.mv.move_type().is_capture() {
                        if killer_moves[ply as usize][0] != Some(mv.mv) {
                            killer_moves[ply as usize][1] = killer_moves[ply as usize][0];
                            killer_moves[ply as usize][0] = Some(mv.mv);
                        }
                    }
                    undo_move::<S>(mv.mv, board, undo);
                    return best_score;
                }
                if best_score > alpha {
                    alpha = best_score;
                }
            }
            undo_move::<S>(mv.mv, board, undo);
        }
        if legal_moves == 0 && move_generator.is_king_in_check(board, S::COLOR) {
            return -30000 + ply as i16;
        } else if legal_moves == 0 {
            return 0;
        }
        return best_score;
    }
    fn quiescence_search<S: Side + Castle + PawnDirection + Evaluation, T: TimeLimit>(
        move_generator: &MoveGenerator,
        board: &mut Board,
        mut alpha: i16,
        beta: i16,
        nodes_searched: &mut u32,
        stop: &StopFlag,
        aborted: &mut bool,
        search_limit: &T,
    ) -> i16 {
        if *aborted {
            return 0;
        }
        let static_score = board.evaluate() * S::MULTIPLIER;
        if *nodes_searched % 2048 == 0 {
            if stop.load(Ordering::Relaxed) || search_limit.should_stop() {
                // never stop depth_wise
                *aborted = true;
                return 0;
            }
        };

        if static_score >= beta {
            return static_score;
        }
        if alpha < static_score {
            alpha = static_score;
        }
        let mut best_score = static_score;
        let mut move_list = MoveList::default();
        move_generator.generate_captures::<S>(&mut move_list, board);
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
            let victim_value = PestoEvaluation::EG_MATERIAL_VAL[victim_piece as usize];

            // delta pruning
            let safety_margin = 200;
            if static_score + victim_value + safety_margin < alpha {
                continue;
            }

            *nodes_searched += 1;
            let undo = make_move::<S>(board, mv.mv);
            if !move_generator.is_king_in_check(board, S::COLOR) {
                let score = -AI::quiescence_search::<S::Opposite, T>(
                    move_generator,
                    board,
                    -beta,
                    -alpha,
                    nodes_searched,
                    stop,
                    aborted,
                    search_limit,
                );
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
}
pub trait TimeLimit {
    fn should_stop(&self) -> bool;
}
pub struct NoLimit;
impl TimeLimit for NoLimit {
    fn should_stop(&self) -> bool {
        false
    }
}
pub struct LimitedTime {
    pub start: Instant,
    pub allocated_time: u128,
}
impl TimeLimit for LimitedTime {
    fn should_stop(&self) -> bool {
        self.start.elapsed().as_millis() > self.allocated_time
    }
}

mod tests {
    use std::sync::{Arc, atomic::AtomicBool};

    use crate::{
        ai::ai::NoLimit,
        board::{
            bitboard::Square,
            board::{Board, Color},
        },
        engine::make_move,
        moves::{
            move_generator::MoveGenerator,
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
        let mut nodes = 0;
        let time_limit = NoLimit;
        let move_made = AI::make_decision(
            2,
            &mvg,
            &mut board,
            &stop_flag,
            None,
            &mut false,
            &mut nodes,
            &time_limit,
        );
        make_move::<White>(&mut board, move_made.unwrap());
        board.side_to_move = board.side_to_move ^ 1;
        println!("{:?}", move_made.unwrap());
        println!("{}", board);
        assert!(false)
    }
    #[test]
    fn nega_max_mate_test() {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let mut nodes = 0;
        let mut board = Board::from_FEN(
            "r1bq2r1/b4pk1/p1pp1p2/1p2pP2/1P2P1PB/3P4/1PPQ2P1/R3K2R w KQ - 0 1".to_string(),
        );
        println! {"{}", board};
        let mvg = MoveGenerator::default();
        let time_limit = NoLimit;
        let move_made = AI::make_decision(
            4,
            &mvg,
            &mut board,
            &stop_flag,
            None,
            &mut false,
            &mut nodes,
            &time_limit,
        );
        println!("{:?}", move_made.unwrap());
        make_move::<White>(&mut board, move_made.unwrap());
        println! {"{}", board};
        board.side_to_move = board.side_to_move ^ 1;
        let move_made = AI::make_decision(
            4,
            &mvg,
            &mut board,
            &stop_flag,
            None,
            &mut false,
            &mut nodes,
            &time_limit,
        );
        println!("{:?}", move_made.unwrap());
        make_move::<Black>(&mut board, move_made.unwrap());
        println! {"{}", board};
        board.side_to_move = board.side_to_move ^ 1;
        let move_made = AI::make_decision(
            4,
            &mvg,
            &mut board,
            &stop_flag,
            None,
            &mut false,
            &mut nodes,
            &time_limit,
        );
        make_move::<White>(&mut board, move_made.unwrap());
        println!("{:?}", move_made.unwrap());
        println! {"{}", board};
        assert!(false)
    }
}
