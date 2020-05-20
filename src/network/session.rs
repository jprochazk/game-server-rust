
use crate::crossbeam;
use crate::ws;
use std::collections::HashMap;

use log::*;

use super::ws::SocketEvent;

pub struct SessionManager {
    sessions: HashMap<u32, Session>,
    channel: crossbeam::Receiver<SocketEvent>
}

impl SessionManager {
    pub fn new(channel: crossbeam::Receiver<SocketEvent>) -> SessionManager {
        SessionManager { 
            sessions: HashMap::new(),
            channel
        }
    }

    pub fn update(&mut self) {
        for msg in self.channel.try_iter() {
            match msg {
                SocketEvent::Open { id, addr, socket } => {
                    info!(target: "SESSION", "ID {} -> Opened remote {}", id, addr);
                    self.sessions.insert(id, Session{id,addr,socket});
                }
                SocketEvent::Close { id, code, reason } => {
                    if let Some(session) = self.sessions.remove(&id) {
                        info!(target: "SESSION", "ID {} -> Closed: {:?} {}", session.id, code, reason);
                    }
                }
                SocketEvent::Error { id, error } => {
                    error!(target: "SESSION", "ID {} -> Error: {}", id, error);
                }
                SocketEvent::Message { id, data } => {
                    info!(target: "SESSION", "ID {} -> Message: {:?}", id, data);
                    // TODO: handle packet
                }
            }
        }
    }
}

pub struct Session {
    pub id: u32,
    pub addr: String,
    pub socket: ws::Sender
}
