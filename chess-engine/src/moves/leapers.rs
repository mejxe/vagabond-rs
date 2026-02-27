use crate::bitboard::BitBoard;
use crate::bitboard::Square;
use crate::board::Color;
use crate::board::PieceType;

pub const NOT_A_COLUMN: BitBoard = {
    let mut tbl = BitBoard(u64::MAX);
    let mut i = 0u8;
    while i < 64u8 {
        let square = Square::from_u8_unchecked(i);
        tbl.unset_bit(square);
        i += 8;
    }
    tbl
};

pub const NOT_H_COLUMN: BitBoard = {
    let mut tbl = BitBoard(u64::MAX);
    let mut i = 7u8;
    while i < 64u8 {
        let square = Square::from_u8_unchecked(i);
        tbl.unset_bit(square);
        i += 8;
    }
    tbl
};
pub const NOT_AB_COLUMN: BitBoard = {
    let mut tbl = BitBoard(u64::MAX);
    let mut i = 0u8;
    while i < 64u8 {
        let square_a = Square::from_u8_unchecked(i);
        let square_b = Square::from_u8_unchecked(i + 1);
        tbl.unset_bit(square_a);
        tbl.unset_bit(square_b);
        i += 8;
    }
    tbl
};
pub const NOT_GH_COLUMN: BitBoard = {
    let mut tbl = BitBoard(u64::MAX);
    let mut i = 6u8;
    while i < 64u8 {
        let square_g = Square::from_u8_unchecked(i);
        let square_h = Square::from_u8_unchecked(i + 1);
        tbl.unset_bit(square_g);
        tbl.unset_bit(square_h);
        i += 8;
    }
    tbl
};
pub const NOT_1ST_ROW: BitBoard = {
    let mut tbl = BitBoard(u64::MAX);
    let mut i = 0u8;
    while i < 8 {
        let square = Square::from_u8_unchecked(i);
        tbl.unset_bit(square);
        i += 1;
    }
    tbl
};
pub const NOT_8TH_ROW: BitBoard = {
    let mut tbl = BitBoard(u64::MAX);
    let mut i = 63u8 - 7;
    while i < 64 {
        let square = Square::from_u8_unchecked(i);
        tbl.unset_bit(square);
        i += 1;
    }
    tbl
};
pub const NOT_1ST2ND_ROW: BitBoard = {
    let mut tbl = BitBoard(u64::MAX);
    let mut i = 0u8;
    while i < 16 {
        let square = Square::from_u8_unchecked(i);
        tbl.unset_bit(square);
        i += 1;
    }
    tbl
};
pub const NOT_7TH8TH_ROW: BitBoard = {
    let mut tbl = BitBoard(u64::MAX);
    let mut i = 63u8 - 15;
    while i < 64 {
        let square = Square::from_u8_unchecked(i);
        tbl.unset_bit(square);
        i += 1;
    }
    tbl
};

const fn generate_knight_atk(square: Square) -> BitBoard {
    let mut piece_pos = BitBoard(0);
    let mut attacks = BitBoard(0);
    piece_pos.set_bit(square);
    // +- 6, 15, 17, 10
    // FORWARD
    attacks.mask((piece_pos.0 << 17) & NOT_A_COLUMN.0);
    attacks.mask((piece_pos.0 << 10) & NOT_AB_COLUMN.0);
    attacks.mask((piece_pos.0 << 15) & NOT_H_COLUMN.0);
    attacks.mask((piece_pos.0 << 6) & NOT_GH_COLUMN.0);
    // BACKWARD
    attacks.mask((piece_pos.0 >> 17) & NOT_H_COLUMN.0);
    attacks.mask((piece_pos.0 >> 10) & NOT_GH_COLUMN.0);
    attacks.mask((piece_pos.0 >> 15) & NOT_A_COLUMN.0);
    attacks.mask((piece_pos.0 >> 6) & NOT_AB_COLUMN.0);
    attacks
}

const fn generate_king_atk(square: Square) -> BitBoard {
    let mut piece_pos = BitBoard(0);
    let mut attacks = BitBoard(0);
    piece_pos.set_bit(square);
    // +- 7,8,9,1
    // FORWARD
    attacks.mask((piece_pos.0 << 7) & NOT_H_COLUMN.0);
    attacks.mask(piece_pos.0 << 8);
    attacks.mask((piece_pos.0 << 9) & NOT_A_COLUMN.0);
    attacks.mask((piece_pos.0 << 1) & NOT_A_COLUMN.0);
    // BACKWARD
    attacks.mask((piece_pos.0 >> 7) & NOT_A_COLUMN.0);
    attacks.mask(piece_pos.0 >> 8);
    attacks.mask((piece_pos.0 >> 9) & NOT_H_COLUMN.0);
    attacks.mask((piece_pos.0 >> 1) & NOT_H_COLUMN.0);
    attacks
}
pub const fn generate_pawn_atk(color: Color, square: Square) -> BitBoard {
    let mut piece_pos = BitBoard(0);
    let mut attack = BitBoard(0);
    piece_pos.set_bit(square);

    match color {
        Color::Black => {
            attack.mask((piece_pos.0 >> 7) & NOT_A_COLUMN.0);
            attack.mask((piece_pos.0 >> 9) & NOT_H_COLUMN.0);
        }

        Color::White => {
            attack.mask((piece_pos.0 << 7) & NOT_H_COLUMN.0);
            attack.mask((piece_pos.0 << 9) & NOT_A_COLUMN.0);
        }
    };
    attack
}

pub const fn generate_leaper_table(piece: PieceType, color: Option<Color>) -> [BitBoard; 64] {
    let mut attack_table: [BitBoard; 64] = [BitBoard(0); 64];
    let mut i = 0u8;
    while i < 64u8 {
        let square = Square::from_u8_unchecked(i);
        attack_table[i as usize] = match piece {
            PieceType::Knight => generate_knight_atk(square),
            PieceType::King => generate_king_atk(square),
            PieceType::Pawn => {
                let color = color.expect("should be provided");
                generate_pawn_atk(color, square)
            }
            _ => panic!("Not a leaper."),
        };
        i += 1;
    }
    attack_table
}
pub const KING_ATK_TABLE: [BitBoard; 64] = generate_leaper_table(PieceType::King, None);
pub const KNIGHT_ATK_TABLE: [BitBoard; 64] = generate_leaper_table(PieceType::Knight, None);
pub const W_PAWN_ATK_TABLE: [BitBoard; 64] =
    generate_leaper_table(PieceType::Pawn, Some(Color::White));
pub const B_PAWN_ATK_TABLE: [BitBoard; 64] =
    generate_leaper_table(PieceType::Pawn, Some(Color::Black));
