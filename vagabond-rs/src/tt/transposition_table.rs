use crate::{
    board::bitboard::Square,
    moves::move_structs::{Move, MoveType},
};

use super::zobrist::ZobristHash;

pub const TT_DEFAULT_SIZE_MB: usize = 256;
pub struct TT {
    entries: Vec<TTEntry>,
    size: usize,
}
impl TT {
    fn new(size_mb: usize) -> TT {
        let size = TT::calculate_size(size_mb);
        let entries = vec![TTEntry::default(); size];
        TT { entries, size }
    }
    pub fn put(&mut self, entry: TTEntry) {
        let index = entry.key as usize & (self.size - 1);
        //let old_entry = self.entries[index];
        //if old_entry.depth() < entry.depth {
        self.entries[index] = entry;
        //}
    }
    pub fn get(&self, key: ZobristHash) -> Option<&TTEntry> {
        let index = key as usize & (self.size - 1);
        self.entries.get(index).filter(|entry| entry.key == key)
    }
    pub fn resize(&mut self, size_mb: usize) {
        self.size = TT::calculate_size(size_mb);
        self.entries.resize(self.size, TTEntry::default());
    }
    pub fn clear_tt(&mut self) {
        self.entries.fill(TTEntry::default());
    }
    fn calculate_size(size_mb: usize) -> usize {
        size_mb * 1024 * 1024 / std::mem::size_of::<TTEntry>()
    }
}
impl Default for TT {
    fn default() -> Self {
        TT::new(TT_DEFAULT_SIZE_MB)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    key: ZobristHash,
    best_move: Move,
    score: i16,
    depth: u8,
    node_type: NodeType,
}
impl Default for TTEntry {
    fn default() -> Self {
        TTEntry {
            best_move: Move::new(Square::A1, Square::A1, MoveType::Quiet),
            node_type: NodeType::Exact,
            key: 0,
            depth: 0,
            score: 0,
        }
    }
}

impl TTEntry {
    pub fn new(
        best_move: Move,
        node_type: NodeType,
        key: ZobristHash,
        depth: u8,
        score: i16,
    ) -> Self {
        Self {
            best_move,
            node_type,
            key,
            depth,
            score,
        }
    }

    #[inline(always)]
    pub fn best_move(&self) -> Move {
        self.best_move
    }

    #[inline(always)]
    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }

    #[inline(always)]
    pub fn key(&self) -> u64 {
        self.key
    }

    #[inline(always)]
    pub fn depth(&self) -> u8 {
        self.depth
    }

    #[inline(always)]
    pub fn score(&self) -> i16 {
        self.score
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NodeType {
    Exact,
    Lowerbound,
    Upperbound,
}
