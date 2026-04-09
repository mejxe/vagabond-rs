use std::sync::OnceLock;

use crate::board::{
    bitboard::Square,
    board::{Board, CastlingRights, Color, Piece, PieceType},
};

use super::prng::PRNG;

pub type ZobristHash = u64;
pub struct ZobristHasher {
    pub(crate) piece_hashes: [[ZobristHash; 12]; 64],
    pub(crate) castle_hashes: [ZobristHash; 16],
    pub(crate) ep_square_hashes: [ZobristHash; 8],
    pub(crate) side_to_move_hash: ZobristHash,
}
impl ZobristHasher {
    fn new(seed: u64) -> ZobristHasher {
        let mut prng = PRNG::new(seed);
        let piece_hashes: [[ZobristHash; 12]; 64] =
            std::array::from_fn(|_| std::array::from_fn(|_| prng.rand_64()));
        let castle_hashes: [ZobristHash; 16] = std::array::from_fn(|_| prng.rand_64());
        let ep_square_hashes: [ZobristHash; 8] = std::array::from_fn(|_| prng.rand_64());
        let side_to_move_hash: ZobristHash = prng.rand_64();
        ZobristHasher {
            piece_hashes,
            side_to_move_hash,
            castle_hashes,
            ep_square_hashes,
        }
    }
    pub fn get_hasher() -> &'static ZobristHasher {
        let seed = 337583;
        ZOBRIST_HASHER.get_or_init(|| ZobristHasher::new(seed))
    }
    pub fn calculate_board_hash(
        &self,
        mailbox: &[Option<Piece>; 64],
        castling: CastlingRights,
        ep_square: Option<Square>,
        side_to_move: Color,
    ) -> ZobristHash {
        let mut hash: ZobristHash = 0;
        // pieces hash
        for sq in 0..64 {
            let sq = Square::from_u8_unchecked(sq);
            if let Some(piece) = mailbox[sq as usize] {
                let piece_index = piece.piece_type as usize + 6 * piece.color as usize;
                hash ^= self.piece_hashes[sq as usize][piece_index];
            }
        }
        // side to move hash
        if side_to_move == Color::Black {
            hash ^= self.side_to_move_hash;
        }
        // castle hash
        hash ^= self.castle_hashes[castling.0 as usize % 16];
        // ep hash
        if let Some(ep_square) = ep_square {
            hash ^= self.ep_square_hashes[ep_square as usize % 8];
        }
        hash
    }
    pub fn update_piece_hash(&self, hash: &mut ZobristHash, piece: Piece, sq: Square) {
        // modifies zobrist hash in place
        let piece_index = piece.piece_type as usize + 6 * piece.color as usize;
        *hash ^= self.piece_hashes[sq as usize][piece_index];
    }
    pub fn flip_zobrist_hash(&self, board: &mut Board) {
        // modifies zobrist hash in place
        board.zobrist ^= self.castle_hashes[board.castling_rights().0 as usize % 16];
        if let Some(ep_square) = board.en_passant_square() {
            board.zobrist ^= self.ep_square_hashes[ep_square as usize % 8];
        }
    }
    pub fn flip_side_to_move_hash(&self, board: &mut Board) {
        board.zobrist ^= self.side_to_move_hash;
    }
}
pub static ZOBRIST_HASHER: OnceLock<ZobristHasher> = OnceLock::new();

