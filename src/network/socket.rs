use super::session::Session;
use crate::crossbeam;
use crate::ws;
use crossbeam::crossbeam_channel as cb;

pub enum SocketEvent {
    Open {
        id: u32,
        addr: String,
        socket: ws::Sender,
    },
    Close {
        id: u32,
        code: ws::CloseCode,
        reason: String,
    },
    Message {
        id: u32,
        data: Vec<u8>,
    },
    Error {
        id: u32,
        error: ws::Error,
    },
}

pub struct Socket {
    out: ws::Sender,
    socket_queue: Option<crossbeam::Sender<SocketEvent>>,
    open: bool,
    _smgr_queue: crossbeam::Sender<Session>,
}

impl Socket {
    pub fn new(sender: ws::Sender, channel: crossbeam::Sender<Session>) -> Socket {
        Socket {
            out: sender,
            socket_queue: None,
            open: false,
            _smgr_queue: channel,
        }
    }
}

/// This ws::Handler implementation does a few basic checks, and otherwise
/// relays any events to the main thread
impl ws::Handler for Socket {
    fn on_open(&mut self, handshake: ws::Handshake) -> ws::Result<()> {
        let addr: String = handshake
            .remote_addr()
            .unwrap_or_else(|_| Some(String::from("null")))
            .unwrap_or_else(|| String::from("null"));

        // if we can't get a remote address, close the connection
        if &addr == "null" {
            return self.out.close(ws::CloseCode::Policy);
        }

        let channel = Box::new(cb::unbounded::<SocketEvent>());
        self.socket_queue = Some((*channel).0.clone());
        self.open = true;

        self._smgr_queue
            .send(Session {
                id: self.out.connection_id(),
                addr,
                queue: recv,
                socket: self.out.clone(),
            })
            .unwrap_or_else(|e| panic!("{}", e));

        Ok(())
    }
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        if !self.open {
            return Ok(());
        }

        self.socket_queue
            .unwrap()
            .send(SocketEvent::Message {
                id: self.out.connection_id(),
                data: msg.into_data(),
            })
            .unwrap_or_else(|e| panic!("{}", e));
        Ok(())
    }
    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        if !self.open {
            return;
        }

        if code == ws::CloseCode::Policy {
            return;
        }

        self.socket_queue
            .unwrap()
            .send(SocketEvent::Close {
                id: self.out.connection_id(),
                code,
                reason: String::from(reason),
            })
            .unwrap_or_else(|e| panic!("{}", e));
    }
    fn on_error(&mut self, err: ws::Error) {
        if !self.open {
            return;
        }
        self.socket_queue
            .unwrap()
            .send(SocketEvent::Error {
                id: self.out.connection_id(),
                error: err,
            })
            .unwrap_or_else(|e| panic!("{}", e));
    }
}
