use std::{arch::x86_64::_pext_u64, fmt::Display};

use crate::{
    board::bitboard::{self, BitBoard, Square},
    board::board::{Board, CastlingRights, Color, Piece, PieceType},
    moves::sliders::{
        BISHOP_MASK_TABLE, ROOK_MASK_TABLE, generate_bishop_attacks, generate_bishop_mask,
        generate_rook_attacks, generate_rook_mask,
    },
};

use super::{
    leapers::{
        B_PAWN_ATK_TABLE, KING_ATK_TABLE, KNIGHT_ATK_TABLE, PAWN_ATK_TABLE, W_PAWN_ATK_TABLE,
    },
    traits::{Black, Castle, PawnDirection, Side, White},
};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Copy, Clone)]
pub struct Undo {
    pub captured_piece: Option<PieceType>,
    pub previous_ep_square: Option<Square>,
    pub castling_rights: CastlingRights,
}
#[repr(transparent)]
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Copy, Clone)]
pub struct Move(u16);
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
#[derive(Debug)]
pub struct MoveList {
    moves: [Move; 256],
    count: usize,
}
impl Default for MoveList {
    fn default() -> Self {
        MoveList {
            moves: [Move(0); 256],
            count: 0,
        }
    }
}
impl MoveList {
    pub fn push(&mut self, mv: Move) {
        // safe because there will be never more than 256 moves per depth
        debug_assert!(self.count < 256);
        unsafe {
            *self.moves.get_unchecked_mut(self.count) = mv;
            self.count += 1;
        }
    }
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[0..self.count]
    }
}
pub struct MoveGenerator {
    rook_atk: Vec<BitBoard>,
    bishop_atk: Vec<BitBoard>,
}
impl MoveGenerator {
    pub fn generate_moves(&self, move_list: &mut MoveList, board: &Board) {
        match board.side_to_move {
            Color::Black => self.generate_moves_generic::<Black>(move_list, board),
            Color::White => self.generate_moves_generic::<White>(move_list, board),
        }
    }
    pub fn generate_moves_generic<S: Side + PawnDirection + Castle>(
        &self,
        move_list: &mut MoveList,
        board: &Board,
    ) {
        self.generate_captures::<S>(move_list, board);
        self.generate_quiets::<S>(move_list, board);
    }
    pub fn generate_captures<S: Side + PawnDirection>(
        &self,
        move_list: &mut MoveList,
        board: &Board,
    ) {
        let color = S::COLOR;
        let pieces = [
            Piece {
                color,
                piece_type: PieceType::Queen,
            },
            Piece {
                color,
                piece_type: PieceType::Rook,
            },
            Piece {
                color,
                piece_type: PieceType::Bishop,
            },
            Piece {
                color,
                piece_type: PieceType::Knight,
            },
            Piece {
                color,
                piece_type: PieceType::King,
            },
        ];
        // captures
        for piece in pieces {
            self.generate_moves_for_piece(
                piece,
                board,
                move_list,
                board.occupied_by_color(S::Opposite::COLOR),
                MoveType::Capture,
            );
        }
        self.generate_capture_pawn_moves::<S>(board, move_list);
    }
    pub fn generate_quiets<S: Side + PawnDirection + Castle>(
        &self,
        move_list: &mut MoveList,
        board: &Board,
    ) {
        let color = S::COLOR;
        let pieces = [
            Piece {
                color,
                piece_type: PieceType::Queen,
            },
            Piece {
                color,
                piece_type: PieceType::Rook,
            },
            Piece {
                color,
                piece_type: PieceType::Bishop,
            },
            Piece {
                color,
                piece_type: PieceType::Knight,
            },
            Piece {
                color,
                piece_type: PieceType::King,
            },
        ];
        for piece in pieces {
            self.generate_moves_for_piece(
                piece,
                board,
                move_list,
                BitBoard(!board.all_occupied().0),
                MoveType::Quiet,
            );
        }
        self.generate_castle_moves::<S>(board, move_list);
        self.generate_quiet_pawn_moves::<S>(board, move_list);
    }
    fn init_rook_atk_table() -> Vec<BitBoard> {
        let mut table = vec![BitBoard(0); 4096 * 64];
        let mut i = 0;
        while i < 64 {
            let square = Square::from_u8_unchecked(i);
            let mask = ROOK_MASK_TABLE[i as usize];
            let bits_in_mask = mask.0.count_ones() as u8;
            let mut variant = 0u32;
            while variant < (1 << bits_in_mask) {
                let occupancy = Occupancy::get_nth_occupancy_for_mask(mask, variant, bits_in_mask);

                let attack = generate_rook_attacks(square, occupancy);

                let index = unsafe { _pext_u64(occupancy.0, mask.0) };
                table[i as usize * 4096 + index as usize] = attack;

                variant += 1;
            }
            i += 1;
        }
        table
    }
    fn init_bishop_atk_table() -> Vec<BitBoard> {
        let mut table = vec![BitBoard(0); 512 * 64];
        let mut i = 0;
        while i < 64 {
            let square = Square::from_u8_unchecked(i);
            let mask = BISHOP_MASK_TABLE[i as usize];
            let bits_in_mask = mask.0.count_ones() as u8;
            let mut variant = 0u32;
            while variant < (1 << bits_in_mask) {
                let occupancy = Occupancy::get_nth_occupancy_for_mask(mask, variant, bits_in_mask);

                let attack = generate_bishop_attacks(square, occupancy);

                let index = unsafe { _pext_u64(occupancy.0, mask.0) };
                table[i as usize * 512 + index as usize] = attack;

                variant += 1;
            }
            i += 1;
        }
        table
    }
    pub fn get_rook_atk(&self, square: Square, occupancy: BitBoard) -> BitBoard {
        let mask = ROOK_MASK_TABLE[square as usize];

        let index = unsafe { _pext_u64(occupancy.0, mask.0) };
        let offset_index = (square as u64 * 4096 + index as u64) as usize;

        self.rook_atk[offset_index]
    }
    pub fn get_bishop_atk(&self, square: Square, occupancy: BitBoard) -> BitBoard {
        let mask = BISHOP_MASK_TABLE[square as usize];

        let index = unsafe { _pext_u64(occupancy.0, mask.0) };
        let offset_index = (square as u64 * 512 + index as u64) as usize;

        self.bishop_atk[offset_index]
    }
    pub fn generate_moves_for_piece(
        &self,
        piece: Piece,
        board: &Board,
        move_list: &mut MoveList,
        target: BitBoard,
        move_type: MoveType,
    ) {
        let friends = board.occupied_by_color(piece.color);
        let squares = board.get_pieces(piece.piece_type, piece.color);
        for square in squares {
            let attacks = match piece.piece_type {
                PieceType::Bishop => self.get_bishop_atk(square, board.all_occupied()),
                PieceType::King => KING_ATK_TABLE[square as usize],
                PieceType::Knight => KNIGHT_ATK_TABLE[square as usize],
                PieceType::Pawn => return, // need more complex logic for pawn
                PieceType::Rook => self.get_rook_atk(square, board.all_occupied()),
                PieceType::Queen => BitBoard(
                    self.get_rook_atk(square, board.all_occupied()).0
                        | self.get_bishop_atk(square, board.all_occupied()).0,
                ),
            };
            let filtered_attacks = BitBoard(attacks.0 & !friends.0);
            self.fill_moves(
                square,
                filtered_attacks,
                move_list,
                target,
                move_type,
                piece.piece_type,
                piece,
            );
        }
    }
    pub fn generate_castle_moves<S: Side + Castle>(&self, board: &Board, move_list: &mut MoveList) {
        let occupied = board.all_occupied();
        let castling_rights = board.castling_rights();
        if castling_rights.k_for_color(S::COLOR)
            && occupied.0 & S::KING_SIDE.0 == 0
            && !self.is_king_in_check(&board, S::COLOR)
            && !self.is_square_attacked::<S>(&board, S::KING_SIDE_ROOK_POS)
            && !self.is_square_attacked::<S>(&board, S::KING_SIDE_KING_POS)
        {
            move_list.push(Move::new(
                S::KING_START_POS,
                S::KING_SIDE_KING_POS,
                MoveType::KingSideCastle,
            ));
        }
        if castling_rights.q_for_color(S::COLOR)
            && occupied.0 & S::QUEEN_SIDE.0 == 0
            && !self.is_king_in_check(&board, S::COLOR)
            && !self.is_square_attacked::<S>(&board, S::QUEEN_SIDE_ROOK_POS)
            && !self.is_square_attacked::<S>(&board, S::QUEEN_SIDE_KING_POS)
        {
            move_list.push(Move::new(
                S::KING_START_POS,
                S::QUEEN_SIDE_KING_POS,
                MoveType::QueenSideCastle,
            ));
        }
    }
    #[inline(always)]
    fn fill_moves(
        &self,
        from_square: Square,
        attacks: BitBoard,
        move_list: &mut MoveList,
        target: BitBoard,
        move_type: MoveType,
        piece_type: PieceType,
        piece: Piece,
    ) {
        let target_squares = BitBoard(attacks.0 & target.0);
        for to_square in target_squares {
            move_list.push(Move::new(from_square, to_square, move_type));
        }
    }

