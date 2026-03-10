use std::{
    fmt::Display,
    iter::MapWhile,
    ops::{BitXor, Index},
};

use crate::{
    bitboard::{BitBoard, Square},
    evaluation::{Evaluation, PestoEvaluation},
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct CastlingRights(pub u8);
impl CastlingRights {
    pub fn new(K: bool, Q: bool, k: bool, q: bool) -> Self {
        let mut rights = 0u8;
        rights |= ((K as u8) << 3) | ((Q as u8) << 2) | ((k as u8) << 1) | (q as u8);
        CastlingRights(rights)
    }
    pub fn from_mask(mask: u8) -> Self {
        if mask > 0b1111 {
            panic!("Incorrect mask format.");
        }
        CastlingRights(mask)
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
impl Index<PieceType> for [BitBoard; 6] {
    type Output = BitBoard;
    #[inline(always)]
    fn index(&self, index: PieceType) -> &Self::Output {
        &self[index as usize]
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct Board {
    mailbox: [Option<Piece>; 64],

    pub occupied_by_color: [BitBoard; 2],

    // index with [Piece as usize] (same order as Piece)
    pub pieces: [[BitBoard; 6]; 2],

    castling_rights: CastlingRights,
    en_passant_square: Option<Square>,

    // pesto eval
    pub mg_score: i16,
    pub eg_score: i16,
    pub phase: i16,

    pub side_to_move: Color,
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
    pub fn set_castling_rights(&mut self, cr: CastlingRights) {
        self.castling_rights = cr;
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
        let (mg_score, eg_score, phase) = Board::evaluate_whole_board(mailbox, curr_move);
        Board {
            mailbox,
            occupied_by_color,
            pieces,
            castling_rights,
            en_passant_square,
            mg_score,
            eg_score,
            phase,
            side_to_move: curr_move,
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
    fn evaluate_whole_board(mailbox: [Option<Piece>; 64], current_move: Color) -> (i16, i16, i16) {
        let mut mg_scores = [0; 2]; // per color
        let mut eg_scores = [0; 2];
        let mut game_phase = 0;
        for (i, possible_piece) in mailbox.into_iter().enumerate() {
            if let Some(piece) = possible_piece {
                mg_scores[piece.color as usize] +=
                    PestoEvaluation::get_mg_score(Square::from_u8_unchecked(i as u8), piece);
                eg_scores[piece.color as usize] +=
                    PestoEvaluation::get_eg_score(Square::from_u8_unchecked(i as u8), piece);
                game_phase += PestoEvaluation::PIECE_PHASE_INCR[piece.piece_type as usize];
            }
        }
        (
            mg_scores[current_move as usize] - mg_scores[current_move as usize ^ 1],
            eg_scores[current_move as usize] - eg_scores[current_move as usize ^ 1],
            game_phase,
        )
    }
    pub fn evaluate(&self) -> i16 {
        (self.mg_score / 24 * self.phase + self.eg_score / 24 * (24 - self.phase))
    }
    pub fn add_score<S: Evaluation>(&mut self, square: Square, piece: Piece) {
        self.mg_score += PestoEvaluation::get_mg_score(square, piece) * S::MULTIPLIER;
        self.eg_score += PestoEvaluation::get_eg_score(square, piece) * S::MULTIPLIER;
    }
    pub fn subtract_score<S: Evaluation>(&mut self, square: Square, piece: Piece) {
        self.mg_score -= PestoEvaluation::get_mg_score(square, piece) * S::MULTIPLIER;
        self.eg_score -= PestoEvaluation::get_eg_score(square, piece) * S::MULTIPLIER;
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

#[cfg(test)]
mod tests {
    use crate::{
        bitboard::{BitBoard, Square},
        board::{Color, PieceType},
        moves::traits::White,
    };

    use super::Board;
    #[test]
    fn test_starting_position_parsing() {
        // Arrange
        let board =
            Board::from_FEN("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());

        // Act & Assert
        let white_pawns = board.get_pieces(PieceType::Pawn, Color::White);
        let black_kings = board.get_pieces(PieceType::King, Color::Black);

        // White pawns should be on the 2nd rank (bits 8-15)
        assert_eq!(white_pawns.0, 0x000000000000FF00);

        // Black king should be on e8 (bit 60)
        assert_eq!(black_kings.0, 1 << 60);

        // Verify castling rights are fully intact
        let rights = board.castling_rights();
        assert!(rights.K() && rights.Q() && rights.k() && rights.q());
    }
    #[test]
    fn test_evaluation() {
        // Arrange
        let board =
            Board::from_FEN("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());

        // Act & Assert
        assert_eq!(board.evaluate(), 0);
        let board =
            Board::from_FEN("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
        assert_eq!(board.evaluate(), 992);
    }
}
#[cfg(test)]
mod debug_tests {
    use super::Board;
    use crate::{
        bitboard::{BitBoard, Square},
        board::{Color, PieceType},
        moves::traits::White,
    };

    #[test]
    #[ignore]
    fn fill_board() {
        let board = Board::default();
        println!("{}", board);
        assert!(false);
    }
    #[test]
    #[ignore]
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
