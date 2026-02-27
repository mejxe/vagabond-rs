use std::arch::x86_64::_pext_u64;

use crate::{
    bitboard::{BitBoard, Square},
    board::PieceType,
    moves::sliders::{
        BISHOP_MASK_TABLE, ROOK_MASK_TABLE, generate_bishop_attacks, generate_bishop_mask,
        generate_rook_attacks, generate_rook_mask,
    },
};

pub struct MoveGenerator {
    rook_atk: Vec<BitBoard>,
    bishop_atk: Vec<BitBoard>,
}
impl MoveGenerator {
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

    use super::{MoveGenerator, Occupancy, Square};
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
}
