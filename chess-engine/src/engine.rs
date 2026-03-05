use crate::{
    ai::ai::AI,
    bitboard::Square,
    board::{Board, CastlingRights, Color, Piece, PieceType},
    moves::{
        move_generator::{Move, MoveGenerator, MoveType, Promotion, Undo},
        sliders::CASTLING_MASK,
        traits::{Castle, PawnDirection, Side},
    },
};

pub struct Engine {
    board: Board,
    move_gen: MoveGenerator,
    ai: AI,
}
impl Default for Engine {
    fn default() -> Self {
        Self {
            board: Board::default(),
            move_gen: MoveGenerator::default(),
            ai: AI {},
        }
    }
}

pub fn make_move<S: Side + PawnDirection + Castle>(board: &mut Board, mv: Move) -> Undo {
    let from = mv.from();
    let to = mv.to();
    let move_type = mv.move_type();
    let mut undo_move = Undo {
        castling_rights: board.castling_rights(),
        previous_ep_square: board.en_passant_square(),
        captured_piece: None,
    };
    let color = S::COLOR;
    let mover = board.get_piece_at_square(from).expect("has to exist");
    if move_type.is_capture() {
        let piece_pos = if let MoveType::EnPassant = move_type {
            let pos = S::get_source_single(to);
            board.set_piece_at_square(pos, None);
            pos
        } else {
            to
        };
        let opposite_color = S::Opposite::COLOR;
        let captured = board
            .get_piece_at_square(piece_pos)
            .expect("has to exist")
            .piece_type;
        undo_move.captured_piece = Some(captured);
        let cap_mask = 1u64 << (piece_pos as u8);
        board.occupied_by_color[opposite_color as usize].0 ^= cap_mask;
        board.pieces[opposite_color as usize][captured as usize].0 ^= cap_mask;
    }

    // set ep square for dpush
    if let MoveType::DoublePush = move_type {
        board.set_en_passant_square(Some(S::get_source_single(to)));
    } else {
        board.set_en_passant_square(None);
    }

    let move_mask = (1u64 << from as u8) | (1u64 << to as u8);
    if let Some(promotion) = mv.promotion_to() {
        board.pieces[color as usize][promotion as usize].0 ^= 1u64 << to as u8;
        board.pieces[color as usize][PieceType::Pawn as usize].0 ^= 1u64 << from as u8;
        board.occupied_by_color[color as usize].0 ^= move_mask;
        board.set_piece_at_square(from, None);
        board.set_piece_at_square(
            to,
            Some(Piece {
                piece_type: promotion,
                color,
            }),
        );
    } else {
        match move_type {
            MoveType::QueenSideCastle => {
                board.pieces[color as usize][PieceType::Rook as usize].0 ^= (1u64
                    << S::QUEEN_SIDE_ROOK_POS as u8)
                    & (1u64 << S::QUEEN_ROOK_START_POS as u8);
                board.set_piece_at_square(S::QUEEN_ROOK_START_POS, None);
                board.set_piece_at_square(
                    S::QUEEN_SIDE_ROOK_POS,
                    Some(Piece {
                        piece_type: PieceType::Rook,
                        color,
                    }),
                );
            }
            MoveType::KingSideCastle => {
                board.pieces[color as usize][PieceType::Rook as usize].0 ^=
                    (1u64 << S::KING_SIDE_ROOK_POS as u8) & (1u64 << S::KING_ROOK_START_POS as u8);
                board.set_piece_at_square(S::KING_ROOK_START_POS, None);
                board.set_piece_at_square(
                    S::KING_SIDE_ROOK_POS,
                    Some(Piece {
                        piece_type: PieceType::Rook,
                        color,
                    }),
                );
            }
            _ => {}
        };
        board.pieces[color as usize][mover.piece_type as usize].0 ^= move_mask;
        board.occupied_by_color[color as usize].0 ^= move_mask;
        board.set_piece_at_square(from, None);
        board.set_piece_at_square(to, Some(mover));
    }
    // update castle rights
    board.set_castling_rights(CastlingRights(
        board.castling_rights().0 & (CASTLING_MASK[from as usize] & CASTLING_MASK[to as usize]),
    ));
    undo_move
}
pub fn undo_move(mv: Move, board: &Board, undo: Undo) {}

mod tests {
    use crate::{
        bitboard::Square,
        board::{Board, Color, PieceType},
        moves::{
            move_generator::{Move, MoveGenerator, MoveList, MoveType},
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

        // Act
        make_move::<White>(&mut board, mv);

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
    }
}
mod debug_tests {
    use crate::{
        board::{Board, Color, PieceType},
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
        make_move::<White>(&mut board, moves[0]);
        println!("{}", board.get_pieces(PieceType::Knight, Color::White));
        let mut move_list = MoveList::default();
        move_generator.generate_quiets::<White>(&mut move_list, &board);
        let moves = move_list.as_slice();
        for mv in moves {
            println!("{mv}");
        }
        make_move::<White>(&mut board, moves[5]);
        println!("{}", board.get_pieces(PieceType::Knight, Color::White));
        let mut move_list = MoveList::default();
        move_generator.generate_captures::<White>(&mut move_list, &board);
        let moves = move_list.as_slice();
        dbg!(moves.len());
        for mv in moves {
            println!("{mv}");
        }
        make_move::<White>(&mut board, moves[0]);
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
        make_move::<White>(&mut board, moves[7]);
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
