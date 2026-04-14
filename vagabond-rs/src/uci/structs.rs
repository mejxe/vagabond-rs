use std::fmt::Display;

use crate::{
    board::{bitboard::Square, board::Board},
    moves::move_structs::{Move, MoveType},
};

#[derive(Debug, PartialEq)]
pub enum UciIn {
    Uci,
    IsReady,
    Position(Board),
    GoDepth(u8),
    GoTime(GoTimeParams),
    GoInfinite,
    NewGame,
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
    Info(InfoParams),
}
#[derive(Debug, PartialEq)]
pub struct InfoParams {
    pub curr_depth: u8,
    pub pv: Vec<Move>,
    pub nodes_searched: u32,
    pub evaluation: i16,
    pub time: u128,
}
#[derive(Debug, PartialEq)]
pub struct GoTimeParams {
    // all in miliseconds
    pub wtime: u128,
    pub btime: u128,
    pub winc: u128,
    pub binc: u128,
}
impl Display for InfoParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pv = self
            .pv
            .iter()
            .map(|mv| mv.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        let nps = self.nodes_searched as u128 / self.time.max(1) * 1000;
        let msg = format!(
            "depth {} nodes {} score cp {} time {} nps {} pv {} ",
            self.curr_depth, self.nodes_searched, self.evaluation, self.time, nps, pv
        );
        write!(f, "{}", msg)
    }
}

#[derive(Debug, PartialEq)]
pub struct EngineIdentity {
    name: &'static str,
    author: &'static str,
}
pub const IDENTITY: EngineIdentity = EngineIdentity {
    name: "Vagabond-rs",
    author: "mejxe",
};
impl Display for EngineIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = format!("id name {}\nid author {}", self.name, self.author);
        write!(f, "{msg}")
    }
}
