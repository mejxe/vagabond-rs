use std::arch::x86_64::_pext_u64;
use std::sync::OnceLock;

use crate::board::bitboard::BitBoard;
use crate::board::bitboard::Square;
use crate::board::board::PieceType;

use super::move_generator::Occupancy;

pub const ROOK_MASK_TABLE: [BitBoard; 64] = Occupancy::generate_slider_mask_tbl(PieceType::Rook);
pub const BISHOP_MASK_TABLE: [BitBoard; 64] =
    Occupancy::generate_slider_mask_tbl(PieceType::Bishop);
pub static BISHOP_ATK_TABLE: OnceLock<Vec<BitBoard>> = OnceLock::new();
pub static ROOK_ATK_TABLE: OnceLock<Vec<BitBoard>> = OnceLock::new();

pub const CASTLING_MASK: [u8; 64] = {
    let mut mask = [0xf; 64];
    mask[Square::A1 as usize] = 0b1011;
    mask[Square::H1 as usize] = 0b0111;
    mask[Square::A8 as usize] = 0b1110;
    mask[Square::H8 as usize] = 0b1101;
    mask[Square::E1 as usize] = 0b0011;
    mask[Square::E8 as usize] = 0b1100;
    mask
};
pub const fn generate_rook_mask(square: Square) -> BitBoard {
    let mut attacks = BitBoard(0);
    let mut piece_pos = BitBoard(0);
    piece_pos.set_bit(square);
    let mut c = (square as i8) % 8;

    // up
    let mut r = (square as i8) / 8 + 1;
    while r < 7 {
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        r += 1;
    }
    // down

    r = ((square as i8) / 8) - 1;
    while r > 0 {
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        r -= 1;
    }
    r = (square as i8) / 8;
    // right

    c = (square as i8) % 8 + 1;
    while c < 7 {
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        c += 1;
    }
    // left

    c = ((square as i8) % 8) - 1;
    while c > 0 {
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        c -= 1;
    }
    attacks
}
pub const fn generate_bishop_mask(square: Square) -> BitBoard {
    let mut attacks = BitBoard(0);
    let mut piece_pos = BitBoard(0);
    piece_pos.set_bit(square);
    let mut r = (square as i8) / 8 + 1;
    let mut c = (square as i8) % 8 + 1;
    // ++
    while r < 7 && c < 7 {
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        r += 1;
        c += 1;
    }
    //+-
    r = (square as i8) / 8 + 1;
    c = (square as i8) % 8 - 1;
    while r < 7 && c > 0 {
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        r += 1;
        c -= 1;
    }
    // -+
    r = (square as i8) / 8 - 1;
    c = (square as i8) % 8 + 1;
    while r > 0 && c < 7 {
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        r -= 1;
        c += 1;
    }
    // --
    r = (square as i8) / 8 - 1;
    c = (square as i8) % 8 - 1;
    while r > 0 && c > 0 {
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        r -= 1;
        c -= 1;
    }
    attacks
}
pub const fn generate_rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let mut attacks = BitBoard(0);
    let mut piece_pos = BitBoard(0);
    piece_pos.set_bit(square);
    let mut c = (square as i8) % 8;

    // up
    let mut r = (square as i8) / 8 + 1;
    while r <= 7 {
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        if blockers.check_bit(Square::from_u8_unchecked((r * 8 + c) as u8)) {
            break;
        }
        r += 1;
    }
    // down

    r = ((square as i8) / 8) - 1;
    while r >= 0 {
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        if blockers.check_bit(Square::from_u8_unchecked((r * 8 + c) as u8)) {
            break;
        }
        r -= 1;
    }
    r = (square as i8) / 8;
    // right

    c = (square as i8) % 8 + 1;
    while c <= 7 {
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        if blockers.check_bit(Square::from_u8_unchecked((r * 8 + c) as u8)) {
            break;
        }
        c += 1;
    }
    // left

    c = ((square as i8) % 8) - 1;
    while c >= 0 {
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        if blockers.check_bit(Square::from_u8_unchecked((r * 8 + c) as u8)) {
            break;
        }
        c -= 1;
    }
    attacks
}
pub const fn generate_bishop_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let mut attacks = BitBoard(0);
    let mut piece_pos = BitBoard(0);
    piece_pos.set_bit(square);
    let mut r = ((square as i8) / 8 + 1);
    let mut c = ((square as i8) % 8 + 1);
    // ++
    while r <= 7 && c <= 7 {
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        if blockers.check_bit(Square::from_u8_unchecked((r * 8 + c) as u8)) {
            break;
        }
        r += 1;
        c += 1;
    }
    //+-
    r = (square as i8) / 8 + 1;
    c = (square as i8) % 8 - 1;
    while r <= 7 && c >= 0 {
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        if blockers.check_bit(Square::from_u8_unchecked((r * 8 + c) as u8)) {
            break;
        }
        r += 1;
        c -= 1;
    }
    // -+
    r = (square as i8) / 8 - 1;
    c = (square as i8) % 8 + 1;
    while r >= 0 && c <= 7 {
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        if blockers.check_bit(Square::from_u8_unchecked((r * 8 + c) as u8)) {
            break;
        }
        r -= 1;
        c += 1;
    }
    r = (square as i8) / 8 - 1;
    c = (square as i8) % 8 - 1;
    // --
    while r >= 0 && c >= 0 {
        // useless but readable
        attacks.set_bit(Square::from_u8_unchecked((r * 8 + c) as u8));
        if blockers.check_bit(Square::from_u8_unchecked((r * 8 + c) as u8)) {
            break;
        }
        r -= 1;
        c -= 1;
    }
    attacks
}

const fn generate_pext(val: BitBoard, mut mask: BitBoard) -> u64 {
    let mut ret = 0u64;
    let mut rightmost_empty_bit = 0;
    while mask.0 != 0 {
        let lsb = 1 << mask.0.trailing_zeros();
        mask.0 |= mask.0 - 1;
        if (val.0 & lsb) != 0 {
            ret |= 1 << rightmost_empty_bit;
            rightmost_empty_bit += 1
        }
    }
    ret
}

pub mod debug_tests {
    use std::arch::x86_64::_pext_u64;

    use crate::board::bitboard::BitBoard;
    use crate::board::board::PieceType;
    use crate::moves::move_generator::Occupancy;
    use crate::moves::sliders::ROOK_MASK_TABLE;
    use crate::moves::sliders::generate_bishop_attacks;
    use crate::moves::sliders::generate_bishop_mask;
    use crate::moves::sliders::generate_pext;
    use crate::moves::sliders::generate_rook_mask;

    use super::Square;
    use super::generate_rook_attacks;

    #[test]
    #[ignore]
    fn test_rook_attack() {
        let rook_atk = generate_rook_attacks(Square::A1, BitBoard(0));
        println!("{}", rook_atk);
        let rook_mask = generate_rook_mask(Square::B1);
        println!("{}", rook_mask);
        let bishop_mask = generate_bishop_mask(Square::E5);
        println!("{}", bishop_mask);
        let bishop_atk = generate_bishop_attacks(Square::E5, BitBoard(0));
        println!("{}", bishop_atk);
        assert!(false);
    }
}
