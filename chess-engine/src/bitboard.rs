use std::fmt::Display;

use macros::create_board_enum;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct BitBoard(pub u64);
create_board_enum!(Square, false);

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board = String::new();
        for row in (0..8u8).rev() {
            board.push_str(&format!(
                "{} ",
                char::from_digit((row + 1).into(), 10).unwrap()
            ));
            for col in 0..8u8 {
                let square: Square = (row * 8u8 + col).try_into().unwrap();
                let occupied = self.check_bit(square);
                board.push_str(&format!(" {} ", occupied as u8));
            }
            board.push('\n');
        }
        board.push_str("\n   A  B  C  D  E  F  G  H \n");
        write!(f, "{}", board)
    }
}
impl BitBoard {
    pub const fn check_bit(&self, square: Square) -> bool {
        (self.0 & (1 << (square as u8))) != 0
    }
    pub const fn set_bit(&mut self, square: Square) {
        self.0 |= 1 << (square as u8)
    }
    pub const fn mask(&mut self, mask: u64) {
        self.0 |= mask;
    }
    pub const fn unset_bit(&mut self, square: Square) {
        if self.check_bit(square) {
            self.0 ^= 1 << (square as u8)
        }
    }
}
