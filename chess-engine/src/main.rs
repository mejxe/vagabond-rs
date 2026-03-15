use chess_engine::{
    ai::ai::AI,
    board::bitboard::{BitBoard, Square},
    board::board::{Board, PieceType},
    engine::{make_move, undo_move},
    moves::{
        move_generator::{MoveGenerator, MoveList, Occupancy},
        sliders::{
            ROOK_MASK_TABLE, generate_bishop_attacks, generate_bishop_mask, generate_rook_attacks,
        },
        traits::{Black, White},
    },
    performance::{perft_divide, perft_divide_by_move_type, perft_entry},
};

fn main() {
    //let mut board = Board::from_FEN("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string(),);
    //assert_eq!(4865609, perft_divide::<White>(&mut board, 4));
    test_nega_max();
    //let mut board =
    //    Board::from_FEN("rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPP1NnPP/RNBQK2R w KQ - 0 1".to_string());
    //let mvg = MoveGenerator::default();
    //let mut move_list = MoveList::default();
    //mvg.generate_moves(&mut move_list, &board);
    //for mv in move_list.as_slice() {
    //    println!("{mv}");
    //}
    //test_perft();
}
fn test_move_rating() {
    let mut board = Board::from_FEN("4k3/8/8/4N3/8/8/8/4K3 w - - 0 1".to_string());
    println!("{}", board.evaluate());
    let mut board = Board::from_FEN("N3k3/8/8/8/8/8/8/4K3 w - - 0 1".to_string());
    println!("{}", board.evaluate());
    let mut board = Board::from_FEN("q3qkq1/8/8/8/8/8/7R/2K5 w - - 0 1".to_string());
    println!("{}", board.evaluate());
}
fn test_perft() {
    let mut board = Board::from_FEN(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ".to_string(),
    ); // kiwipete
    let mut board = Board::from_FEN("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 ".to_string()); // pos 3 cpw
    let mut board = Board::from_FEN(
        "r1bq2r1/b4pk1/p1pp1p2/1p2pP2/1P2P1PB/3P4/1PPQ2P1/R3K2R w KQ - 0 1".to_string(),
    ); //tricky pos

    perft_divide_by_move_type::<White>(&mut board, 5);
}
fn test_nega_max() {
    //let mut board = Board::from_FEN("7k/8/8/8/8/8/4P3/4K3 w - - 0 1".to_string());
    //let mut board = Board::from_FEN("4k3/4p3/8/8/8/8/8/7K w - - 0 1".to_string());
    //let mut board =
    //   Board::from_FEN("1k1r4/pp1b1R2/3q2pp/4p3/2B5/4Q3/PPP2B2/2K5 b - - 0 1".to_string());
    // let mut board = Board::from_FEN("r3k2r/p1ppqpb1/bn2pnpp/3PN3/1p2P3/2N2Q2/PPPBBPPP/R3K2R w KQkq - 0 1".to_string(),);
    let mut board = Board::from_FEN(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string(),
    );
    println!("{}", board);
    let mvg = MoveGenerator::default();
    for i in 0..1 {
        let move_made = AI::make_decision(7, &mvg, &mut board);
        make_move::<White>(&mut board, move_made.unwrap());
        board.side_to_move = board.side_to_move ^ 1;
        println!("{i}: {}", move_made.unwrap());
        let move_made = AI::make_decision(7, &mvg, &mut board);
        make_move::<Black>(&mut board, move_made.unwrap());
        board.side_to_move = board.side_to_move ^ 1;
        println!("{}: {}", i + 1, move_made.unwrap());
    }
}