    fn generate_quiet_pawn_moves<S: Side>(&self, board: &Board, move_list: &mut MoveList) {
        // normal move forward
        let color = S::COLOR;
        let pawns = board.get_pieces(PieceType::Pawn, color);
        let empty = !board.all_occupied().0;
        let single_push = BitBoard((S::shift(pawns)).0 & empty);
        let double_push = BitBoard(
            S::shift(BitBoard(
                S::shift(BitBoard(pawns.0 & S::STARTING_RANK as u64)).0 & empty,
            ))
            .0 & empty,
        );
        let promotion = BitBoard((single_push.0 & S::PROMOTION_RANK as u64) & empty);
        let not_promotion = BitBoard((single_push.0 & !S::PROMOTION_RANK as u64) & empty);
        // promotions
        for square in promotion {
            move_list.push(Move::new(
                S::get_source_single(square),
                square,
                MoveType::BishopPromotion,
            ));
            move_list.push(Move::new(
                S::get_source_single(square),
                square,
                MoveType::RookPromotion,
            ));
            move_list.push(Move::new(
                S::get_source_single(square),
                square,
                MoveType::KnightPromotion,
            ));
            move_list.push(Move::new(
                S::get_source_single(square),
                square,
                MoveType::QueenPromotion,
            ));
        }
        // double push
        for square in double_push {
            move_list.push(Move::new(
                S::get_source_double(square),
                square,
                MoveType::DoublePush,
            ));
        }
        // single push
        for square in not_promotion {
            move_list.push(Move::new(
                S::get_source_single(square),
                square,
                MoveType::Quiet,
            ));
        }
    }
    pub fn get_pawn_attack<S: Side>(square: Square) -> BitBoard {
        PAWN_ATK_TABLE[S::COLOR as usize][square as usize]
    }
    fn generate_capture_pawn_moves<S: Side>(&self, board: &Board, move_list: &mut MoveList) {
        let color = S::COLOR;
        let enemies = board.occupied_by_color(S::Opposite::COLOR);
        let pawns = board.get_pieces(PieceType::Pawn, color);
        for pawn_square in pawns {
            let filttered_attacks = BitBoard(
                MoveGenerator::get_pawn_attack::<S>(pawn_square).0
                    & !board.occupied_by_color(S::COLOR).0,
            );
            let attacks = filttered_attacks & enemies;
            let promotions = BitBoard(attacks.0 & S::PROMOTION_RANK as u64);
            let not_promotions = BitBoard(attacks.0 & !S::PROMOTION_RANK as u64);
            if let Some(en_passant_square) = board.en_passant_square() {
                if (BitBoard(1u64 << en_passant_square as u64) & filttered_attacks).0 != 0 {
                    move_list.push(Move::new(
                        pawn_square,
                        en_passant_square,
                        MoveType::EnPassant,
                    ));
                }
            }
            for attack in promotions {
                move_list.push(Move::new(
                    pawn_square,
                    attack,
                    MoveType::QueenCapturePromotion,
                ));
                move_list.push(Move::new(
                    pawn_square,
                    attack,
                    MoveType::RookCapturePromotion,
                ));
                move_list.push(Move::new(
                    pawn_square,
                    attack,
                    MoveType::KnightCapturePromotion,
                ));
                move_list.push(Move::new(
                    pawn_square,
                    attack,
                    MoveType::BishopCapturePromotion,
                ));
            }
            for attack in not_promotions {
                move_list.push(Move::new(pawn_square, attack, MoveType::Capture));
            }
        }
    }
    pub fn is_king_in_check(&self, board: &Board, color: Color) -> bool {
        let king = Square::from_u8_unchecked(
            board.pieces[color][PieceType::King].0.trailing_zeros() as u8,
        );
        match color {
            Color::White => self.is_square_attacked::<White>(board, king),
            Color::Black => self.is_square_attacked::<Black>(board, king),
        }
    }
    #[inline(always)]
    pub fn is_square_attacked<S: Side>(&self, board: &Board, sq: Square) -> bool {
        let attacker_color = S::Opposite::COLOR;
        if (MoveGenerator::get_pawn_attack::<S>(sq).0
            & board.pieces[attacker_color][PieceType::Pawn].0)
            != 0
        {
            return true;
        }

        if (KNIGHT_ATK_TABLE[sq].0 & board.pieces[attacker_color][PieceType::Knight].0) != 0 {
            return true;
        }

        let occupancy = board.all_occupied();
        if (self.get_bishop_atk(sq, occupancy)
            & (board.pieces[attacker_color][PieceType::Bishop]
                | board.pieces[attacker_color][PieceType::Queen]))
            .0
            != 0
        {
            return true;
        }

        if (self.get_rook_atk(sq, occupancy)
            & (board.pieces[attacker_color][PieceType::Rook]
                | board.pieces[attacker_color][PieceType::Queen]))
            .0
            != 0
        {
            return true;
        }

        if (KING_ATK_TABLE[sq] & board.pieces[attacker_color][PieceType::King]).0 != 0 {
            return true;
        }

        false
    }
}

