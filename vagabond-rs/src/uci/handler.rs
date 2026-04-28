use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, Sender},
};

use crate::{ai::negamax::MAX_SEARCH_DEPTH, board, engine::Engine, uci::structs::IDENTITY};

use super::structs::{AVAILABLE_OPTIONS, EngineOption, UciIn, UciOut};

pub type StopFlag = Arc<AtomicBool>;
pub struct Handler {
    engine: Engine,
    receiver: Receiver<UciIn>,
    transmiter: Sender<UciOut>,
    stop: StopFlag,
}
impl Handler {
    pub fn new(
        engine: Engine,
        receiver: Receiver<UciIn>,
        transmiter: Sender<UciOut>,
        stop_flag: StopFlag,
    ) -> Handler {
        Handler {
            engine,
            receiver,
            transmiter,
            stop: stop_flag,
        }
    }
    pub fn handle(&mut self) {
        while let Ok(instruction) = self.receiver.recv() {
            match instruction {
                UciIn::Uci => {
                    self.transmiter.send(UciOut::UciOk(IDENTITY)).unwrap();
                    self.transmiter
                        .send(UciOut::Options(AVAILABLE_OPTIONS))
                        .unwrap();
                }
                UciIn::GoDepth(depth) => {
                    self.stop.store(false, Ordering::Relaxed);
                    let stop_clone = self.stop.clone();
                    let mut engine_clone = self.engine.clone();
                    let tx_clone = self.transmiter.clone();
                    std::thread::spawn(move || {
                        if let Some(mv) = engine_clone.go_depth(depth, stop_clone) {
                            tx_clone.send(UciOut::BestMove(mv)).unwrap();
                        } else {
                            panic!("No move returned")
                        }
                    });
                }
                UciIn::GoTime(time_params) => {
                    self.stop.store(false, Ordering::Relaxed);
                    let stop_clone = self.stop.clone();
                    let mut engine_clone = self.engine.clone();
                    let tx_clone = self.transmiter.clone();
                    std::thread::spawn(move || {
                        if let Some(mv) = engine_clone.go_time(time_params, stop_clone) {
                            tx_clone.send(UciOut::BestMove(mv)).unwrap();
                        } else {
                            panic!("No move returned")
                        }
                    });
                }
                UciIn::GoInfinite => {
                    self.stop.store(false, Ordering::Relaxed);
                    let stop_clone = self.stop.clone();
                    let mut engine_clone = self.engine.clone();
                    let tx_clone = self.transmiter.clone();
                    std::thread::spawn(move || {
                        if let Some(mv) = engine_clone.go_depth(MAX_SEARCH_DEPTH as u8, stop_clone)
                        {
                            tx_clone.send(UciOut::BestMove(mv)).unwrap();
                        } else {
                            panic!("No move returned")
                        }
                    });
                }
                UciIn::IsReady => self.transmiter.send(UciOut::ReadyOk).unwrap(),
                UciIn::Position(pos) => self.engine.set_board(pos),
                UciIn::Board => self
                    .transmiter
                    .send(UciOut::Board(self.engine.board().clone()))
                    .unwrap(),
                UciIn::NewGame => {
                    self.stop.store(true, Ordering::Relaxed);
                    let board_ref = self.engine.board_mut();
                    board_ref.history.clear();
                    board_ref.half_move_clock = 0;
                    self.engine.tt_mut().lock().unwrap().clear_tt();
                }
                UciIn::SetOption(options) => {
                    for option in options {
                        self.engine.set_option(option);
                    }
                }
                _ => {}
            }
        }
    }
}
