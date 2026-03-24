use std::{
    sync::{Arc, atomic::AtomicBool, mpsc::Sender},
    time::Instant,
};

use crate::{
    ai::{
        LimitedTime, NoLimit,
        ai::AI,
        evaluation::{Evaluation, PestoEvaluation},
    },
    board::{
        bitboard::Square,
        board::{Board, CastlingRights, Color, Piece, PieceType},
    },
    moves::{
        leapers::KNIGHT_ATK_TABLE,
        move_generator::{MoveGenerator, Undo},
        move_structs::{Move, MoveType},
        sliders::CASTLING_MASK,
        traits::{Black, Castle, PawnDirection, Side, White},
    },
    uci::{
        handler::StopFlag,
        structs::{GoTimeParams, InfoParams, UciOut},
    },
};

#[derive(Clone)]
pub struct Engine {
    board: Board,
    move_gen: MoveGenerator,
    depth: u8,
    tx: Option<Sender<UciOut>>,
}
impl Default for Engine {
    fn default() -> Self {
        MoveGenerator::init_slider_atk_tables();
        Self {
            board: Board::default(),
            move_gen: MoveGenerator::default(),
            depth: 5,
            tx: None,
        }
    }
}
impl Engine {
    pub fn set_board(&mut self, board: Board) {
        self.board = board;
    }
    pub fn set_tx(&mut self, tx: Sender<UciOut>) {
        self.tx = Some(tx);
    }
    pub fn go_time(&mut self, time_data: GoTimeParams, stop: StopFlag) -> Option<Move> {
        let my_time_left = if self.board.side_to_move == Color::White {
            time_data.wtime
        } else {
            time_data.btime
        };
        let my_increment = if self.board.side_to_move == Color::White {
            time_data.winc
        } else {
            time_data.binc
        };
        let mut allocated_time = (my_time_left / 20) + (my_increment / 2); //simple heuristic for the move time

        if allocated_time >= my_time_left {
            allocated_time = my_time_left - 50; // padding for delays
        };
        let mut current_depth = 1;
        let mut aborted = false;
        let mut nodes_searched = 0;
        let mut best_move: Option<Move> = None;
        let time_limit = LimitedTime {
            start: Instant::now(),
            allocated_time,
        };
        let mut ai = AI::new(
            aborted,
            stop,
            time_limit,
            nodes_searched,
            self.move_gen.clone(),
        );
        while !aborted {
            let current_best_move = ai.make_decision(current_depth, &mut self.board, best_move);
            if aborted && best_move.is_some() {
                break; // dont send the move that it aborted on
            }
            if let Some(tx) = &self.tx {
                let pv = ai.pv_table.get_pv(current_depth);
                let uci_params = InfoParams {
                    nodes_searched,
                    pv,
                    curr_depth: current_depth,
                };
                tx.send(UciOut::Info(uci_params)).unwrap();
            }
            best_move = current_best_move;
            current_depth += 1;
        }
        best_move
    }
    pub fn go(&mut self, max_depth: u8, stop: StopFlag) -> Option<Move> {
        let aborted = false;
        let nodes_searched = 0;
        let mut best_move: Option<Move> = None;
        let time_limit = NoLimit;
        let mut ai = AI::new(
            aborted,
            stop,
            time_limit,
            nodes_searched,
            self.move_gen.clone(),
        );
        for current_depth in 1..=max_depth {
            let current_best_move = ai.make_decision(current_depth, &mut self.board, best_move);
            if aborted {
                break;
            }
            if let Some(tx) = &self.tx {
                let pv = ai.pv_table.get_pv(current_depth);
                let uci_params = InfoParams {
                    nodes_searched: ai.nodes_searched(),
                    pv,
                    curr_depth: current_depth,
                };
                tx.send(UciOut::Info(uci_params)).unwrap();
            }
            best_move = current_best_move;
        }
        best_move
    }

    pub fn set_depth(&mut self, depth: u8) {
        self.depth = depth;
    }

    pub fn board(&self) -> Board {
        self.board
    }
}

