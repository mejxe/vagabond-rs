use std::{
    fmt::Display,
    ops::{BitXor, BitXorAssign, Index},
};

use super::{
    bitboard::BitBoard,
    board::{Board, CastlingRights, Color, PieceType},
};

impl Display for CastlingRights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if self.K() {
            s.push('K');
        }
        if self.Q() {
            s.push('Q');
        }
        if self.k() {
            s.push('k');
        }
        if self.q() {
            s.push('q');
        }
        write!(f, "{s}")
    }
}

impl Index<PieceType> for [BitBoard; 6] {
    type Output = BitBoard;
    #[inline(always)]
    fn index(&self, index: PieceType) -> &Self::Output {
        &self[index as usize]
    }
}
impl Default for Board {
    fn default() -> Self {
        Board::from_FEN("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string())
    }
}
impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board = String::new();
        for row in (0..8u8).rev() {
            board.push_str(&format!(
                "{} ",
                char::from_digit((row + 1).into(), 10).unwrap()
            ));
            for col in 0..8u8 {
                match self.mailbox[(row * 8 + col) as usize] {
                    Some(piece) => board.push_str(&format!(" {} ", piece)),
                    None => board.push_str(&format!("   ")),
                }
            }
            board.push('\n');
        }
        board.push_str("\n   A  B  C  D  E  F  G  H \n");
        board.push_str(&format!(
            "\n castling: {} | en_passant_square = {:?}",
            self.castling_rights(),
            self.en_passant_square()
        ));
        write!(f, "{}", board)
    }
}

impl Index<Color> for [[BitBoard; 6]; 2] {
    type Output = [BitBoard; 6];
    #[inline(always)]
    fn index(&self, index: Color) -> &Self::Output {
        &self[index as usize]
    }
}
impl BitXor<u8> for Color {
    type Output = Color;
    fn bitxor(self, rhs: u8) -> Self::Output {
        Color::from_u8_unchecked(self as u8 ^ rhs)
    }
}
impl BitXorAssign<u8> for Color {
    fn bitxor_assign(&mut self, rhs: u8) {
        *self = *self ^ rhs
    }
}
impl Color {
    pub const fn from_u8_unchecked(v: u8) -> Self {
        if v > 1 {
            panic!("Color does not match");
        }
        unsafe { std::mem::transmute(v) }
    }
}
impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = match self {
            Color::White => "White",
            Color::Black => "Black",
        };
        write!(f, "{}", color)
    }
}
