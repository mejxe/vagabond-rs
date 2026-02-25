use crate::bitboard::BitBoard;

pub enum Piece {
    Bishop,
    King,
    Knight,
    Pawn,
    Rook,
    Queen,
}

pub struct Board {
    all_occupied: BitBoard,

    white_occupied: BitBoard,
    black_occupied: BitBoard,

    white_pieces: [BitBoard; 6],
    black_pieces: [BitBoard; 6],
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum Color {
    White,
    Black,
}