pub fn make_move_non_generic(board: &mut Board, mv: Move) -> Undo {
    match board.side_to_move {
        Color::White => make_move::<White>(board, mv),
        Color::Black => make_move::<Black>(board, mv),
    }
}
pub fn undo_move_non_generic(board: &mut Board, mv: Move, undo: Undo, color: Color) {
    match color {
        Color::White => undo_move::<White>(mv, board, undo),
        Color::Black => undo_move::<Black>(mv, board, undo),
    };
}
pub fn make_move<S: Side + PawnDirection + Castle + Evaluation>(
    board: &mut Board,
    mv: Move,
) -> Undo {
    let from = mv.from();
    let to = mv.to();
    let move_type = mv.move_type();
    let mut undo_move = Undo {
        castling_rights: board.castling_rights(),
        previous_ep_square: board.en_passant_square(),
        captured_piece: None,
    };
    board.set_en_passant_square(None);
    let color = S::COLOR;
    let mover = board.get_piece_at_square(from).unwrap();
    if move_type.is_capture() {
        let piece_pos = if let MoveType::EnPassant = move_type {
            let pos = S::get_source_single(to);
            pos
        } else {
            to
        };
        let opposite_color = S::Opposite::COLOR;
        let captured = board.get_piece_at_square(piece_pos).unwrap();
        undo_move.captured_piece = Some(captured.piece_type);
        // update evaluation
        board.phase -= PestoEvaluation::PIECE_PHASE_INCR[captured.piece_type as usize]; // - phase of captured
        board.subtract_score::<S::Opposite>(piece_pos, captured); // + value of captured

        let cap_mask = 1u64 << (piece_pos as u8);
        board.occupied_by_color[opposite_color as usize].0 ^= cap_mask;
        board.pieces[opposite_color as usize][captured.piece_type as usize].0 ^= cap_mask;
        board.set_piece_at_square(piece_pos, None);
    }

    // set ep square for dpush
    if let MoveType::DoublePush = move_type {
        board.set_en_passant_square(Some(S::get_source_single(to)));
    }

    let move_mask = (1u64 << from as u8) | (1u64 << to as u8);
    if let Some(promotion) = mv.promotion_to() {
        board.pieces[color as usize][promotion as usize].0 ^= 1u64 << to as u8;
        board.pieces[color as usize][PieceType::Pawn as usize].0 ^= 1u64 << from as u8;
        board.occupied_by_color[color as usize].0 ^= move_mask;
        let new_piece = Piece {
            piece_type: promotion,
            color,
        };
        board.set_piece_at_square(from, None);
        board.set_piece_at_square(to, Some(new_piece));
        // update evaluation
        board.phase += PestoEvaluation::PIECE_PHASE_INCR[promotion as usize]; // + phase of promoted
        board.add_score::<S>(to, new_piece); //+ val of promoted
        board.subtract_score::<S>(from, mover); // - val of old square
    } else {
        let rook = Piece {
            piece_type: PieceType::Rook,
            color,
        };
        match move_type {
            MoveType::QueenSideCastle => {
                let mask = (1u64 << S::QUEEN_SIDE_ROOK_POS as u8)
                    | (1u64 << S::QUEEN_ROOK_START_POS as u8);
                board.pieces[color as usize][PieceType::Rook as usize].0 ^= mask;
                board.occupied_by_color[color as usize].0 ^= mask;
                board.set_piece_at_square(S::QUEEN_ROOK_START_POS, None);
                board.set_piece_at_square(S::QUEEN_SIDE_ROOK_POS, Some(rook));
                board.add_score::<S>(S::QUEEN_SIDE_ROOK_POS, rook); // + val of rook new square
                board.subtract_score::<S>(S::QUEEN_ROOK_START_POS, rook); // - val of starting rook square
            }
            MoveType::KingSideCastle => {
                let mask =
                    (1u64 << S::KING_SIDE_ROOK_POS as u8) | (1u64 << S::KING_ROOK_START_POS as u8);
                board.pieces[color as usize][PieceType::Rook as usize].0 ^= mask;
                board.occupied_by_color[color as usize].0 ^= mask;

                board.set_piece_at_square(S::KING_ROOK_START_POS, None);
                board.set_piece_at_square(S::KING_SIDE_ROOK_POS, Some(rook));
                board.add_score::<S>(S::KING_SIDE_ROOK_POS, rook); // + val of rook new square
                board.subtract_score::<S>(S::KING_ROOK_START_POS, rook); // - val of starting rook square
            }
            _ => {}
        };
        board.pieces[color as usize][mover.piece_type as usize].0 ^= move_mask;
        board.occupied_by_color[color as usize].0 ^= move_mask;
        board.set_piece_at_square(from, None);
        board.set_piece_at_square(to, Some(mover));

        board.subtract_score::<S>(from, mover); // - val of old square
        board.add_score::<S>(to, mover); // + val of new square
    }
    // update castle rights
    board.set_castling_rights(CastlingRights(
        board.castling_rights().0 & (CASTLING_MASK[from as usize] & CASTLING_MASK[to as usize]),
    ));
    undo_move
}
pub fn undo_move<S: Side + Castle + Evaluation>(mv: Move, board: &mut Board, undo: Undo) {
    let color = S::COLOR;
    let from = mv.from();
    let to = mv.to();
    let move_type = mv.move_type();
    let move_mask = (1u64 << from as u8) | (1u64 << to as u8);
    let mover = board.get_piece_at_square(to).expect("has to exist");
    if let Some(promotion) = mv.promotion_to() {
        board.pieces[color as usize][promotion as usize].0 ^= 1u64 << to as u8;
        board.pieces[color as usize][PieceType::Pawn as usize].0 ^= 1u64 << from as u8;
        board.occupied_by_color[color as usize].0 ^= move_mask;
        let new_piece = Piece {
            piece_type: PieceType::Pawn,
            color,
        };
        board.phase -= PestoEvaluation::PIECE_PHASE_INCR[promotion as usize]; // - phase of promoted
        board.add_score::<S>(from, new_piece); //+ val of promoted
        board.subtract_score::<S>(to, mover); // - val of old square
        board.set_piece_at_square(to, None);
        board.set_piece_at_square(from, Some(new_piece));
    } else {
        let rook = Piece {
            piece_type: PieceType::Rook,
            color,
        };
        match move_type {
            MoveType::QueenSideCastle => {
                let mask = (1u64 << S::QUEEN_SIDE_ROOK_POS as u8)
                    | (1u64 << S::QUEEN_ROOK_START_POS as u8);
                board.pieces[color as usize][PieceType::Rook as usize].0 ^= mask;
                board.occupied_by_color[color as usize].0 ^= mask;
                board.set_piece_at_square(S::QUEEN_ROOK_START_POS, Some(rook));
                board.set_piece_at_square(S::QUEEN_SIDE_ROOK_POS, None);
                board.subtract_score::<S>(S::QUEEN_SIDE_ROOK_POS, rook); // + val of rook new square
                board.add_score::<S>(S::QUEEN_ROOK_START_POS, rook); // - val of starting rook square
            }
            MoveType::KingSideCastle => {
                let mask =
                    (1u64 << S::KING_SIDE_ROOK_POS as u8) | (1u64 << S::KING_ROOK_START_POS as u8);
                board.pieces[color as usize][PieceType::Rook as usize].0 ^= mask;
                board.occupied_by_color[color as usize].0 ^= mask;
                board.set_piece_at_square(S::KING_ROOK_START_POS, Some(rook));
                board.set_piece_at_square(S::KING_SIDE_ROOK_POS, None);
                board.subtract_score::<S>(S::KING_SIDE_ROOK_POS, rook); // + val of rook new square
                board.add_score::<S>(S::KING_ROOK_START_POS, rook); // - val of starting rook square
            }
            _ => {}
        };
        board.pieces[color as usize][mover.piece_type as usize].0 ^= move_mask;
        board.occupied_by_color[color as usize].0 ^= move_mask;
        board.set_piece_at_square(from, Some(mover));
        board.set_piece_at_square(to, None);
        board.add_score::<S>(from, mover);
        board.subtract_score::<S>(to, mover);
    }
    if let Some(captured) = undo.captured_piece {
        let opposite_color = S::Opposite::COLOR;
        let captured = Piece {
            piece_type: captured,
            color: opposite_color,
        };
        let piece_pos = if let MoveType::EnPassant = move_type {
            S::get_source_single(to)
        } else {
            to
        };
        let cap_mask = 1u64 << (piece_pos as u8);
        //TODO: fix en passant
        board.occupied_by_color[opposite_color as usize].0 ^= cap_mask;
        board.pieces[opposite_color as usize][captured.piece_type as usize].0 ^= cap_mask;
        board.phase += PestoEvaluation::PIECE_PHASE_INCR[captured.piece_type as usize]; // - phase of captured
        board.add_score::<S::Opposite>(piece_pos, captured);
        board.set_piece_at_square(piece_pos, Some(captured));
    }

    board.set_castling_rights(undo.castling_rights);
    board.set_en_passant_square(undo.previous_ep_square);
}

