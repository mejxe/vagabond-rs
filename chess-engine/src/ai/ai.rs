use std::f32::MIN;

use crate::{
    ai::evaluation::Evaluation,
    board::{
        bitboard::Square,
        board::{Board, Color},
    },
    engine::{make_move, make_move_non_generic, undo_move, undo_move_non_generic},
    moves::{
        move_generator::{MoveGenerator, MoveList},
        move_structs::{ExtMove, Move},
        traits::{Black, Castle, PawnDirection, Side, White},
    },
};

pub struct AI {}
impl AI {
    pub fn make_decision(
        depth: u8,
        move_generator: &MoveGenerator,
        board: &mut Board,
    ) -> Option<Move> {
        let mut nodes_searched = 0;
        let mut killer_moves: [[Option<Move>; 2]; 64] = [[None; 2]; 64];
        let mut best_mv: Option<Move> = None;
        let mut max = -31000;
        let mut move_list = MoveList::default();
        let mut alpha = max - 1000;
        let beta = -alpha;
        move_generator.generate_moves(&mut move_list, board);
        move_list.score_moves(board, 0, &killer_moves);
        let moves = move_list.move_fetcher();
        for mv in moves {
            let undo = make_move_non_generic(board, mv.mv, board.side_to_move);
            if !move_generator.is_king_in_check(board, board.side_to_move) {
                let score = match board.side_to_move {
                    Color::White => -AI::nega_max::<Black>(
                        depth - 1,
                        move_generator,
                        board,
                        -beta,
                        -alpha,
                        &mut nodes_searched,
                        &mut killer_moves,
                        depth,
                    ),
                    Color::Black => -AI::nega_max::<White>(
                        depth - 1,
                        move_generator,
                        board,
                        -beta,
                        -alpha,
                        &mut nodes_searched,
                        &mut killer_moves,
                        depth,
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
        println!("nodes searched: {nodes_searched}");
        return best_mv;
    }
    fn nega_max<S: Side + Castle + PawnDirection + Evaluation>(
        depth: u8,
        move_generator: &MoveGenerator,
        board: &mut Board,
        mut alpha: i16,
        beta: i16,
        nodes_searched: &mut u32,
        killer_moves: &mut [[Option<Move>; 2]; 64],
        max_depth: u8,
    ) -> i16 {
        if depth == 0 {
            return board.evaluate() * S::MULTIPLIER;
        }
        let mut max = i16::MIN;
        let ply = max_depth - depth;

        let mut move_list = MoveList::default();

        move_generator.generate_moves_generic::<S>(&mut move_list, board);
        move_list.score_moves(board, ply, killer_moves);
        let mut legal_moves = 0;
        let moves = move_list.move_fetcher();
        for mv in moves {
            *nodes_searched += 1;
            let undo = make_move::<S>(board, mv.mv);
            if !move_generator.is_king_in_check(board, S::COLOR) {
                legal_moves += 1;
                let score = -AI::nega_max::<S::Opposite>(
                    depth - 1,
                    move_generator,
                    board,
                    -beta,
                    -alpha,
                    nodes_searched,
                    killer_moves,
                    max_depth,
                );

                if score > max {
                    max = score;
                }
                if max >= beta {
                    if !mv.mv.move_type().is_capture() {
                        if killer_moves[ply as usize][0] != Some(mv.mv) {
                            killer_moves[ply as usize][1] = killer_moves[ply as usize][0];
                            killer_moves[ply as usize][0] = Some(mv.mv);
                        }
                    }
                    undo_move::<S>(mv.mv, board, undo);
                    return max;
                }
                alpha = i16::max(alpha, max);
            }
            undo_move::<S>(mv.mv, board, undo);
        }
        if legal_moves == 0 && move_generator.is_king_in_check(board, S::COLOR) {
            return -30000 + depth as i16;
        } else if legal_moves == 0 {
            return 0;
        }
        return max;
    }
    fn quiescence_search() {
        todo!();
    }
}
mod tests {
    use crate::{
        board::bitboard::Square,
        board::board::{Board, Color},
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
        let move_made = AI::make_decision(2, &mvg, &mut board);
        make_move::<White>(&mut board, move_made.unwrap());
        board.side_to_move = board.side_to_move ^ 1;
        println!("{}", move_made.unwrap());
        println!("{}", board);
        assert!(false)
    }
    #[test]
    fn nega_max_mate_test() {
        let mut board = Board::from_FEN(
            "r1bq2r1/b4pk1/p1pp1p2/1p2pP2/1P2P1PB/3P4/1PPQ2P1/R3K2R w KQ - 0 1".to_string(),
        );
        println! {"{}", board};
        let mvg = MoveGenerator::default();
        let move_made = AI::make_decision(4, &mvg, &mut board);
        println!("{}", move_made.unwrap());
        make_move::<White>(&mut board, move_made.unwrap());
        println! {"{}", board};
        board.side_to_move = board.side_to_move ^ 1;
        let move_made = AI::make_decision(4, &mvg, &mut board);
        println!("{}", move_made.unwrap());
        make_move::<Black>(&mut board, move_made.unwrap());
        println! {"{}", board};
        board.side_to_move = board.side_to_move ^ 1;
        let move_made = AI::make_decision(4, &mvg, &mut board);
        make_move::<White>(&mut board, move_made.unwrap());
        println!("{}", move_made.unwrap());
        println! {"{}", board};
        assert!(false)
    }
}
