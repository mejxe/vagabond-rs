use chess_engine::{
    bitboard::{BitBoard, Square},
    board::PieceType,
    moves::{
        move_generator::Occupancy,
        sliders::{
            ROOK_MASK_TABLE, generate_bishop_attacks, generate_bishop_mask, generate_rook_attacks,
        },
    },
};

fn main() {
    let mut atk_table = [[BitBoard(0); 4096]; 64];
    let mut i = 0;
    while i < 64 {
        let square = Square::from_u8_unchecked(i);
        let mask = ROOK_MASK_TABLE[i as usize];
        let bits_in_mask = mask.0.count_ones() as u8;
        println!("{}", mask);
        dbg!(bits_in_mask);
        let mut variant = 0u32;
        while variant < (1 << bits_in_mask) {
            let occupancy = Occupancy::get_nth_occupancy_for_mask(mask, variant, bits_in_mask);
            let attack = generate_rook_attacks(square, occupancy);
            println!("{}", attack);
            variant += 1;
        }
        break;
        i += 1;
    }
}
