use std::str::SplitAsciiWhitespace;

use crate::{
    board::board::{Board, PieceType, chess_notation_to_sq},
    engine::{make_move, make_move_non_generic},
    moves::{
        move_generator::{self, MoveGenerator, MoveList},
        move_structs::{Move, MoveType},
    },
};

use super::structs::UciIn;

#[derive(Debug)]
pub struct Parser;

impl Parser {
    pub fn parse(line: String) -> Option<UciIn> {
        let mut words = line.split_ascii_whitespace();
        let first_word = words.next();
        match first_word {
            Some("uci") => Some(UciIn::Uci),
            Some("go") => Self::parse_go(words.collect()), // TODO: may be a performance bottleneck
            Some("isready") => Some(UciIn::IsReady),
            Some("position") => Self::parse_position(words.collect()),
            Some("stop") => Some(UciIn::Stop),
            Some("g") => Some(UciIn::Board),
            Some("quit") => Some(UciIn::Exit),
            _ => None,
        }
    }
    fn parse_go(words: Vec<&str>) -> Option<UciIn> {
        let go_type = words.get(0).copied();
        match go_type {
            Some("depth") => {
                let depth = words[1].parse::<u8>();
                if let Ok(depth) = depth {
                    return Some(UciIn::GoDepth(depth));
                } else {
                    return None;
                }
            }
            _ => None,
        }
    }
    fn parse_position(words: Vec<&str>) -> Option<UciIn> {
        let position_type = words.get(0).copied();
        dbg!(&words);
        let mut moves_offset = 0;
        let mut entry_board = match position_type {
            Some("startpos") => Board::default(),

            Some("fen") => {
                let [pieces, side_to_move, castling, ep, hf, fm, rest_ @ ..] = &words[1..] else {
                    return None;
                };
                moves_offset = 6;
                dbg!(format!("{pieces} {side_to_move} {castling} {ep} {hf} {fm}"));
                Board::from_FEN(format!("{pieces} {side_to_move} {castling} {ep} {hf} {fm}"))
            }
            _ => return None,
        };
        if words.len() > moves_offset {
            let move_generator = MoveGenerator::default();
            words[(moves_offset + 1)..]
                .iter()
                .filter_map(|w| {
                    let mut chars = w.chars();
                    let row_from = chars.next()?;
                    let col_from = chars.next()?;
                    let row_to = chars.next()?;
                    let col_to = chars.next()?;
                    let promotion_opt = chars.next();
                    let promotion = if let Some(prom) = promotion_opt {
                        &prom.to_string()
                    } else {
                        ""
                    };
                    Some(format!(
                        "{}{}{}{}{}",
                        row_from, col_from, row_to, col_to, promotion
                    ))
                })
                .for_each(|passed_mv| {
                    let mut move_list = MoveList::default();
                    move_generator.generate_moves(&mut move_list, &entry_board);
                    let found_move = move_list
                        .as_slice()
                        .iter()
                        .find(|legal_mv| legal_mv.mv.to_string() == *passed_mv);
                    if let Some(mv) = found_move {
                        make_move_non_generic(&mut entry_board, mv.mv);
                        entry_board.swap_side();
                    }
                });
        }
        Some(UciIn::Position(entry_board))
    }
}
