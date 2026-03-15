use std::fmt::Display;

use crate::board::{
    bitboard::Square,
    board::{Board, PieceType},
};

#[repr(transparent)]
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Copy, Clone)]
pub struct Move(pub u16);
impl Move {
    pub fn new(from: Square, to: Square, move_type: MoveType) -> Self {
        let mut r#move = 0u16;
        r#move |= from as u16;
        r#move |= (to as u16) << 6;
        r#move |= (move_type as u16) << 12;
        Self(r#move)
    }

    pub fn from(&self) -> Square {
        Square::from_u8_unchecked((self.0 & 0x3F) as u8)
    }
    pub fn to(&self) -> Square {
        Square::from_u8_unchecked(((self.0 >> 6) & 0x3F) as u8)
    }
    pub fn move_type(&self) -> MoveType {
        MoveType::from_u8_unchecked(((self.0 >> 12) & 0xF) as u8)
    }
    pub fn promotion_to(&self) -> Option<PieceType> {
        match self.move_type() {
            MoveType::BishopCapturePromotion | MoveType::BishopPromotion => Some(PieceType::Bishop),
            MoveType::KnightCapturePromotion | MoveType::KnightPromotion => Some(PieceType::Knight),
            MoveType::RookCapturePromotion | MoveType::RookPromotion => Some(PieceType::Rook),
            MoveType::QueenCapturePromotion | MoveType::QueenPromotion => Some(PieceType::Queen),
            _ => None,
        }
    }
}
pub enum Promotion {
    ToBishop,
    ToKnight,
    ToRook,
    ToQueen,
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
#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Copy, Clone)]
pub enum MoveType {
    Quiet,
    Capture,
    DoublePush,
    KingSideCastle,
    QueenSideCastle,
    EnPassant,
    BishopPromotion,
    KnightPromotion,
    RookPromotion,
    QueenPromotion,
    BishopCapturePromotion,
    KnightCapturePromotion,
    RookCapturePromotion,
    QueenCapturePromotion,
}
impl MoveType {
    const VARIANTS: u8 = 13;
    pub fn is_capture(&self) -> bool {
        match self {
            MoveType::Capture => true,
            MoveType::QueenCapturePromotion => true,
            MoveType::RookCapturePromotion => true,
            MoveType::BishopCapturePromotion => true,
            MoveType::KnightCapturePromotion => true,
            MoveType::EnPassant => true,
            _ => false,
        }
    }
    pub const fn from_u8_unchecked(v: u8) -> Self {
        if v > Self::VARIANTS {
            panic!("Value doesn't match an Move Type");
        }
        unsafe { std::mem::transmute(v) }
    }
}
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Copy, Clone)]
pub struct ExtMove {
    pub mv: Move,
    pub score: u16,
}
impl ExtMove {
    // for scoring capture moves only
    pub fn score_move(&mut self, board: &Board, ply: u8, killers: &[[Option<Move>; 2]; 64]) {
        let move_type = self.mv.move_type();
        if move_type.is_capture() {
            let aggressor = board
                .get_piece_at_square(self.mv.from())
                .unwrap()
                .piece_type;

            let victim = if move_type == MoveType::EnPassant {
                PieceType::Pawn
            } else {
                board.get_piece_at_square(self.mv.to()).unwrap().piece_type
            };

            self.score = MVV_LVA_TABLE[victim as usize][aggressor as usize] + 100;
        } else {
            if killers[ply as usize][0] == Some(self.mv) {
                self.score = 90
            } else if killers[ply as usize][1] == Some(self.mv) {
                self.score = 80;
            } else {
                self.score = 0;
            }
        }
    }
}

// TODO: Test if no branching with None variant is better
pub const MVV_LVA_TABLE: [[u16; 6]; 6] = [
    [0, 0, 0, 0, 0, 0],       // victim K, attacker K, Q, R, B, N, P
    [50, 51, 52, 53, 54, 55], // victim Q, attacker K, Q, R, B, N, P
    [40, 41, 42, 43, 44, 45], // victim R, attacker K, Q, R, B, N, P
    [30, 31, 32, 33, 34, 35], // victim B, attacker K, Q, R, B, N, P
    [20, 21, 22, 23, 24, 25], // victim N, attacker K, Q, R, B, N, P
    [10, 11, 12, 13, 14, 15], // victim P, attacker K, Q, R, B, N, P
];
