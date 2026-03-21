use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, Sender},
};

use crate::{engine::Engine, uci::structs::IDENTITY};

use super::structs::{UciIn, UciOut};

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
                UciIn::Uci => self.transmiter.send(UciOut::Info).unwrap(),
                UciIn::GoDepth(depth) => {
                    self.stop.store(false, Ordering::Relaxed);
                    let stop_clone = self.stop.clone();
                    let mut engine_clone = self.engine.clone();
                    let tx_clone = self.transmiter.clone();
                    std::thread::spawn(move || {
                        if let Some(mv) = engine_clone.go(depth, stop_clone) {
                            tx_clone.send(UciOut::BestMove(mv));
                        }
                    });
                }
                UciIn::IsReady => self.transmiter.send(UciOut::ReadyOk).unwrap(),
                UciIn::Position(pos) => self.engine.set_board(pos),
                UciIn::Board => self
                    .transmiter
                    .send(UciOut::Board(self.engine.board()))
                    .unwrap(),
                _ => {}
            }
        }
    }
}
