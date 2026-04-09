use crate::{
    ai::evaluation::Evaluation,
    board::{
        bitboard::Square,
        board::{Board, Color, PieceType},
    },
    engine::{make_move, undo_move},
    moves::{
        move_generator::{MoveGenerator, MoveList},
        move_structs::{Move, MoveType},
        sliders::{BISHOP_MASK_TABLE, ROOK_MASK_TABLE},
        traits::{Black, Castle, PawnDirection, Side, White},
    },
    performance::perft_entry,
};
#[test]
fn test_move_generation_perft_starting_board() {
    let mut board = Board::default();
    let mut board_2 =
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string());
    println!("{board_2}");
    //let mv = Move::new(Square::E2, Square::E4, MoveType::Quiet);
    //make_move::<White>(&mut board, mv);
    //let mv = Move::new(Square::D7, Square::D5, MoveType::DoublePush);
    //make_move::<Black>(&mut board, mv);
    //let mv = Move::new(Square::E1, Square::E2, MoveType::Quiet);
    //make_move::<White>(&mut board, mv);
    println!("{board}");
    //assert_eq!(8902, perft_entry(&mut board, 3));
    //assert_eq!(197281, perft_entry(&mut board, 4));
    //assert_eq!(4865609, perft_divide::<White>(&mut board, 5));
    //assert_eq!(4865609, perft_divide::<White>(&mut board, 5));
    assert_eq!(119060324, perft_entry(&mut board, 6));
}
#[test]
fn test_is_king_in_check() {
    let move_generator = MoveGenerator::default();
    let board = Board::from_fen("4k3/8/8/8/8/8/8/4K2r w - - 0 1".to_string());
    let king_sq = Square::from_u8_unchecked(
        board
            .get_pieces(PieceType::King, Color::White)
            .0
            .trailing_zeros() as u8,
    );
    let is_attacked = move_generator.is_square_attacked::<White>(&board, king_sq);
    println!("{}", board);
    assert!(is_attacked);
    let board = Board::from_fen("8/8/8/8/8/2n5/8/1K6 w - - 0 1".to_string());
    let king_sq = Square::from_u8_unchecked(
        board
            .get_pieces(PieceType::King, Color::White)
            .0
            .trailing_zeros() as u8,
    );
    let is_attacked = move_generator.is_square_attacked::<White>(&board, king_sq);
    assert!(is_attacked);
    let board = Board::from_fen("8/6b1/8/8/3K4/8/8/8 w - - 0 1".to_string());
    let king_sq = Square::from_u8_unchecked(
        board
            .get_pieces(PieceType::King, Color::White)
            .0
            .trailing_zeros() as u8,
    );
    let is_attacked = move_generator.is_square_attacked::<White>(&board, king_sq);
    assert!(is_attacked);
    let board = Board::from_fen("8/8/8/q3K3/8/8/8/8 w - - 0 1".to_string());
    let king_sq = Square::from_u8_unchecked(
        board
            .get_pieces(PieceType::King, Color::White)
            .0
            .trailing_zeros() as u8,
    );
    let is_attacked = move_generator.is_square_attacked::<White>(&board, king_sq);
    assert!(is_attacked);
    let board = Board::from_fen("8/8/8/q1P1K3/8/8/8/8 w - - 0 1".to_string());
    let king_sq = Square::from_u8_unchecked(
        board
            .get_pieces(PieceType::King, Color::White)
            .0
            .trailing_zeros() as u8,
    );
    let is_attacked = move_generator.is_square_attacked::<White>(&board, king_sq);
    assert!(!is_attacked);
}
#[test]
fn test_move_constructor() {
    let test_move = Move::new(Square::B1, Square::C1, MoveType::Capture);
    let expected_move = Move(0b0001000010000001); // 0001 - capture, 000010 - 3rd square (c1), 000001 - 2nd square (b1)
    println!("{:0>16b}", test_move.0);
    assert!(test_move == expected_move)
}
#[test]
fn test_starting_position_move_count() {
    // Setup
    let board = Board::default();
    let move_generator = MoveGenerator::default();
    let mut move_list = MoveList::default();

    // Act
    move_generator.generate_quiets::<White>(&mut move_list, &board);
    move_generator.generate_captures::<White>(&mut move_list, &board);

    // Assert
    assert_eq!(
        move_list.as_slice().len(),
        20,
        "Starting position should have exactly 20 moves"
    );
}
