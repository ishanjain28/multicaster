// TODO(ishan): Eventually we'll have a listener and transmitter module for every thing we want to
// support. So, 1 for MDNS, another for WSDD?
// Or a common listener/transmitter and then different modules to parse and transmit each type of
// traffic

pub mod communications;
pub use communications::*;
pub mod config;
pub use config::*;
pub mod listener;
pub mod processor;
pub use processor::*;

fn main() {
    env_logger::init();

    let config = Config::parse("config.toml").expect("error in parsing config");

    println!("{:?}", config);

    // TODO(ishan): Start listeners and transmitters on v4 and v6 here
    let mut comms = Communications::new(config).expect("error in starting comms");

    comms
        .start_listeners()
        .expect("error in starting listeners");

    let processor = Processor::new(comms.get_reader());

    processor.start_read_loop();

    comms.wait();
}
