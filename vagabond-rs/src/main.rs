use std::{
    io::{self, BufReader},
    sync::{Arc, atomic::AtomicBool, mpsc::channel},
};

use vagabond_rs::{
    engine::Engine,
    uci::{
        communication::Communication,
        handler::Handler,
        structs::{UciIn, UciOut},
    },
};

fn main() -> Result<(), ()> {
    //engine init
    let mut engine = Engine::default();
    let stop_flag = Arc::new(AtomicBool::new(false));

    // comms init
    let (tx_in, rx_in) = channel::<UciIn>();
    let (tx_out, rx_out) = channel::<UciOut>();
    engine.set_tx(tx_out.clone());

    //handler init
    let mut handler = Handler::new(engine, rx_in, tx_out, stop_flag.clone());

    let std_in = BufReader::new(io::stdin());

    //engine and uci out threads
    std::thread::spawn(move || handler.handle());
    std::thread::spawn(move || Communication::broadcast(rx_out));

    // main loop
    Communication::communication_loop(std_in, tx_in, stop_flag.clone())
}