impl Default for MoveGenerator {
    fn default() -> Self {
        let rook_atk = MoveGenerator::init_rook_atk_table();
        let bishop_atk = MoveGenerator::init_bishop_atk_table();
        Self {
            rook_atk,
            bishop_atk,
        }
    }
}
pub struct Occupancy; // method aggregate for dealing with occupancy
impl Occupancy {
    pub const fn get_nth_occupancy_for_mask(
        mut mask: BitBoard,
        nth_variant: u32,
        bits_in_mask: u8,
    ) -> BitBoard {
        let mut occupancy = BitBoard(0);
        let mut nth_variant_bit = 0;
        while nth_variant_bit < bits_in_mask {
            let index_in_mask = mask.0.trailing_zeros() as u8;
            let square = Square::from_u8_unchecked(index_in_mask);
            mask.unset_bit(square);
            if (nth_variant & (1 << nth_variant_bit)) != 0 {
                occupancy.set_bit(square);
            }
            nth_variant_bit += 1;
        }
        occupancy
    }
    pub const fn generate_slider_mask_tbl(piece: PieceType) -> [BitBoard; 64] {
        let mut attack_table: [BitBoard; 64] = [BitBoard(0); 64];
        let mut i = 0u8;
        while i < 64u8 {
            let square = Square::from_u8_unchecked(i);
            attack_table[i as usize] = match piece {
                PieceType::Rook => generate_rook_mask(square),
                PieceType::Bishop => generate_bishop_mask(square),
                _ => panic!("Not a leaper."),
            };
            i += 1;
        }
        attack_table
    }
}
#[cfg(test)]
mod tests {

