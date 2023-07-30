#![feature(async_closure)]
// TODO(ishan): Eventually we'll have a listener and transmitter module for every thing we want to
// support. So, 1 for MDNS, another for WSDD?
// Or a common listener/transmitter and then different modules to parse and transmit each type of
// traffic
use log::info;

pub mod config;
pub use config::*;
pub mod mdns;
pub use mdns::*;

fn main() {
    env_logger::init();

    info!("starting up");

    let config = Config::parse("config.toml").expect("error in parsing config");

    println!("{:?}", config);

    let mdns_client = Mdns::new(config);

    mdns_client.listener_loop();
}
