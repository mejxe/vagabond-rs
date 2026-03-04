use std::{fmt::Display, iter::MapWhile};

use crate::bitboard::{BitBoard, Square};
const ALL_STARTING_BOARD: BitBoard = BitBoard(18446462598732906495);
const WHITE_STARTING_BOARD: BitBoard = BitBoard(65535);
const BLACK_STARTING_BOARD: BitBoard = BitBoard(18446462598732840960);
const WHITE_STARTING_BOARD_BY_PIECE: [BitBoard; 6] = [
    BitBoard(66),
    BitBoard(16),
    BitBoard(36),
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
pub struct CastlingRights(u8);
impl CastlingRights {
    pub fn new(K: bool, Q: bool, k: bool, q: bool) -> Self {
        let mut rights = 0u8;
        rights |= ((K as u8) << 3) | ((Q as u8) << 2) | ((k as u8) << 1) | (q as u8);
        CastlingRights(rights)
    }
    pub fn K(&self) -> bool {
        (self.0 >> 3 & 1) != 0
    }
    pub fn Q(&self) -> bool {
        (self.0 >> 2 & 1) != 0
    }
    pub fn k(&self) -> bool {
        (self.0 >> 1 & 1) != 0
    }
    pub fn q(&self) -> bool {
        (self.0 & 1) != 0
    }
    pub fn for_color(&self, color: Color) -> bool {
        match color {
            Color::Black => self.k() & self.q(),
            Color::White => self.K() & self.Q(),
        }
    }
}
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
    mailbox: [Option<Piece>; 64],

    pub occupied_by_color: [BitBoard; 2],

    // index with [Piece as usize] (same order as Piece)
    pub pieces: [[BitBoard; 6]; 2],

    castling_rights: CastlingRights,
    en_passant_square: Option<Square>,
}
impl Default for Board {
    fn default() -> Self {
        let occupied_by_color = [WHITE_STARTING_BOARD, BLACK_STARTING_BOARD];
        let pieces = [WHITE_STARTING_BOARD_BY_PIECE, BLACK_STARTING_BOARD_BY_PIECE];
        let castling_rights = CastlingRights::new(true, true, true, true);

        Board {
            mailbox: Board::fill_mailbox(),
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
            self.castling_rights, self.en_passant_square
        ));
        write!(f, "{}", board)
    }
}
impl Board {
    const fn fill_mailbox() -> [Option<Piece>; 64] {
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
    pub fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    #[inline(always)]
    pub fn en_passant_square(&self) -> Option<Square> {
        self.en_passant_square
    }

    #[inline(always)]
    pub fn get_piece_at_square(&self, square: Square) -> Option<Piece> {
        self.mailbox[square as usize]
    }
    #[inline(always)]
    pub fn set_piece_at_square(&mut self, square: Square, piece: Option<Piece>) {
        self.mailbox[square as usize] = piece;
    }
    #[inline(always)]
    pub fn occupied_by_color(&self, color: Color) -> BitBoard {
        self.occupied_by_color[color as usize]
    }
    #[inline(always)]
    pub fn all_occupied(&self) -> BitBoard {
        BitBoard(self.black_occupied().0 | self.white_occupied().0)
    }

    pub fn set_en_passant_square(&mut self, en_passant_square: Option<Square>) {
        self.en_passant_square = en_passant_square;
    }
    pub fn from_mailbox(
        mailbox: [Option<Piece>; 64],
        curr_move: Color,
        castling_rights: CastlingRights,
        en_passant_square: Option<Square>,
    ) -> Board {
        //let mut b = Board {}
        let mut pieces = [[BitBoard(0); 6]; 2];
        let mut occupied_by_color = [BitBoard(0); 2];
        for (i, piece) in mailbox.into_iter().enumerate() {
            if let Some(piece) = piece {
                pieces[piece.color as usize][piece.piece_type as usize].0 ^= 1u64 << i;
                occupied_by_color[piece.color as usize].0 ^= 1u64 << i;
            }
        }
        Board {
            mailbox,
            occupied_by_color,
            pieces,
            castling_rights,
            en_passant_square,
        }
    }
    pub fn from_FEN(fen_string: String) -> Board {
        let mut mailbox: [Option<Piece>; 64] = [None; 64];
        let mut iter = fen_string.split_ascii_whitespace();
        let placements = iter.next().expect("Not enough parts");
        let curr_move = iter.next().expect("Not enough parts");
        let castling_rights = iter.next().expect("Not enough parts");
        let ep_square = iter.next().expect("Not enough parts");
        let mut row = 7;
        let mut col = 0;
        for c in placements.chars() {
            match c {
                '/' => {
                    row -= 1;
                    col = 0
                }
                '1'..='8' => col += c.to_digit(10).unwrap() as u8,
                piece_char => {
                    let square = Square::from_u8_unchecked(row * 8 + col);
                    let piece = c_to_piece(piece_char).expect("Wrong character provided for FEN");
                    mailbox[square as usize] = Some(piece);
                    col += 1;
                    if col > 7 {
                        col = 0;
                    }
                }
            };
        }
        let to_move = match curr_move.chars().next().expect("Current move not found") {
            'w' => Color::White,
            'b' => Color::Black,
            _ => panic!("Wrong FEN format at 2nd pos"),
        };
        let mut cr = [false; 4];
        for c in castling_rights.chars() {
            match c {
                'K' => cr[0] = true,
                'Q' => cr[1] = true,
                'k' => cr[2] = true,
                'q' => cr[3] = true,
                _ => {}
            }
        }
        let castling_rights = CastlingRights::new(cr[0], cr[1], cr[2], cr[3]);
        let en_passant_square = chess_notation_to_sq(ep_square);
        Board::from_mailbox(mailbox, to_move, castling_rights, en_passant_square)
    }
}
fn c_to_piece(c: char) -> Option<Piece> {
    let piece_type = match c
        .to_lowercase()
        .next()
        .expect("Wrong character length-wise provided for FEN")
    {
        'b' => PieceType::Bishop,
        'k' => PieceType::King,
        'n' => PieceType::Knight,
        'p' => PieceType::Pawn,
        'r' => PieceType::Rook,
        'q' => PieceType::Queen,
        _ => return None,
    };
    let color = {
        if c.is_uppercase() {
            Color::White
        } else {
            Color::Black
        }
    };
    Some(Piece { piece_type, color })
}
fn chess_notation_to_sq(notation: &str) -> Option<Square> {
    let chars: Vec<_> = notation.chars().collect();
    if chars[0] == '-' {
        return None;
    };
    let col = chars[0] as u8 - b'a';
    let row = chars[1] as u8 - b'1';
    Some(Square::from_u8_unchecked(row * 8 + col))
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum Color {
    White,
    Black,
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

mod tests {
    use crate::{
        bitboard::{BitBoard, Square},
        board::Color,
        moves::traits::White,
    };

    use super::Board;

    #[test]
    fn fill_board() {
        let board = Board::default();
        println!("{}", board);
        assert!(false);
    }
    #[test]
    fn starting_from_FEN() {
        let starting =
            Board::from_FEN("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
        println!("{}", starting.all_occupied());
        println!(
            "{}",
            starting.get_pieces(crate::board::PieceType::Bishop, Color::White)
        );
        assert!(false)
    }
    #[test]
    fn ep_from_FEN() {
        let ep = Board::from_FEN(
            "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq e3 0 3".to_string(),
        );
        println!("{}", ep.all_occupied());
        println!(
            "{}",
            ep.get_pieces(crate::board::PieceType::Bishop, Color::White)
        );
        println!("{ep}");
        assert!(false)
    }
    #[test]
    fn kiwipete_from_FEN() {
        let b = Board::from_FEN(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string(),
        );
        println!("{}", b.all_occupied());
        println!(
            "{}",
            b.get_pieces(crate::board::PieceType::Bishop, Color::White)
        );
        println!("{b}");
        assert!(false)
    }
}
