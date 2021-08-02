use std::{net::{SocketAddr, ToSocketAddrs}, sync::{Arc, atomic::{AtomicBool, Ordering}}, thread::{self, JoinHandle, sleep}, time::{Duration, Instant}};

pub use laminar::{Config as NetworkConfig, Packet};

use crossbeam::channel::{Receiver, Sender, TryRecvError};
use laminar::SocketEvent;

use crate::Result;

pub enum NetworkEvent {
    Message(Packet),
    Connect(SocketAddr),
    Timeout(SocketAddr),
    Disconnect(SocketAddr),
}

pub struct Socket {
    sender: Sender<Packet>,
    stop_signal: Arc<AtomicBool>,
}

impl Socket {
    pub fn send(&self, packet: Packet) -> &Self {
        self.sender.send(packet).unwrap();
        self
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        self.stop_signal.swap(true, Ordering::Relaxed);
    }
}

#[derive(Default)]
pub struct Network {
    socket_thread: Option<JoinHandle<()>>,
    receiver: Option<Receiver<SocketEvent>>,
}

impl Network {
    pub(crate) fn new() -> Self {
        Network::default()
    }

    pub(crate) fn get_event(&mut self) -> Option<NetworkEvent> {
        if let Some(receiver) = &self.receiver {
            loop {
                match receiver.try_recv() {
                    Ok(message) => return Some(match message {
                        SocketEvent::Packet(packet) => NetworkEvent::Message(packet),
                        SocketEvent::Connect(address) => NetworkEvent::Connect(address),
                        SocketEvent::Timeout(address) => NetworkEvent::Timeout(address),
                        SocketEvent::Disconnect(address) => NetworkEvent::Disconnect(address),
                    }),
                    Err(e) => match e {
                        TryRecvError::Empty => break,
                        TryRecvError::Disconnected => {
                            self.socket_thread.take().unwrap().join().unwrap();
                            self.receiver.take();
                            break;
                        },
                    },
                }
            }
        }

        None
    }

    pub fn bind<A: ToSocketAddrs>(&mut self, addresses: A) -> Result<Socket> {
        self.bind_with_config(addresses, NetworkConfig::default())
    }

    pub fn bind_with_config<A: ToSocketAddrs>(&mut self, addresses: A, config: NetworkConfig) -> Result<Socket> {
        let mut socket = laminar::Socket::bind_with_config(addresses, config)?;
        let sender = socket.get_packet_sender();
        let receiver = socket.get_event_receiver();
        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop = stop_signal.clone();

        let socket_thread = thread::spawn(move ||
            while !stop.load(Ordering::Relaxed) {
                socket.manual_poll(Instant::now());
                sleep(Duration::from_millis(1));
            }
        );

        self.socket_thread = Some(socket_thread);
        self.receiver = Some(receiver);

        Ok(Socket {
            sender,
            stop_signal,
        })
    }
}
