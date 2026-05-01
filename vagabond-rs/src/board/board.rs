use std::{
    fmt::Display,
    iter::MapWhile,
    ops::{BitXor, Index},
};

use crate::{
    ai::evaluation::{Evaluation, PestoEvaluation},
    board::bitboard::{BitBoard, Square},
    moves::{move_generator::Undo, move_structs::Move},
    tt::zobrist::{ZOBRIST_HASHER, ZobristHash, ZobristHasher},
    uci::handler,
};
pub const MAX_HALF_MOVES: usize = 1024;

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

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Board {
    pub(crate) mailbox: [Option<Piece>; 64],

    pub occupied_by_color: [BitBoard; 2],

    // index with [Piece as usize] (same order as Piece)
    pub pieces: [[BitBoard; 6]; 2],

    pub side_to_move: Color,
    castling_rights: CastlingRights,
    en_passant_square: Option<Square>,

    // pesto eval
    pub mg_score: i16,
    pub eg_score: i16,
    pub phase: i16,

    pub zobrist: ZobristHash,
    pub history: Vec<ZobristHash>,
    pub half_move_clock: usize,
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

    #[inline(always)]
    pub fn set_en_passant_square(&mut self, en_passant_square: Option<Square>) {
        self.en_passant_square = en_passant_square;
    }
    pub fn swap_side(&mut self) {
        self.side_to_move ^= 1;
    }
    pub fn from_mailbox(
        mailbox: [Option<Piece>; 64],
        curr_move: Color,
        castling_rights: CastlingRights,
        en_passant_square: Option<Square>,
        half_move_clock: usize,
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
        let (mg_score, eg_score, phase) = Board::evaluate_whole_board(mailbox);
        let zobrist = ZobristHasher::get_hasher().calculate_board_hash(
            &mailbox,
            castling_rights,
            en_passant_square,
            curr_move,
        );
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
            zobrist,
            history: Vec::new(),
            half_move_clock,
        }
    }
    pub fn from_fen(fen_string: String) -> Board {
        let mut mailbox: [Option<Piece>; 64] = [None; 64];
        let mut iter = fen_string.split_ascii_whitespace();
        let placements = iter.next().expect("Not enough parts");
        let curr_move = iter.next().expect("Not enough parts");
        let castling_rights = iter.next().expect("Not enough parts");
        let ep_square = iter.next().expect("Not enough parts");
        let half_move_clock = iter.next().expect("Not enough parts");
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
        let half_move_clock: usize = half_move_clock
            .parse()
            .expect("Not a valid integer for half move clock.");
        Board::from_mailbox(
            mailbox,
            to_move,
            castling_rights,
            en_passant_square,
            half_move_clock,
        )
    }
    fn evaluate_whole_board(mailbox: [Option<Piece>; 64]) -> (i16, i16, i16) {
        let mut mg_scores = [0; 2]; // per color
        let mut eg_scores = [0; 2];
        let mut game_phase = 0;
        for (i, possible_piece) in mailbox.into_iter().enumerate() {
            if let Some(piece) = possible_piece {
                let eval = PestoEvaluation::get_mg_score(Square::from_u8_unchecked(i as u8), piece);
                let eval = match piece.color {
                    Color::White => eval,
                    Color::Black => -eval,
                };
                mg_scores[piece.color as usize] +=
                    PestoEvaluation::get_mg_score(Square::from_u8_unchecked(i as u8), piece);
                eg_scores[piece.color as usize] +=
                    PestoEvaluation::get_eg_score(Square::from_u8_unchecked(i as u8), piece);
                game_phase += PestoEvaluation::PIECE_PHASE_INCR[piece.piece_type as usize];
            }
        }
        (
            mg_scores[0] - mg_scores[1],
            eg_scores[0] - eg_scores[1],
            game_phase,
        )
    }
    pub fn evaluate(&self) -> i16 {
        let clamped = self.phase.min(24);
        let mg = self.mg_score as i32 * clamped as i32;
        let eg = self.eg_score as i32 * (24 - clamped as i32);
        ((mg + eg) / 24) as i16
    }
    pub fn evaluate_piece(&self, piece: PieceType) -> u16 {
        let clamped = self.phase.min(24);
        let mg = PestoEvaluation::MG_MATERIAL_VAL[piece as usize] as u32 * clamped as u32;
        let eg = PestoEvaluation::EG_MATERIAL_VAL[piece as usize] as u32 * (24 - clamped as u32);
        ((mg + eg) / 24) as u16
    }
    pub fn add_score<S: Evaluation>(&mut self, square: Square, piece: Piece) {
        self.mg_score += PestoEvaluation::get_mg_score(square, piece) * S::MULTIPLIER;
        self.eg_score += PestoEvaluation::get_eg_score(square, piece) * S::MULTIPLIER;
    }
    pub fn subtract_score<S: Evaluation>(&mut self, square: Square, piece: Piece) {
        self.mg_score -= PestoEvaluation::get_mg_score(square, piece) * S::MULTIPLIER;
        self.eg_score -= PestoEvaluation::get_eg_score(square, piece) * S::MULTIPLIER;
    }

    #[inline(always)]
    pub fn is_draw(&self) -> bool {
        // max_repetitions - how many times a board can repeat before being flagged
        // check for 50 non capture moves
        if self.half_move_clock >= 100 {
            return true;
        }
        // check for 3 fold draw
        if self.num_repetitions() >= 3 {
            return true;
        }
        let pawns = self.pieces[Color::White][PieceType::Pawn as usize].0
            | self.pieces[Color::Black][PieceType::Pawn as usize].0;
        let rooks = self.pieces[Color::White][PieceType::Rook as usize].0
            | self.pieces[Color::Black][PieceType::Rook as usize].0;
        let queens = self.pieces[Color::White][PieceType::Queen as usize].0
            | self.pieces[Color::Black][PieceType::Queen as usize].0;

        let sufficient_material = (pawns | rooks | queens) != 0;
        if !sufficient_material {
            if ((self.pieces[Color::White][PieceType::Bishop]
                | self.pieces[Color::White][PieceType::Knight])
                .0
                .count_ones()
                < 2)
                && ((self.pieces[Color::Black][PieceType::Bishop]
                    | self.pieces[Color::Black][PieceType::Knight])
                    .0
                    .count_ones()
                    < 2)
            {
                return true;
            }
        }
        false
    }
    pub fn num_repetitions(&self) -> usize {
        let mut i = 2; // previous move of current side is at history[n-2]
        let mut repetitions = 1;
        while i <= self.history.len() && i <= self.half_move_clock {
            if self.history[self.history.len() - i] == self.zobrist {
                repetitions += 1;
            }
            i += 2; // next moves are n - k, where k is 1 .. self.half_move_clock with step 2
        }
        repetitions
    }
    pub fn update_history_and_hm(&mut self, mv: Move) {
        self.history.push(self.zobrist);
        if mv.move_type().is_capture()
            || self.get_piece_at_square(mv.from()).unwrap().piece_type == PieceType::Pawn
        {
            self.half_move_clock = 0;
        } else {
            self.half_move_clock += 1;
        }
    }
    pub fn make_null_move(&mut self) -> Undo {
        let undo = Undo {
            previous_ep_square: self.en_passant_square(),
            captured_piece: None,
            castling_rights: self.castling_rights,
            half_move_clock: self.half_move_clock,
        };
        let hasher = ZobristHasher::get_hasher();
        hasher.flip_zobrist_hash(self);
        self.en_passant_square = None;
        self.swap_side();
        hasher.flip_zobrist_hash(self);
        hasher.flip_side_to_move_hash(self);
        undo
    }
    pub fn undo_null_move(&mut self, undo: Undo) {
        let hasher = ZobristHasher::get_hasher();
        hasher.flip_zobrist_hash(self);
        self.swap_side();
        self.en_passant_square = undo.previous_ep_square;
        hasher.flip_zobrist_hash(self);
        hasher.flip_side_to_move_hash(self);
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
pub fn chess_notation_to_sq(notation: &str) -> Option<Square> {
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
    pub fn k_for_color(&self, color: Color) -> bool {
        match color {
            Color::White => self.K(),
            Color::Black => self.k(),
        }
    }
    pub fn q_for_color(&self, color: Color) -> bool {
        match color {
            Color::White => self.Q(),
            Color::Black => self.q(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::bitboard::{BitBoard, Square},
        board::board::{Color, PieceType},
        moves::traits::White,
    };

    use super::Board;
    #[test]
    fn test_starting_position_parsing() {
        // Arrange
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());

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
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());

        // Act & Assert
        assert_eq!(board.evaluate(), 0);
        let board =
            Board::from_fen("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
        assert_eq!(board.evaluate(), 992);
    }
}
#[cfg(test)]
mod debug_tests {
    use super::Board;
    use crate::{
        board::bitboard::{BitBoard, Square},
        board::board::{Color, PieceType},
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
        let ep = Board::from_fen(
            "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq e3 0 3".to_string(),
        );
        println!("{}", ep.all_occupied());
        println!("{}", ep.get_pieces(PieceType::Bishop, Color::White));
        println!("{ep}");
        assert!(false)
    }
    #[test]
    fn kiwipete_from_FEN() {
        let b = Board::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string(),
        );
        println!("{}", b.all_occupied());
        println!("{}", b.get_pieces(PieceType::Bishop, Color::White));
        println!("{b}");
        assert!(false)
    }
    #[test]
    fn evaluate_position_test() {
        let b = Board::from_fen(
            "r3k2r/p2pqpb1/bnpPpnp1/4N3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 2".to_string(),
        );
        println!("{}", b.evaluate());
        assert!(false);
    }
}
