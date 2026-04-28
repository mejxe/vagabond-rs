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
    SetOption(Vec<EngineOption>),
    Exit,
}
#[derive(Debug, PartialEq)]
pub enum EngineOption {
    MultiPV(usize),
}
#[derive(Debug, PartialEq)]
pub enum UciOut {
    UciOk(EngineIdentity),
    ReadyOk,
    BestMove(Move),
    Board(Board),
    Info(Vec<InfoParams>),
    Options(EngineCommandArray),
}
#[derive(Debug, PartialEq)]
pub struct InfoParams {
    pub curr_depth: u8,
    pub multi_pv: usize,
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
impl Display for EngineOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match self {
            EngineOption::MultiPV(val) => {
                format!("option name MultiPV type spin default 1 min 1 max {}", val)
            }
        };
        write!(f, "{}", result)
    }
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
            "depth {} multipv {} score cp {} time {} nodes {} nps {} pv {} ",
            self.curr_depth,
            self.multi_pv,
            self.evaluation,
            self.time,
            self.nodes_searched,
            nps,
            pv
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
pub const MAX_MULTI_PV: usize = 5;
pub type EngineCommandArray = [EngineOption; 1];
pub const AVAILABLE_OPTIONS: EngineCommandArray = [EngineOption::MultiPV(MAX_MULTI_PV)];
impl Display for EngineIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = format!("id name {}\nid author {}", self.name, self.author);
        write!(f, "{msg}")
    }
}
