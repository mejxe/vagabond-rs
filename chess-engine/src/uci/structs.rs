use std::fmt::Display;

use crate::{board::board::Board, moves::move_structs::Move};

#[derive(Debug, PartialEq)]
pub enum UciIn {
    Uci,
    IsReady,
    Position(Board),
    GoDepth(u8),
    GoTime(u128),
    Stop,
    Board,
    Exit,
}
#[derive(Debug, PartialEq)]
pub enum UciOut {
    UciOk(EngineIdentity),
    ReadyOk,
    BestMove(Move),
    Board(Board),
    Info,
}

#[derive(Debug, PartialEq)]
pub struct EngineIdentity {
    name: &'static str,
    author: &'static str,
}
pub const IDENTITY: EngineIdentity = EngineIdentity {
    name: "placeholder",
    author: "mejxe",
};
impl Display for EngineIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = format!("id name {}\nid author {}", self.name, self.author);
        write!(f, "{msg}")
    }
}