#[cfg(test)]
mod tests {
    use crate::{
        board::{bitboard::Square, board::Board},
        engine::{make_move_non_generic, undo_move_non_generic},
        moves::move_structs::{Move, MoveType},
        tt::zobrist::ZobristHasher,
    };
    #[test]
    fn test_symetricity() {
        let mut board = Board::default();

        // A helper closure to test any move and rigorously check for state corruption
        let mut test_move = |b: &mut Board, mv: Move, move_name: &str| {
            let hash_before = b.zobrist;
            // If you have castling rights / en passant fields, save them too

            let undo = make_move_non_generic(b, mv);
            undo_move_non_generic(b, mv, undo);

            assert_eq!(
                hash_before, b.zobrist,
                "ZOBRIST CORRUPTION on move: {}",
                move_name
            );
        };

        // ---------------------------------------------------------
        // 1. QUIET MOVE
        // ---------------------------------------------------------
        test_move(
            &mut board,
            Move::new(Square::E2, Square::E4, MoveType::Quiet),
            "Quiet (e2e4)",
        );

        // ---------------------------------------------------------
        // 2. CAPTURE
        // ---------------------------------------------------------
        // Setup: Play e4, then d5
        let u1 = make_move_non_generic(
            &mut board,
            Move::new(Square::E2, Square::E4, MoveType::Quiet),
        );
        let u2 = make_move_non_generic(
            &mut board,
            Move::new(Square::D7, Square::D5, MoveType::Quiet),
        );

        // Test the capture: exd5
        test_move(
            &mut board,
            Move::new(Square::E4, Square::D5, MoveType::Capture),
            "Capture (exd5)",
        );

        // ---------------------------------------------------------
        // 3. EN PASSANT
        // ---------------------------------------------------------
        // Setup: Push the e-pawn to e5, then Black plays f5 (allowing en passant)
        let u3 = make_move_non_generic(
            &mut board,
            Move::new(Square::E4, Square::E5, MoveType::Quiet),
        );
        let u4 = make_move_non_generic(
            &mut board,
            Move::new(Square::F7, Square::F5, MoveType::Quiet),
        ); // Double pawn push

        // Test En Passant: exf6
        test_move(
            &mut board,
            Move::new(Square::E5, Square::F6, MoveType::EnPassant),
            "En Passant (exf6)",
        );

        // Undo the setup moves to get back to a clean board
        undo_move_non_generic(
            &mut board,
            Move::new(Square::F7, Square::F5, MoveType::Quiet),
            u4,
        );
        undo_move_non_generic(
            &mut board,
            Move::new(Square::E4, Square::E5, MoveType::Quiet),
            u3,
        );
        undo_move_non_generic(
            &mut board,
            Move::new(Square::D7, Square::D5, MoveType::Quiet),
            u2,
        );
        undo_move_non_generic(
            &mut board,
            Move::new(Square::E2, Square::E4, MoveType::Quiet),
            u1,
        );

        // ---------------------------------------------------------
        // 4. CASTLING (Kingside)
        // ---------------------------------------------------------
        // Setup: Clear out the kingside for White (play e4, e5, Nf3, Nc6, Bc4, Nf6)
        let c1 = make_move_non_generic(
            &mut board,
            Move::new(Square::E2, Square::E4, MoveType::Quiet),
        );
        let c2 = make_move_non_generic(
            &mut board,
            Move::new(Square::E7, Square::E5, MoveType::Quiet),
        );
        let c3 = make_move_non_generic(
            &mut board,
            Move::new(Square::G1, Square::F3, MoveType::Quiet),
        );
        let c4 = make_move_non_generic(
            &mut board,
            Move::new(Square::B8, Square::C6, MoveType::Quiet),
        );
        let c5 = make_move_non_generic(
            &mut board,
            Move::new(Square::F1, Square::C4, MoveType::Quiet),
        );
        let c6 = make_move_non_generic(
            &mut board,
            Move::new(Square::G8, Square::F6, MoveType::Quiet),
        );

        // Test Castling: O-O
        test_move(
            &mut board,
            Move::new(Square::E1, Square::G1, MoveType::KingSideCastle),
            "Kingside Castle (O-O)",
        );

        // Test Castling Rights removal: Move the King instead of castling (rights should be lost and then restored)
        test_move(
            &mut board,
            Move::new(Square::E1, Square::E2, MoveType::Quiet),
            "King Move (Rights Loss Test)",
        );
    }
    #[cfg(test)]
    mod tests {
        use crate::{
            ai::evaluation::Evaluation,
            board::board::Color,
            engine::{make_move, undo_move},
            moves::{
                move_generator::{MoveGenerator, MoveList},
                traits::{Black, Castle, Side, White},
            },
        };

        use super::*;

        fn verify_zobrist_tree<S: Side + Castle + Evaluation>(
            board: &mut Board,
            hasher: &ZobristHasher,
            depth: u8,
        ) {
            // 1. Absolute Truth Check: Does incremental match reality?
            let scratch_hash = hasher.calculate_board_hash(
                &board.mailbox,
                board.castling_rights(),
                board.en_passant_square(),
                S::COLOR,
            );
            assert_eq!(
                board.zobrist, scratch_hash,
                "DESYNC DETECTED! Incremental: {}, Scratch: {},\n {}",
                board.zobrist, scratch_hash, board
            );

            if depth == 0 {
                return;
            }

            let mut move_list = MoveList::default();
            // Use whatever move generator you have to get all pseudo-legal/legal moves
            let move_gen = MoveGenerator::default();
            move_gen.generate_moves_generic::<S>(&mut move_list, &board);
            dbg!(move_list.as_slice());

            for mv in move_list.move_fetcher() {
                let hash_before = board.zobrist;

                // Assuming you have a way to know whose turn it is to call the generic make_move
                let undo = make_move::<S>(board, mv.mv);

                // Recurse into the next depth
                verify_zobrist_tree::<S::Opposite>(board, hasher, depth - 1);

                undo_move::<S>(mv.mv, board, undo);

                // 2. Make/Undo Symmetry Check
                assert_eq!(
                    board.zobrist, hash_before,
                    "UNDO CORRUPTION! Hash did not restore after undoing move: {:?}",
                    mv.mv
                );
            }
        }

        #[test]
        fn test_zobrist_rigorous() {
            MoveGenerator::init_slider_atk_tables();
            let hasher = ZobristHasher::get_hasher();

            // Test 1: Standard Start Position
            let mut board = Board::default(); // Assuming this sets up the standard starting board
            match board.side_to_move {
                Color::White => verify_zobrist_tree::<White>(&mut board, &hasher, 4), // Depth 3 or 4 is usually fast enough for a unit test
                Color::Black => verify_zobrist_tree::<Black>(&mut board, &hasher, 4), // Depth 3 or 4 is usually fast enough for a unit test
            }

            // Test 2: "Kiwipete" - The Ultimate Stress Test Position
            // This FEN has captures, castling, en passant, and promotions all available instantly.
            let mut kiwipete = Board::from_fen(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string(),
            );
            match board.side_to_move {
                Color::White => verify_zobrist_tree::<White>(&mut board, &hasher, 3), // Depth 3 or 4 is usually fast enough for a unit test
                Color::Black => verify_zobrist_tree::<Black>(&mut board, &hasher, 3), // Depth 3 or 4 is usually fast enough for a unit test
            }
        }
    }
}
