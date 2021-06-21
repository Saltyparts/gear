use std::{fmt::Display, net::ToSocketAddrs, rc::{Rc, Weak}};

use laminar::Socket;
use log::info;

pub enum NetworkEvent {
    Connected(Connection),
    MessageReceived,
}

pub struct Connection(Rc<Socket>);

pub struct Network {
    connections: Vec<Weak<Socket>>,
}

impl Network {
    pub(crate) fn new() -> Self {
        info!("Initializing network backend");

        Self {
            connections: vec![],
        }
    }

    pub fn bind<A: ToSocketAddrs + Display>(&self, address: A) {
        info!("Binding to address: {}", address);
    }
}