    use crate::{
        ai::evaluation::Evaluation,
        board::bitboard::Square,
        board::board::{Board, Color, PieceType},
        engine::{make_move, undo_move},
        moves::{
            move_generator::{Move, MoveGenerator, MoveList, MoveType},
            sliders::{BISHOP_MASK_TABLE, ROOK_MASK_TABLE},
            traits::{Black, Castle, PawnDirection, Side, White},
        },
        performance::perft_entry,
    };
    #[test]
    fn test_move_generation_perft_starting_board() {
        let mut board = Board::default();
        let mut board_2 = Board::from_FEN(
            "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_string(),
        );
        println!("{board_2}");
        //let mv = Move::new(Square::E2, Square::E4, MoveType::Quiet);
        //make_move::<White>(&mut board, mv);
        //let mv = Move::new(Square::D7, Square::D5, MoveType::DoublePush);
        //make_move::<Black>(&mut board, mv);
        //let mv = Move::new(Square::E1, Square::E2, MoveType::Quiet);
        //make_move::<White>(&mut board, mv);
        println!("{board}");
        //assert_eq!(8902, perft_entry(&mut board, 3));
        //assert_eq!(197281, perft_entry(&mut board, 4));
        //assert_eq!(4865609, perft_divide::<White>(&mut board, 5));
        //assert_eq!(4865609, perft_divide::<White>(&mut board, 5));
        assert_eq!(119060324, perft_entry(&mut board, 6));
    }
    #[test]
    fn test_is_king_in_check() {
        let move_generator = MoveGenerator::default();
        let board = Board::from_FEN("4k3/8/8/8/8/8/8/4K2r w - - 0 1".to_string());
        let king_sq = Square::from_u8_unchecked(
            board
                .get_pieces(PieceType::King, Color::White)
                .0
                .trailing_zeros() as u8,
        );
        let is_attacked = move_generator.is_square_attacked::<White>(&board, king_sq);
        println!("{}", board);
        assert!(is_attacked);
        let board = Board::from_FEN("8/8/8/8/8/2n5/8/1K6 w - - 0 1".to_string());
        let king_sq = Square::from_u8_unchecked(
            board
                .get_pieces(PieceType::King, Color::White)
                .0
                .trailing_zeros() as u8,
        );
        let is_attacked = move_generator.is_square_attacked::<White>(&board, king_sq);
        assert!(is_attacked);
        let board = Board::from_FEN("8/6b1/8/8/3K4/8/8/8 w - - 0 1".to_string());
        let king_sq = Square::from_u8_unchecked(
            board
                .get_pieces(PieceType::King, Color::White)
                .0
                .trailing_zeros() as u8,
        );
        let is_attacked = move_generator.is_square_attacked::<White>(&board, king_sq);
        assert!(is_attacked);
        let board = Board::from_FEN("8/8/8/q3K3/8/8/8/8 w - - 0 1".to_string());
        let king_sq = Square::from_u8_unchecked(
            board
                .get_pieces(PieceType::King, Color::White)
                .0
                .trailing_zeros() as u8,
        );
        let is_attacked = move_generator.is_square_attacked::<White>(&board, king_sq);
        assert!(is_attacked);
        let board = Board::from_FEN("8/8/8/q1P1K3/8/8/8/8 w - - 0 1".to_string());
        let king_sq = Square::from_u8_unchecked(
            board
                .get_pieces(PieceType::King, Color::White)
                .0
                .trailing_zeros() as u8,
        );
        let is_attacked = move_generator.is_square_attacked::<White>(&board, king_sq);
        assert!(!is_attacked);
    }
    #[test]
    fn test_move_constructor() {
        let test_move = Move::new(Square::B1, Square::C1, MoveType::Capture);
        let expected_move = Move(0b0001000010000001); // 0001 - capture, 000010 - 3rd square (c1), 000001 - 2nd square (b1)
        println!("{:0>16b}", test_move.0);
        assert!(test_move == expected_move)
    }
    #[test]
    fn test_starting_position_move_count() {
        // Setup
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let mut move_list = MoveList::default();

        // Act
        move_generator.generate_quiets::<White>(&mut move_list, &board);
        move_generator.generate_captures::<White>(&mut move_list, &board);

        // Assert
        assert_eq!(
            move_list.as_slice().len(),
            20,
            "Starting position should have exactly 20 moves"
        );
    }
}
#[cfg(test)]
mod debug_tests {
    use crate::{
        board::bitboard::BitBoard,
        board::board::{Board, Color},
        engine::{make_move, undo_move},
        moves::{
            leapers::KNIGHT_ATK_TABLE,
            sliders::{BISHOP_MASK_TABLE, ROOK_MASK_TABLE},
            traits::{Black, White},
        },
    };

