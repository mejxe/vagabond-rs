use chess_engine::{
    bitboard::{BitBoard, Square},
    moves::sliders::{generate_bishop_occupancy, generate_rook_occupancy},
};

fn main() {
    println!("{}", generate_bishop_occupancy(Square::E4))
}
