extern crate crossbeam;
extern crate simple_logger;
extern crate ws;

mod network;

use std::thread;
use std::time;

use log::*;
use crossbeam::crossbeam_channel as cb;

use network::session::{SessionManager};
use network::ws::{Socket, SocketEvent};

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let address = "127.0.0.1:8001";
    let mut settings = ws::Settings::default();
    settings.max_connections = 1000;
    settings.queue_size = 10;
    settings.max_fragment_size = 1024;

    let (sq_send, sq_recv) = cb::unbounded::<SocketEvent>();

    let mut smgr = SessionManager::new(sq_recv);
    
    thread::Builder::new()
        .name("network".into())
        .spawn(move || {
            let socket = ws::Builder::new()
                .with_settings(settings)
                .build(|sender| {
                    Socket::new(sender, sq_send.clone())
                })
                .unwrap();

            info!("Listening on {}", address);
            socket.listen(address).unwrap();
        })
        .expect("Failed to spawn network thread");

    let tick_interval = 1000 / 30;
    let mut last_tick = time::Instant::now();
    loop {
        if last_tick.elapsed().as_millis() < tick_interval {
            continue;
        }
        last_tick = time::Instant::now();
        
        smgr.update();
    }
}
