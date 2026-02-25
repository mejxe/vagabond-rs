use crate::bitboard::BitBoard;
use crate::bitboard::Square;
use crate::board::Piece;

pub const ROOK_OCCUPANCY_TABLE: [BitBoard; 64] = generate_slider_occupancy(Piece::Rook);

const fn generate_slider_occupancy(piece: Piece) -> [BitBoard; 64] {
    let mut attack_table: [BitBoard; 64] = [BitBoard(0); 64];
    let mut i = 0u8;
    while i < 64u8 {
        let square = Square::from_u8_unchecked(i);
        attack_table[i as usize] = match piece {
            Piece::Rook => generate_rook_occupancy(square),
            Piece::Bishop => generate_bishop_occupancy(square),
            _ => panic!("Not a leaper."),
        };
        i += 1;
    }
    attack_table
}

pub const fn generate_rook_occupancy(square: Square) -> BitBoard {
    let mut attacks = BitBoard(0);
    let mut piece_pos = BitBoard(0);
    piece_pos.set_bit(square);
    let mut r = (square as u8) / 8;
    let mut c = (square as u8) % 8;

    while r < 7 {
        attacks.set_bit(Square::from_u8_unchecked(r * 8 + c));
        r += 1;
    }
    r = (square as u8) / 8;
    while r > 0 {
        attacks.set_bit(Square::from_u8_unchecked(r * 8 + c));
        r -= 1;
    }
    r = (square as u8) / 8;
    while c < 7 {
        attacks.set_bit(Square::from_u8_unchecked(r * 8 + c));
        c += 1;
    }
    c = (square as u8) % 8;
    while c > 0 {
        attacks.set_bit(Square::from_u8_unchecked(r * 8 + c));
        c -= 1;
    }
    attacks
}
pub const fn generate_bishop_occupancy(square: Square) -> BitBoard {
    let mut attacks = BitBoard(0);
    let mut piece_pos = BitBoard(0);
    piece_pos.set_bit(square);
    let mut r = (square as u8) / 8;
    let mut c = (square as u8) % 8;
    // ++
    while r < 7 && c < 7 {
        attacks.set_bit(Square::from_u8_unchecked(r * 8 + c));
        r += 1;
        c += 1;
    }
    //+-
    r = (square as u8) / 8;
    c = (square as u8) % 8;
    while r < 7 && c > 0 {
        attacks.set_bit(Square::from_u8_unchecked(r * 8 + c));
        r += 1;
        c -= 1;
    }
    r = (square as u8) / 8;
    c = (square as u8) % 8;
    // -+
    while r > 0 && c < 7 {
        attacks.set_bit(Square::from_u8_unchecked(r * 8 + c));
        r -= 1;
        c += 1;
    }
    r = (square as u8) / 8;
    c = (square as u8) % 8;
    // --
    while r > 0 && c > 0 {
        attacks.set_bit(Square::from_u8_unchecked(r * 8 + c));
        r -= 1;
        c -= 1;
    }
    attacks
}
