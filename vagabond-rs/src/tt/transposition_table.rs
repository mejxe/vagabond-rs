use crate::{
    board::bitboard::Square,
    moves::move_structs::{Move, MoveType},
};

use super::zobrist::ZobristHash;

pub struct TT {
    entries: Vec<TTEntry>,
    size: usize,
}
impl TT {
    pub fn new(size_mb: usize) -> TT {
        let size = size_mb * 1024 * 1024 / std::mem::size_of::<TTEntry>();
        let entries = vec![TTEntry::default(); size];
        TT { entries, size }
    }
    pub fn put(&mut self, entry: TTEntry) {
        let index = entry.key as usize % self.size;
        let old_entry = self.entries[index];
        if old_entry.depth() <= entry.depth {
            self.entries[index] = entry;
        }
        if old_entry.key == entry.key {
            // dbg!("kolizja: {}", entry);
        }
    }
    pub fn get(&self, key: ZobristHash) -> Option<&TTEntry> {
        let index = key as usize % self.size;
        self.entries.get(index).filter(|entry| entry.key == key)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    best_move: Move,
    node_type: NodeType,
    key: ZobristHash,
    depth: u8,
    score: i16,
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
