use dns_parser::Packet;
use log::info;
use std::sync::mpsc::Receiver;

use crate::Event;

pub struct Processor<'a> {
    reader: &'a Receiver<Event>,
}

impl<'a> Processor<'a> {
    pub fn new(reader: &'a Receiver<Event>) -> Self {
        Self { reader }
    }

    pub fn start_read_loop(&self) {
        for evt in self.reader {
            let packet =
                Packet::parse(&evt.payload).expect("failed to parse packet as a dns packet");

            info!("{:?}", packet);
        }
    }
}
