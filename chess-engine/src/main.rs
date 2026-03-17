use chess_engine::{
    ai::ai::AI,
    board::board::Board,
    engine::make_move,
    moves::{
        move_generator::MoveGenerator,
        traits::{Black, White},
    },
    performance::{perft_divide, perft_divide_by_move_type, perft_entry},
};

fn main() {
    test_nega_max();
    //test_perft();
}
fn test_perft() {
    let mut board = Board::from_FEN(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ".to_string(),
    ); // kiwipete
    let mut board = Board::from_FEN("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 ".to_string()); // pos 3 cpw
    let mut board = Board::from_FEN(
        "r1bq2r1/b4pk1/p1pp1p2/1p2pP2/1P2P1PB/3P4/1PPQ2P1/R3K2R w KQ - 0 1".to_string(),
    ); //tricky pos
    let mut board = Board::default();

    //perft_divide_by_move_type::<White>(&mut board, 5);
    perft_entry(&mut board, 7);
}
fn test_nega_max() {
    let mut board = Board::from_FEN(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string(),
    );
    println!("{}", board);
    let mvg = MoveGenerator::default();
    for i in 0..5 {
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
