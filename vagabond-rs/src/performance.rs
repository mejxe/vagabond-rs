use std::{
    ops::{Add, AddAssign},
    time::Instant,
};

use crate::{
    ai::evaluation::Evaluation,
    board::{
        bitboard::Square,
        board::{Board, Color, PieceType},
    },
    engine::{make_move, undo_move},
    format_with_commas,
    moves::{
        move_generator::{MoveGenerator, MoveList},
        move_structs::MoveType,
        traits::{Black, Castle, PawnDirection, Side, White},
    },
};

pub fn perft_entry(board: &mut Board, depth: u32) -> u64 {
    let move_generator = MoveGenerator::default();
    let time = Instant::now();
    let sum = match board.side_to_move {
        Color::White => perft::<White>(&move_generator, board, depth),
        Color::Black => perft::<Black>(&move_generator, board, depth),
    };
    let duration = time.elapsed();
    println!("-----------------------");
    println!("Total nodes: {}", format_with_commas(sum.into()));
    println!("Took: {:?}", duration.as_secs_f64());
    println!(
        "Nodes Per Second: {} nps",
        format_with_commas((sum as f64 / duration.as_secs_f64()).floor() as u64)
    );
    sum
}
pub fn perft_divide<S: Side + Castle + Evaluation + PawnDirection>(
    board: &mut Board,
    depth: u32,
) -> u64 {
    let move_generator = MoveGenerator::default();
    println!("--- Perft Divide Depth {} ---", depth);
    let mut total_nodes = 0;
    let mut move_list = MoveList::default();

    move_generator.generate_moves_generic::<S>(&mut move_list, board);

    for mv in move_list.as_slice() {
        let undo = make_move::<S>(board, mv.mv);

        if !move_generator.is_king_in_check(board, S::COLOR) {
            let nodes = perft::<S::Opposite>(&move_generator, board, depth - 1);

            println!("{}: {}", mv, format_with_commas(total_nodes.into()));

            total_nodes += nodes;
        }
        undo_move::<S>(mv.mv, board, undo);
    }

    println!("-----------------------");
    println!("Total nodes: {}", format_with_commas(total_nodes.into()));
    total_nodes
}
pub fn perft<S: Side + Castle + Evaluation>(
    move_generator: &MoveGenerator,
    board: &mut Board,
    depth: u32,
) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let mut move_list = MoveList::default();
    move_generator.generate_moves_generic::<S>(&mut move_list, board);
    let moves = move_list.as_slice();
    for mv in moves {
        let undo = make_move::<S>(board, mv.mv);
        //println!("board: {}", board);

        if !move_generator.is_king_in_check(board, S::COLOR) {
            //      println!("{}", board.get_piece_at_square(mv.to()).unwrap());
            //      println!("move: {}, {nodes}", mv);
            let new_nodes = perft::<S::Opposite>(move_generator, board, depth - 1);
            nodes += new_nodes;
        }

        undo_move::<S>(mv.mv, board, undo);
        //println!("undo_board: {}", board);
    }
    nodes
}
#[derive(Debug)]
pub struct PerftMoves {
    quiets: u64,
    captures: u64,
    castles: u64,
    en_passants: u64,
}
impl AddAssign<PerftMoves> for PerftMoves {
    fn add_assign(&mut self, rhs: PerftMoves) {
        self.quiets += rhs.quiets;
        self.captures += rhs.captures;
        self.castles += rhs.castles;
        self.en_passants += rhs.en_passants;
    }
}
pub fn perft_by_move_type<S: Side + Castle + Evaluation>(
    move_generator: &MoveGenerator,
    board: &mut Board,
    depth: u32,
    last_move_type: MoveType,
) -> PerftMoves {
    let mut perft_moves = PerftMoves {
        quiets: 0,
        captures: 0,
        castles: 0,
        en_passants: 0,
    };
    if depth == 0 {
        match last_move_type {
            MoveType::EnPassant => {
                perft_moves.en_passants += 1;
                perft_moves.captures += 1;
            }
            MoveType::KingSideCastle | MoveType::QueenSideCastle => perft_moves.castles += 1,
            MoveType::Capture
            | MoveType::BishopCapturePromotion
            | MoveType::QueenCapturePromotion
            | MoveType::KnightCapturePromotion
            | MoveType::RookCapturePromotion => perft_moves.captures += 1,
            _ => perft_moves.quiets += 1,
        }
        return perft_moves;
    }

    let mut move_list = MoveList::default();
    move_generator.generate_moves_generic::<S>(&mut move_list, board);
    let moves = move_list.move_fetcher();
    for mv in moves {
        let undo = make_move::<S>(board, mv.mv);
        //println!("board: {}", board);

        let king_square = Square::from_u8_unchecked(
            board
                .get_pieces(PieceType::King, S::COLOR)
                .0
                .trailing_zeros() as u8,
        );
        if !move_generator.is_square_attacked::<S>(board, king_square) {
            let new_moves = perft_by_move_type::<S::Opposite>(
                move_generator,
                board,
                depth - 1,
                mv.mv.move_type(),
            );
            perft_moves += new_moves;
        }

        undo_move::<S>(mv.mv, board, undo);
        //println!("undo_board: {}", board);
    }
    perft_moves
}
pub fn perft_divide_by_move_type<S: Side + Castle + Evaluation + PawnDirection>(
    board: &mut Board,
    depth: u32,
) -> PerftMoves {
    let time = Instant::now();
    let move_generator = MoveGenerator::default();
    println!("--- Perft Divide Depth {} ---", depth);
    let mut total_nodes = PerftMoves {
        captures: 0,
        castles: 0,
        en_passants: 0,
        quiets: 0,
    };
    let mut move_list = MoveList::default();

    move_generator.generate_moves_generic::<S>(&mut move_list, board);

    for mv in move_list.as_slice() {
        let undo = make_move::<S>(board, mv.mv);

        let king_bits = board.get_pieces(PieceType::King, S::COLOR).0;

        if king_bits != 0 {
            let king_square = Square::from_u8_unchecked(king_bits.trailing_zeros() as u8);

            if !move_generator.is_square_attacked::<S>(board, king_square) {
                let nodes = perft_by_move_type::<S::Opposite>(
                    &move_generator,
                    board,
                    depth - 1,
                    mv.mv.move_type(),
                );

                println!("{}: {:?}", mv, nodes);

                total_nodes += nodes;
            }
        }

        undo_move::<S>(mv.mv, board, undo);
    }

    let sum = total_nodes.captures + total_nodes.castles + total_nodes.quiets;
    let duration = time.elapsed();
    println!("-----------------------");
    println!("Total nodes: {:?}", total_nodes);
    println!("Total nodes: {}", format_with_commas(sum.into()));
    println!("Took: {:?}", duration.as_secs_f64());
    println!(
        "Nodes Per Second: {} nps",
        format_with_commas((sum as f64 / duration.as_secs_f64()).floor() as u64)
    );
    total_nodes
}

mod tests {
    use crate::{board::board::Board, engine::Engine, performance::perft_entry};

    #[test]
    fn test_perft() {
        let mut board = Board::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ".to_string(),
        ); // kiwipete
        let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 ".to_string()); // pos 3 cpw
        let mut board = Board::from_fen(
            "r1bq2r1/b4pk1/p1pp1p2/1p2pP2/1P2P1PB/3P4/1PPQ2P1/R3K2R w KQ - 0 1".to_string(),
        ); //tricky pos
        let mut board = Board::default();

        //perft_divide_by_move_type::<White>(&mut board, 5);
        perft_entry(&mut board, 7);
    }
    #[test]
    fn test_nega_max() {
        let board = Board::from_fen(
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
