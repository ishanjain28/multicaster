#![feature(async_closure)]
// TODO(ishan): Eventually we'll have a listener and transmitter module for every thing we want to
// support. So, 1 for MDNS, another for WSDD?
// Or a common listener/transmitter and then different modules to parse and transmit each type of
// traffic
use log::info;
use multicast_socket::Message;
use std::sync::mpsc::{self, Receiver, Sender};
use tokio::runtime::Runtime;

pub mod communications;
pub use communications::*;
pub mod config;
pub use config::*;
pub mod processor;
pub use processor::*;

fn main() {
    env_logger::init();

    info!("starting up");

    let config = Config::parse("config.toml").expect("error in parsing config");

    println!("{:?}", config);

    let rt = Runtime::new().expect("error in creating runtime");

    // This channel is used to send packets from this module to the processor
    // TODO(ishan): Eventually, swap out std::mpsc for some thing faster
    // or switch to a different model completely that doesn't use channels like this
    let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

    // TODO(ishan): Start listeners and transmitters on v4 and v6 here
    let mut comms = Communications::new(tx).expect("error in starting comms");

    rt.spawn(async move {
        comms
            .start_listeners()
            .await
            .expect("error in comms listener");
    });

    let processor = Processor::new(rx, config).expect("error in starting processor");

    processor.start_read_loop();
}
