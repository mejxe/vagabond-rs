use std::arch::x86_64::_pext_u64;

use crate::{
    bitboard::{self, BitBoard, Square},
    board::{Board, Color, Piece, PieceType},
    moves::sliders::{
        BISHOP_MASK_TABLE, ROOK_MASK_TABLE, generate_bishop_attacks, generate_bishop_mask,
        generate_rook_attacks, generate_rook_mask,
    },
};

use super::{
    leapers::{B_PAWN_ATK_TABLE, KING_ATK_TABLE, KNIGHT_ATK_TABLE, W_PAWN_ATK_TABLE},
    traits::{Castle, PawnDirection, Side},
};

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
}
#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Copy, Clone)]
pub enum MoveType {
    Quiet,
    Capture,
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
    QueenCaptureCapturePromotion,
}
pub struct MoveList {
    moves: [Move; 256],
    count: usize,
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
}
pub struct MoveGenerator {
    rook_atk: Vec<BitBoard>,
    bishop_atk: Vec<BitBoard>,
}
impl MoveGenerator {
    fn fill_captures<S: Side + PawnDirection>(&self, move_list: &mut MoveList, board: &Board) {
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
                board.occupied_by_color(color),
                MoveType::Capture,
            );
        }
        self.generate_capture_pawn_moves::<S>(color, board, move_list);
        // TODO:  rate them and return if beta cutoff found
        // if not get quiets
    }
    fn fill_quiets<S: Side + PawnDirection + Castle>(
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
                board.occupied_by_color(color),
                MoveType::Quiet,
            );
        }
        self.generate_castle_moves::<S>(color, board, move_list); // TODO: Refactor to use color from the S trait
        self.generate_quiet_pawn_moves::<S>(color, board, move_list);
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
        dbg!(index);
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
        let squares = board.get_pieces(PieceType::Bishop, piece.color);
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
            self.fill_moves(square, filtered_attacks, move_list, target, move_type);
        }
    }
    pub fn generate_castle_moves<C: Castle>(
        &self,
        color: Color,
        board: &Board,
        move_list: &mut MoveList,
    ) {
        if !board.get_castling_rights(color) {
            return;
        };
        let occupied = board.all_occupied();
        if occupied.0 & !C::KING_SIDE.0 == 0 {
            move_list.push(Move::new(
                C::KING_START_POS,
                C::KING_SIDE_KING_POS,
                MoveType::KingSideCastle,
            ));
        }
        if occupied.0 & !C::QUEEN_SIDE.0 == 0 {
            move_list.push(Move::new(
                C::KING_START_POS,
                C::QUEEN_SIDE_KING_POS,
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
    ) {
        let target_squares = BitBoard(attacks.0 & target.0);
        for to_square in target_squares {
            move_list.push(Move::new(from_square, to_square, move_type));
        }
    }

    fn generate_quiet_pawn_moves<Dir: PawnDirection>(
        &self,
        color: Color,
        board: &Board,
        move_list: &mut MoveList,
    ) {
        // normal move forward
        let pawns = board.get_pieces(PieceType::Pawn, color);
        let empty = !board.all_occupied().0;
        let single_push = BitBoard((Dir::shift(pawns)).0 & empty);
        let double_push = BitBoard(
            Dir::shift(BitBoard(
                Dir::shift(BitBoard(pawns.0 & Dir::STARTING_RANK as u64)).0 & empty,
            ))
            .0 & empty,
        );
        let promotion = BitBoard((single_push.0 & Dir::PROMOTION_RANK as u64) & empty);
        let not_promotion = BitBoard((single_push.0 & !Dir::PROMOTION_RANK as u64) & empty);
        // promotions
        for square in promotion {
            move_list.push(Move::new(
                Dir::get_source_double(square),
                square,
                MoveType::QueenPromotion,
            ));
            move_list.push(Move::new(
                Dir::get_source_double(square),
                square,
                MoveType::RookPromotion,
            ));
            move_list.push(Move::new(
                Dir::get_source_double(square),
                square,
                MoveType::KnightPromotion,
            ));
            move_list.push(Move::new(
                Dir::get_source_double(square),
                square,
                MoveType::QueenPromotion,
            ));
        }
        // double push
        for square in double_push {
            move_list.push(Move::new(
                Dir::get_source_double(square),
                square,
                MoveType::Quiet,
            ));
        }
        // single push
        for square in not_promotion {
            move_list.push(Move::new(
                Dir::get_source_single(square),
                square,
                MoveType::Quiet,
            ));
        }
    }
    fn generate_capture_pawn_moves<Dir: PawnDirection>(
        &self,
        color: Color,
        board: &Board,
        move_list: &mut MoveList,
    ) {
        let pawns = board.get_pieces(PieceType::Pawn, color);
        let (attack_table, enemies) = match color {
            Color::White => (W_PAWN_ATK_TABLE, board.black_occupied()),
            Color::Black => (B_PAWN_ATK_TABLE, board.white_occupied()),
        };
        for pawn_square in pawns {
            let attacks = BitBoard(attack_table[pawn_square as usize].0 & enemies.0);
            let promotions = BitBoard(attacks.0 & Dir::PROMOTION_RANK as u64);
            let not_promotions = BitBoard(attacks.0 & !Dir::PROMOTION_RANK as u64);
            if let Some(en_passant_square) = board.en_passant_square() {
                if (1u64 << (en_passant_square as u64)) != 0 {
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
                    MoveType::QueenCaptureCapturePromotion,
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
pub struct Occupancy {} // method aggregate for dealing with occupancy
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

mod tests {
    use crate::moves::sliders::{BISHOP_MASK_TABLE, ROOK_MASK_TABLE};

    use super::{Move, MoveGenerator, MoveType, Occupancy, Square};
    #[test]
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
    fn test_move_constructor() {
        let test_move = Move::new(Square::B1, Square::C1, MoveType::Capture);
        let expected_move = Move(0b0001000010000001); // 0001 - capture, 000010 - 3rd square (c1), 000001 - 2nd square (b1)
        println!("{:0>16b}", test_move.0);
        assert!(test_move == expected_move)
    }
}
