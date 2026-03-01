use std::{fmt::Display, iter::MapWhile};

use crate::bitboard::{BitBoard, Square};
const ALL_STARTING_BOARD: BitBoard = BitBoard(18446462598732906495);
const WHITE_STARTING_BOARD: BitBoard = BitBoard(65535);
const BLACK_STARTING_BOARD: BitBoard = BitBoard(18446462598732840960);
const WHITE_STARTING_BOARD_BY_PIECE: [BitBoard; 6] = [
    BitBoard(36),
    BitBoard(16),
    BitBoard(66),
    BitBoard(65280),
    BitBoard(129),
    BitBoard(8),
];
const BLACK_STARTING_BOARD_BY_PIECE: [BitBoard; 6] = [
    BitBoard(2594073385365405696),
    BitBoard(1152921504606846976),
    BitBoard(4755801206503243776),
    BitBoard(71776119061217280),
    BitBoard(9295429630892703744),
    BitBoard(576460752303423488),
];
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}
impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = match self.piece_type {
            PieceType::Bishop => "B",
            PieceType::King => "K",
            PieceType::Knight => "N",
            PieceType::Pawn => "P",
            PieceType::Rook => "R",
            PieceType::Queen => "Q",
        }
        .to_string();
        if self.color == Color::Black {
            string = string.to_lowercase()
        };
        write!(f, "{}", string)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum PieceType {
    Bishop,
    King,
    Knight,
    Pawn,
    Rook,
    Queen,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct Board {
    mail_box: [Option<Piece>; 64],

    all_occupied: BitBoard,
    occupied_by_color: [BitBoard; 2],

    // index with [Piece as usize] (same order as Piece)
    pieces: [[BitBoard; 6]; 2],

    castling_rights: [bool; 2],
    en_passant_square: Option<Square>,
}
impl Default for Board {
    fn default() -> Self {
        let all_occupied = ALL_STARTING_BOARD;
        let occupied_by_color = [WHITE_STARTING_BOARD, BLACK_STARTING_BOARD];
        let pieces = [WHITE_STARTING_BOARD_BY_PIECE, BLACK_STARTING_BOARD_BY_PIECE];
        let castling_rights = [true, true];

        Board {
            mail_box: Board::fill_mail_box(),
            all_occupied,
            pieces,
            occupied_by_color,
            castling_rights,
            en_passant_square: None,
        }
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
                match self.mail_box[(row * 8 + col) as usize] {
                    Some(piece) => board.push_str(&format!(" {} ", piece)),
                    None => board.push_str(&format!("   ")),
                }
            }
            board.push('\n');
        }
        board.push_str("\n   A  B  C  D  E  F  G  H \n");
        write!(f, "{}", board)
    }
}
impl Board {
    const fn fill_mail_box() -> [Option<Piece>; 64] {
        let mut mailbox = [None; 64];
        let mut sq = 0;
        while sq < 16 {
            if sq / 8 == 1 {
                mailbox[sq] = Some(Piece {
                    color: Color::White,
                    piece_type: PieceType::Pawn,
                });
                mailbox[sq + 40] = Some(Piece {
                    color: Color::Black,
                    piece_type: PieceType::Pawn,
                });
            } else if sq % 8 == 0 || sq % 8 == 7 {
                mailbox[sq] = Some(Piece {
                    color: Color::White,
                    piece_type: PieceType::Rook,
                });
                mailbox[sq + 56] = Some(Piece {
                    color: Color::Black,
                    piece_type: PieceType::Rook,
                });
            } else if sq % 8 == 1 || sq % 8 == 6 {
                mailbox[sq] = Some(Piece {
                    color: Color::White,
                    piece_type: PieceType::Bishop,
                });
                mailbox[sq + 56] = Some(Piece {
                    color: Color::Black,
                    piece_type: PieceType::Bishop,
                });
            } else if sq % 8 == 2 || sq % 8 == 5 {
                mailbox[sq] = Some(Piece {
                    color: Color::White,
                    piece_type: PieceType::Knight,
                });
                mailbox[sq + 56] = Some(Piece {
                    color: Color::Black,
                    piece_type: PieceType::Knight,
                });
            } else if sq % 8 == 3 {
                mailbox[sq] = Some(Piece {
                    color: Color::White,
                    piece_type: PieceType::Queen,
                });
                mailbox[sq + 56] = Some(Piece {
                    color: Color::Black,
                    piece_type: PieceType::Queen,
                });
            } else if sq % 8 == 4 {
                mailbox[sq] = Some(Piece {
                    color: Color::White,
                    piece_type: PieceType::King,
                });
                mailbox[sq + 56] = Some(Piece {
                    color: Color::Black,
                    piece_type: PieceType::King,
                });
            }
            sq += 1;
        }
        mailbox
    }

    pub fn all_occupied(&self) -> BitBoard {
        self.all_occupied
    }
    #[inline(always)]
    pub fn white_occupied(&self) -> BitBoard {
        self.occupied_by_color[0]
    }
    #[inline(always)]
    pub fn black_occupied(&self) -> BitBoard {
        self.occupied_by_color[1]
    }

    #[inline(always)]
    pub fn get_pieces(&self, piece_type: PieceType, color: Color) -> BitBoard {
        self.pieces[color as usize][piece_type as usize]
    }
    #[inline(always)]
    pub fn get_castling_rights(&self, color: Color) -> bool {
        self.castling_rights[color as usize]
    }

    #[inline(always)]
    pub fn en_passant_square(&self) -> Option<Square> {
        self.en_passant_square
    }

    pub fn occupied_by_color(&self, color: Color) -> BitBoard {
        self.occupied_by_color[color as usize]
    }
}
// TODO: Add fen parsing
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum Color {
    White,
    Black,
}

mod tests {
    use crate::bitboard::{BitBoard, Square};

    use super::Board;

    #[test]
    fn fill_board() {
        let board = Board::default();
        println!("{}", board);
        assert!(false);
    }
}
