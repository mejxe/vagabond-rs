use chess_engine::{
    ai::ai::AI,
    bitboard::{BitBoard, Square},
    board::{Board, PieceType},
    engine::{make_move, undo_move},
    moves::{
        move_generator::{MoveGenerator, MoveList, Occupancy},
        sliders::{
            ROOK_MASK_TABLE, generate_bishop_attacks, generate_bishop_mask, generate_rook_attacks,
        },
        traits::{Black, White},
    },
    performance::perft_entry,
};

fn main() {
    //let mut board = Board::default();
    //let mv = Move::new(Square::E2, Square::E4, MoveType::Quiet);
    //make_move::<White>(&mut board, mv);
    //let mv = Move::new(Square::D7, Square::D5, MoveType::DoublePush);
    //make_move::<Black>(&mut board, mv);
    //let mv = Move::new(Square::E1, Square::E2, MoveType::Quiet);
    //make_move::<White>(&mut board, mv);
    //assert_eq!(8902, perft_entry(&mut board, 3));
    //assert_eq!(197281, perft_entry(&mut board, 4));
    //assert_eq!(4865609, perft_divide::<White>(&mut board, 5));
    //assert_eq!(4865609, perft_divide::<White>(&mut board, 5));
    //dbg!(119060324, perft_entry(&mut board, 6));
    //test_move_rating();
    test_nega_max();
}
fn test_move_rating() {
    let mut board = Board::from_FEN("4k3/8/8/4N3/8/8/8/4K3 w - - 0 1".to_string());
    println!("{}", board.evaluate());
    let mut board = Board::from_FEN("N3k3/8/8/8/8/8/8/4K3 w - - 0 1".to_string());
    println!("{}", board.evaluate());
    let mut board = Board::from_FEN("q3qkq1/8/8/8/8/8/7R/2K5 w - - 0 1".to_string());
    println!("{}", board.evaluate());
}
fn test_nega_max() {
    //let mut board = Board::from_FEN("4k3/8/4K3/8/8/8/8/7R w - - 0 1".to_string());
    let mut board =
        Board::from_FEN("1k1r4/pp1b1R2/3q2pp/4p3/2B5/4Q3/PPP2B2/2K5 b - - 0 1".to_string());
    let mut board = Board::from_FEN(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string(),
    );
    println!("{}", board);
    let mvg = MoveGenerator::default();
    let move_made = AI::make_decision(6, &mvg, &mut board);
    //make_move::<White>(&mut board, move_made.unwrap());
    board.side_to_move = board.side_to_move ^ 1;
    println!("{}", move_made.unwrap());
}
