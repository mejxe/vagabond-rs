use std::{
    io::{BufRead, BufReader, BufWriter, Read, Stdin, Stdout, Write},
    sync::{atomic::Ordering, mpsc::Receiver},
};

use crate::uci::{parser::Parser, structs::IDENTITY};
use std::sync::mpsc::Sender;

use super::{
    handler::{Handler, StopFlag},
    structs::{EngineIdentity, UciIn, UciOut},
};

pub struct Communication;

impl Communication {
    pub fn communication_loop<R: BufRead>(
        std_in: R,
        in_tx: Sender<UciIn>,
        stop_flag: StopFlag,
    ) -> Result<(), ()> {
        for line in std_in.lines() {
            let line = line.unwrap();
            let instruction = Parser::parse(line);
            if let Some(UciIn::Exit) = instruction {
                stop_flag.store(true, Ordering::Relaxed);
                break;
            }
            if let Some(UciIn::Stop) = instruction {
                stop_flag.store(true, Ordering::Relaxed);
            }
            if let Some(instruction) = instruction {
                let _ = in_tx.send(instruction);
            }
        }
        Ok(())
    }
    pub fn broadcast(out_rx: Receiver<UciOut>) {
        while let Ok(out_msg) = out_rx.recv() {
            match out_msg {
                UciOut::UciOk(identity) => Self::print_uci(identity),
                UciOut::ReadyOk => println!("readyok"),
                UciOut::BestMove(mv) => println!("bestmove {mv}"),
                UciOut::Board(board) => println!("{board}"),
                UciOut::Info(inf) => inf.iter().for_each(|info_string| println!("{info_string}")),
                UciOut::Options(opts) => {
                    opts.iter().for_each(|opt| println!("{opt}"));
                    println!("uciok")
                }
                _ => {}
            }
        }
    }
    fn print_uci(identity: EngineIdentity) {
        println!("{}\n", identity)
    }
}