mod tests {
    use crate::{
        board::{
            bitboard::Square,
            board::{Board, Color, PieceType},
        },
        engine::undo_move,
        moves::{
            move_generator::{MoveGenerator, MoveList},
            move_structs::{Move, MoveType},
            traits::White,
        },
    };

    use super::make_move;
    #[test]
    fn test_make_double_pawn_push() {
        // Setup
        let mut board = Board::default();
        let mv = Move::new(Square::E2, Square::E4, MoveType::DoublePush);

        // Act
        let undo = make_move::<White>(&mut board, mv);

        // Assert
        // E2 should now be empty
        assert!(board.get_piece_at_square(Square::E2).is_none());

        // E4 should have a White Pawn
        let moved_piece = board
            .get_piece_at_square(Square::E4)
            .expect("Piece should be at e4");
        assert_eq!(moved_piece.piece_type, PieceType::Pawn);
        assert_eq!(moved_piece.color, Color::White);
        assert!(
            board.pieces[Color::White as usize][PieceType::Pawn as usize].check_bit(Square::E4)
        );

        assert_eq!(board.en_passant_square(), Some(Square::E3));

        assert_eq!(undo.captured_piece, None);
    }
    #[test]
    fn test_white_kingside_castling() {
        // Setup
        let mut board =
            Board::from_FEN("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1".to_string());
        let mv = Move::new(Square::E1, Square::G1, MoveType::KingSideCastle);
        let original_board = board.clone();

        // Act
        let undo = make_move::<White>(&mut board, mv);

        // Assert
        // King should be on g1
        let king = board.get_piece_at_square(Square::G1).unwrap();
        assert_eq!(king.piece_type, PieceType::King);

        // Rook should be on f1
        let rook = board.get_piece_at_square(Square::F1).unwrap();
        assert_eq!(rook.piece_type, PieceType::Rook);

        // Original squares e1 and h1 should be empty
        assert!(board.get_piece_at_square(Square::E1).is_none());
        assert!(board.get_piece_at_square(Square::H1).is_none());
        undo_move::<White>(mv, &mut board, undo);
        assert_eq!(original_board, board)
    }
    #[test]
    fn test_undo() {
        let mut board = Board::default();
        let original_board = board.clone();
        let mv = Move::new(Square::E2, Square::E4, MoveType::DoublePush);

        // Act
        let undo = make_move::<White>(&mut board, mv);
        undo_move::<White>(mv, &mut board, undo);

        // Assert
        assert_eq!(board, original_board);
    }
}
mod debug_tests {
    use crate::{
        board::board::{Board, Color, PieceType},
        moves::{
            move_generator::{MoveGenerator, MoveList},
            traits::White,
        },
    };

