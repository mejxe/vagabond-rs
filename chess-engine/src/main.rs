use std::{
    io::{self, BufReader, Stdin},
    sync::{Arc, atomic::AtomicBool, mpsc::channel},
};

use chess_engine::{
    ai::ai::AI,
    board::board::Board,
    engine::{Engine, make_move},
    moves::{
        move_generator::MoveGenerator,
        traits::{Black, White},
    },
    performance::{perft_divide, perft_divide_by_move_type, perft_entry},
    uci::{
        communication::Communication,
        handler::Handler,
        structs::{UciIn, UciOut},
    },
};

fn main() -> Result<(), ()> {
    let mut engine = Engine::default();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let (tx_in, rx_in) = channel::<UciIn>();
    let (tx_out, rx_out) = channel::<UciOut>();
    engine.set_tx(tx_out.clone());
    let mut handler = Handler::new(engine, rx_in, tx_out, stop_flag.clone());
    let std_in = BufReader::new(io::stdin());
    std::thread::spawn(move || handler.handle());
    std::thread::spawn(move || Communication::broadcast(rx_out));
    Communication::communication_loop(std_in, tx_in, stop_flag.clone())
}
mod tests {
    use chess_engine::{board::board::Board, engine::Engine, performance::perft_entry};

    #[test]
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
    #[ignore]
    #[test]
    fn test_nega_max() {
        let board = Board::from_FEN(
            "r2q1rk1/1p1bbppp/p2pbn2/4p3/4P3/1NN1BP2/PPPQ2PP/R3KB1R w KQ - 4 11".to_string(),
        );
        println!("{}", board);
        let mut engine = Engine::default();
        engine.set_board(board);
        engine.set_depth(7);
        for i in 0..1 {
            //let move_made = engine.play();
            //println!("{i}: {}", move_made.unwrap());
            //         let move_made = engine.play();
            //         println!("{}: {}", i + 1, move_made.unwrap());
        }
        assert!(false)
    }
}
