use std::sync::{Arc, Mutex, atomic::AtomicBool};

use crate::{
    ai::NoLimit,
    board::{
        bitboard::Square,
        board::{Board, Color},
    },
    engine::make_move,
    moves::{
        move_generator::MoveGenerator,
        move_structs::Move,
        traits::{Black, White},
    },
    tt::transposition_table::TT,
};

use super::AI;

#[test]
fn nega_max_test() {
    //let mut board = Board::default();
    let mut board = Board::from_fen(
        "r1bq2r1/b4pk1/p1pp1p2/1p2pP2/1P2P1PB/3P4/1PPQ2P1/R3K2R w KQ - 0 1".to_string(),
    );
    let mvg = MoveGenerator::default();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let nodes = 0;
    let time_limit = NoLimit;
    let mut tt = TT::default();
    let mut tt = Arc::new(Mutex::new(TT::default()));
    let mut ai = AI::new(false, stop_flag, time_limit, nodes, mvg, tt);
    let (move_made, _) = ai.make_decision(2, &mut board, None);
    make_move::<White>(&mut board, move_made.unwrap());
    board.side_to_move = board.side_to_move ^ 1;
    println!("{:?}", move_made.unwrap());
    println!("{}", board);
    assert!(false)
}
#[test]
fn nega_max_mate_test() {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let nodes = 0;
    let mut board = Board::from_fen(
        "r1bq2r1/b4pk1/p1pp1p2/1p2pP2/1P2P1PB/3P4/1PPQ2P1/R3K2R w KQ - 0 1".to_string(),
    );
    println! {"{}", board};
    let mvg = MoveGenerator::default();
    let time_limit = NoLimit;
    let mut tt = Arc::new(Mutex::new(TT::default()));
    let mut ai = AI::new(false, stop_flag, time_limit, nodes, mvg.clone(), tt);
    let (move_made, _) = ai.make_decision(4, &mut board, None);
    println!("{:?}", move_made.unwrap());
    make_move::<White>(&mut board, move_made.unwrap());
    println! {"{}", board};
    board.side_to_move = board.side_to_move ^ 1;
    ai.reset_fields();
    let (move_made, _) = ai.make_decision(4, &mut board, None);
    println!("{:?}", move_made.unwrap());
    make_move::<Black>(&mut board, move_made.unwrap());
    println! {"{}", board};
    board.side_to_move = board.side_to_move ^ 1;
    ai.reset_fields();
    let (move_made, _) = ai.make_decision(4, &mut board, None);
    make_move::<White>(&mut board, move_made.unwrap());
    println!("{:?}", move_made.unwrap());
    println! {"{}", board};
    assert!(false)
}
