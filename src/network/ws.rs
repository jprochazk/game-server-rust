
use crate::crossbeam;
use crate::ws;

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
    channel: crossbeam::Sender<SocketEvent>,
}

impl Socket {
    pub fn new(sender: ws::Sender, channel: crossbeam::Sender<SocketEvent>) -> Socket {
        Socket {
            out: sender,
            channel,
        }
    }
}

impl ws::Handler for Socket {
    fn on_open(&mut self, handshake: ws::Handshake) -> ws::Result<()> {
        let addr: String = handshake
            .remote_addr()
            .unwrap_or(Some(String::from("null")))
            .unwrap_or(String::from("null"));

        if &addr == "null" {
            return self.out.close(ws::CloseCode::Policy);
        }

        self.channel
            .send(SocketEvent::Open {
                id: self.out.connection_id(),
                addr,
                socket: self.out.clone(),
            })
            .unwrap();

        Ok(())
    }
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.channel
            .send(SocketEvent::Message {
                id: self.out.connection_id(),
                data: msg.into_data(),
            })
            .unwrap();
        Ok(())
    }
    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        if code == ws::CloseCode::Policy {
            return;
        }

        self.channel
            .send(SocketEvent::Close {
                id: self.out.connection_id(),
                code: code,
                reason: String::from(reason),
            })
            .unwrap();
    }
    fn on_error(&mut self, err: ws::Error) {
        self.channel
            .send(SocketEvent::Error {
                id: self.out.connection_id(),
                error: err,
            })
            .unwrap();
    }
}