    use super::make_move;

    #[test]
    #[ignore]
    fn test_make_move() {
        let mut board = Board::default();
        let move_generator = MoveGenerator::default();
        let mut move_list = MoveList::default();
        move_generator.generate_quiets::<White>(&mut move_list, &board);
        let moves = move_list.as_slice();
        for mv in moves {
            println!("{mv}");
        }
        make_move::<White>(&mut board, moves[0].mv);
        println!("{}", board.get_pieces(PieceType::Knight, Color::White));
        let mut move_list = MoveList::default();
        move_generator.generate_quiets::<White>(&mut move_list, &board);
        let moves = move_list.as_slice();
        for mv in moves {
            println!("{mv}");
        }
        make_move::<White>(&mut board, moves[5].mv);
        println!("{}", board.get_pieces(PieceType::Knight, Color::White));
        let mut move_list = MoveList::default();
        move_generator.generate_captures::<White>(&mut move_list, &board);
        let moves = move_list.as_slice();
        for mv in moves {
            println!("{mv}");
        }
        make_move::<White>(&mut board, moves[0].mv);
        println!("{}", board.get_pieces(PieceType::Knight, Color::White));
        println!("{}", board.black_occupied());
        println!("{}", board);

        assert!(false)
    }
    #[ignore]
    #[test]
    fn test_castle() {
        let mut board =
            Board::from_FEN("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1".to_string());
        println!("{}", board);
        let move_generator = MoveGenerator::default();
        let mut move_list = MoveList::default();
        move_generator.generate_quiets::<White>(&mut move_list, &board);
        let moves = move_list.as_slice();
        for (i, mv) in moves.iter().enumerate() {
            println!("{i}: {mv}");
        }
        make_move::<White>(&mut board, moves[7].mv);
        let mut move_list = MoveList::default();
        move_generator.generate_quiets::<White>(&mut move_list, &board);
        let moves = move_list.as_slice();
        for (i, mv) in moves.iter().enumerate() {
            println!("{i}: {mv}");
        }
        println!("{}", board);
        assert!(false);
    }
}
