use crate::{
    ai::ai::AI,
    bitboard::Square,
    board::{Board, Color, Piece, PieceType},
    moves::{
        move_generator::{Move, MoveGenerator, MoveType, Promotion, Undo},
        traits::{PawnDirection, Side},
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

pub fn make_move<S: Side + PawnDirection>(board: &mut Board, mv: Move) -> Undo {
    // TODO: Check and update castling rights, add castle move
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

    if let MoveType::DoublePush = mv.move_type() {
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
        board.pieces[color as usize][mover.piece_type as usize].0 ^= move_mask;
        board.occupied_by_color[color as usize].0 ^= move_mask;
        board.set_piece_at_square(from, None);
        board.set_piece_at_square(to, Some(mover));
    }
    undo_move
}
mod tests {
    use crate::{
        board::{Board, Color, PieceType},
        moves::{
            move_generator::{MoveGenerator, MoveList},
            traits::White,
        },
    };

    use super::make_move;

    #[test]
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
}