    use super::{Move, MoveGenerator, MoveList, MoveType, Occupancy, Square};
    #[test]
    #[ignore]
    fn test_rook_moves_generation() {
        let square = Square::A1;
        let move_gen = MoveGenerator::default();
        let variant = 1;
        let mask = ROOK_MASK_TABLE[square as usize];
        let bits_in_mask = mask.0.count_ones() as u8;
        let occupancy = Occupancy::get_nth_occupancy_for_mask(mask, variant, bits_in_mask);
        let atk = move_gen.get_rook_atk(square, occupancy);
        println!("{}", atk);
        assert!(false)
    }
    #[test]
    #[ignore]
    fn test_bishop_moves_generation() {
        let square = Square::D5;
        let move_gen = MoveGenerator::default();
        let variant = 321;
        let mask = BISHOP_MASK_TABLE[square as usize];
        let bits_in_mask = mask.0.count_ones() as u8;
        let occupancy = Occupancy::get_nth_occupancy_for_mask(mask, variant, bits_in_mask);
        let atk = move_gen.get_bishop_atk(square, occupancy);
        println!("{}", atk);
        assert!(false)
    }
    #[test]
    #[ignore]
    fn test_move_generation_debug() {
        let mut board = Board::from_FEN(
            "rbnqkbnr/1ppppppp/8/p7/8/1N6/PPPPPPPP/RB1QKBNR w KQkq - 0 1".to_string(),
        );
        println!("{board}");
        let move_generator = MoveGenerator::default();
        let mut move_list = MoveList::default();
        move_generator.generate_moves_generic::<White>(&mut move_list, &board);
        let moves = move_list.as_slice();
        let capture = moves[0];
        for mv in moves {
            println!("{mv}");
        }
        let undo = make_move::<White>(&mut board, capture);
        undo_move::<White>(capture, &mut board, undo);
        println!("{board}");
        assert!(false);
    }
    #[test]
    #[ignore]
    fn test_move_generation() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let mut move_list = MoveList::default();
        move_generator.generate_quiets::<White>(&mut move_list, &board);
        let moves = move_list.as_slice();
        for mv in moves {
            println!("{mv}");
        }
        assert!(false)
    }
    #[test]
    fn test_castling_generation() {
        let board = Board::from_FEN("4k3/8/8/8/8/8/8/4K2R w K - 0 1".to_string());
        println!("{}", board);
        let move_generator = MoveGenerator::default();
        let mut move_list = MoveList::default();
        move_generator.generate_quiets::<White>(&mut move_list, &board);
        let moves = move_list.as_slice();
        for (i, mv) in moves.iter().enumerate() {
            println!("{i}: {mv}");
        }
        assert!(false);
    }
    #[test]
    fn debug_test() {
        let board = Board::from_FEN("8/8/8/3pP3/8/8/8/8 w - d6 0 1".to_string());
        println!("{}", board);
        let move_generator = MoveGenerator::default();
        let mut move_list = MoveList::default();
        let attacks = BitBoard(MoveGenerator::get_pawn_attack::<White>(Square::E5).0);
        println!("{attacks}");
        println!("{}", board.en_passant_square().unwrap());
        let wtf = BitBoard(1u64 << board.en_passant_square().unwrap() as u8);
        println!("{wtf}");
        move_generator.generate_moves_generic::<White>(&mut move_list, &board);
        let moves = move_list.as_slice();
        for (i, mv) in moves.iter().enumerate() {
            println!("{i}: {mv}");
        }
        assert!(false);
    }
}
