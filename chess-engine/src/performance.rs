use crate::{
    bitboard::Square,
    board::{Board, Color, PieceType},
    engine::{make_move, undo_move},
    evaluation::Evaluation,
    moves::{
        move_generator::{MoveGenerator, MoveList},
        traits::{Black, Castle, PawnDirection, Side, White},
    },
};

pub fn perft_entry(board: &mut Board, depth: u32) -> u64 {
    let move_generator = MoveGenerator::default();
    match board.side_to_move {
        Color::White => perft::<White>(&move_generator, board, depth),
        Color::Black => perft::<Black>(&move_generator, board, depth),
    }
}
pub fn perft_divide<S: Side + Castle + Evaluation + PawnDirection>(
    board: &mut Board,
    depth: u32,
) -> u64 {
    let move_generator = MoveGenerator::default();
    println!("--- Perft Divide Depth {} ---", depth);
    let mut total_nodes = 0;
    let mut move_list = MoveList::default();

    // Generate the very first layer of moves
    move_generator.generate_moves_generic::<S>(&mut move_list, board);

    for mv in move_list.as_slice() {
        let undo = make_move::<S>(board, *mv);

        // Find the King to check legality (same as your standard perft)
        let king_bits = board.get_pieces(PieceType::King, S::COLOR).0;

        // Safety check just in case pseudo-legal generator allowed king capture
        if king_bits != 0 {
            let king_square = Square::from_u8_unchecked(king_bits.trailing_zeros() as u8);

            if !move_generator.is_square_attacked::<S>(board, king_square) {
                // Call your normal perft for the rest of the tree
                let nodes = perft::<S::Opposite>(&move_generator, board, depth - 1);

                // Print the root move and how many nodes it generated
                println!("{}: {}", mv, nodes);

                total_nodes += nodes;
            }
        }

        undo_move::<S>(*mv, board, undo);
    }

    println!("-----------------------");
    println!("Total nodes: {}", total_nodes);
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
        let undo = make_move::<S>(board, *mv);
        //println!("board: {}", board);

        let king_square = Square::from_u8_unchecked(
            board
                .get_pieces(crate::board::PieceType::King, S::COLOR)
                .0
                .trailing_zeros() as u8,
        );
        if !move_generator.is_square_attacked::<S>(board, king_square) {
            // TODO: fix that returning true always
            //      println!("{}", board.get_piece_at_square(mv.to()).unwrap());
            //      println!("move: {}, {nodes}", mv);
            let new_nodes = perft::<S::Opposite>(move_generator, board, depth - 1);
            nodes += new_nodes;
        }

        undo_move::<S>(*mv, board, undo);
        //println!("undo_board: {}", board);
    }
    nodes
}
