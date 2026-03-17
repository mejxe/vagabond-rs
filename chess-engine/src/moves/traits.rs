use std::fmt::Display;

use crate::{
    ai::evaluation::Evaluation,
    board::bitboard::{BitBoard, Square},
    board::board::Color,
};

use super::move_structs::{ExtMove, Move};

pub trait Castle {
    const KING_SIDE: BitBoard;
    const QUEEN_SIDE: BitBoard;
    const KING_SIDE_KING_POS: Square;
    const QUEEN_SIDE_KING_POS: Square;

    const KING_START_POS: Square;
    const KING_ROOK_START_POS: Square;
    const QUEEN_ROOK_START_POS: Square;

    const QUEEN_SIDE_ROOK_POS: Square;
    const KING_SIDE_ROOK_POS: Square;
}
impl Castle for White {
    const KING_SIDE: BitBoard = {
        let mut bitboard = BitBoard(0);
        bitboard.set_bit(Square::F1);
        bitboard.set_bit(Square::G1);
        bitboard
    };
    const QUEEN_SIDE: BitBoard = {
        let mut bitboard = BitBoard(0);
        bitboard.set_bit(Square::D1);
        bitboard.set_bit(Square::C1);
        bitboard.set_bit(Square::B1);
        bitboard
    };
    const KING_START_POS: Square = Square::E1;
    const KING_ROOK_START_POS: Square = Square::H1;
    const QUEEN_ROOK_START_POS: Square = Square::A1;

    const KING_SIDE_KING_POS: Square = Square::G1;
    const QUEEN_SIDE_KING_POS: Square = Square::C1;

    const KING_SIDE_ROOK_POS: Square = Square::F1;
    const QUEEN_SIDE_ROOK_POS: Square = Square::D1;
}

impl Castle for Black {
    const KING_SIDE: BitBoard = {
        let mut bitboard = BitBoard(0);
        bitboard.set_bit(Square::F8);
        bitboard.set_bit(Square::G8);
        bitboard
    };
    const QUEEN_SIDE: BitBoard = {
        let mut bitboard = BitBoard(0);
        bitboard.set_bit(Square::D8);
        bitboard.set_bit(Square::C8);
        bitboard.set_bit(Square::B8);
        bitboard
    };
    const KING_START_POS: Square = Square::E8;
    const KING_ROOK_START_POS: Square = Square::H8;
    const QUEEN_ROOK_START_POS: Square = Square::A8;

    const KING_SIDE_KING_POS: Square = Square::G8;
    const QUEEN_SIDE_KING_POS: Square = Square::C8;

    const KING_SIDE_ROOK_POS: Square = Square::F8;
    const QUEEN_SIDE_ROOK_POS: Square = Square::D8;
}

pub trait PawnDirection {
    const SHIFT: u8;
    const DOUBLE_SHIFT: u8;
    const STARTING_RANK: u64;
    const PROMOTION_RANK: u64;

    fn shift(bitboard: BitBoard) -> BitBoard;
    fn get_source_single(square: Square) -> Square;
    fn get_source_double(square: Square) -> Square;
}
pub trait Side {
    const COLOR: Color;
    const STARTING_RANK: u64;
    const PROMOTION_RANK: u64;
    type Opposite: Side + Castle + Evaluation;
}
impl<S: Side> PawnDirection for S {
    const SHIFT: u8 = 8;
    const DOUBLE_SHIFT: u8 = 2 * Self::SHIFT;
    const PROMOTION_RANK: u64 = Self::PROMOTION_RANK;
    const STARTING_RANK: u64 = Self::STARTING_RANK;
    fn shift(bitboard: BitBoard) -> BitBoard {
        if S::COLOR == Color::White {
            BitBoard(bitboard.0 << Self::SHIFT)
        } else {
            BitBoard(bitboard.0 >> Self::SHIFT)
        }
    }
    fn get_source_single(square: Square) -> Square {
        if S::COLOR == Color::White {
            Square::from_u8_unchecked((square as u8) - Self::SHIFT)
        } else {
            Square::from_u8_unchecked((square as u8) + Self::SHIFT)
        }
    }
    fn get_source_double(square: Square) -> Square {
        if S::COLOR == Color::White {
            Square::from_u8_unchecked((square as u8) - Self::DOUBLE_SHIFT)
        } else {
            Square::from_u8_unchecked((square as u8) + Self::DOUBLE_SHIFT)
        }
    }
}
pub struct White;
pub struct Black;
impl Side for White {
    const COLOR: Color = Color::White;
    const STARTING_RANK: u64 = 0xFF << 8;
    const PROMOTION_RANK: u64 = 0xFF << (7 * 8);
    type Opposite = Black;
}
impl Side for Black {
    const COLOR: Color = Color::Black;
    const STARTING_RANK: u64 = 0xFF << (6 * 8);
    const PROMOTION_RANK: u64 = 0xFF;
    type Opposite = White;
}

impl Display for ExtMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | score: {}", self.mv, self.score)
    }
}
impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} to {}, {:?}",
            self.from(),
            self.to(),
            self.move_type()
        )
    }
}
