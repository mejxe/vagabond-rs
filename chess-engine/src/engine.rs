use crate::{ai::ai::AI, board::Board, moves::move_generator::MoveGenerator};

pub struct Engine {
    board: Board,
    move_gen: MoveGenerator,
    ai: AI,
}
impl Default for Engine {
    fn default() -> Self {
        Self {
            board: Board::default(),
            move_gen: MoveGenerator::default(),
            ai: AI {},
        }
    }
}
