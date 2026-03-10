use std::f32::MIN;

use crate::{
    bitboard::Square,
    board::{Board, Color},
    engine::{make_move, make_move_non_generic, undo_move, undo_move_non_generic},
    evaluation::Evaluation,
    moves::{
        move_generator::{Move, MoveGenerator, MoveList},
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
        let mut best_mv: Option<Move> = None;
        let mut max = -30000;
        let mut move_list = MoveList::default();
        let alpha = max;
        let beta = -alpha;
        move_generator.generate_moves(&mut move_list, board);
        for mv in move_list.as_slice() {
            let undo = make_move_non_generic(board, *mv, board.side_to_move);
            if !move_generator.is_king_in_check(board, board.side_to_move) {
                let score = match board.side_to_move {
                    Color::White => {
                        -AI::nega_max::<Black>(depth - 1, move_generator, board, -beta, -alpha)
                    }
                    Color::Black => {
                        AI::nega_max::<White>(depth - 1, move_generator, board, alpha, beta)
                    }
                };
                if score > max {
                    max = score;
                    best_mv = Some(*mv);
                };
            }
            undo_move_non_generic(board, *mv, undo, board.side_to_move);
        }
        return best_mv;
    }
    fn nega_max<S: Side + Castle + PawnDirection + Evaluation>(
        depth: u8,
        move_generator: &MoveGenerator,
        board: &mut Board,
        mut alpha: i16,
        beta: i16,
    ) -> i16 {
        if depth == 0 {
            return board.evaluate() * S::MULTIPLIER;
        }
        let mut max = i16::MIN;
        let mut move_list = MoveList::default();
        move_generator.generate_moves_generic::<S>(&mut move_list, board);
        let mut legal_moves = 0;
        for mv in move_list.as_slice() {
            let undo = make_move::<S>(board, *mv);
            if !move_generator.is_king_in_check(board, S::COLOR) {
                let score =
                    -AI::nega_max::<S::Opposite>(depth - 1, move_generator, board, -beta, -alpha);
                if score > max {
                    max = score;
                    if score > alpha {
                        alpha = score;
                    }
                }
                legal_moves += 1;
                if beta <= alpha {
                    undo_move::<S>(*mv, board, undo);
                    break;
                }
            }
            undo_move::<S>(*mv, board, undo);
        }
        if legal_moves == 0 && move_generator.is_king_in_check(board, S::COLOR) {
            return -30000 - depth as i16;
        } else if legal_moves == 0 {
            return 0;
        }
        return max;
    }
}
mod tests {
    use crate::{
        board::{Board, Color},
        engine::make_move,
        moves::{
            move_generator::{MoveGenerator, MoveType},
            traits::{Black, White},
        },
    };

    use super::AI;

    #[test]
    fn nega_max_test() {
        //let mut board = Board::default();
        let mut board = Board::from_FEN("4k3/8/4K3/8/8/8/8/7R w - - 0 1".to_string());
        let mvg = MoveGenerator::default();
        let move_made = AI::make_decision(7, &mvg, &mut board);
        make_move::<White>(&mut board, move_made.unwrap());
        board.side_to_move = board.side_to_move ^ 1;
        println!("{}", move_made.unwrap());
        println!("{}", board);
        assert!(false)
    }
}
