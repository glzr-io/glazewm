use std::net::TcpListener;
use std::thread::spawn;

use tungstenite::accept;

pub fn start_server() {
  let server = TcpListener::bind("127.0.0.1:6123").unwrap();

  for stream in server.incoming() {
    spawn(move || {
      let mut websocket = accept(stream.unwrap()).unwrap();
      loop {
        let msg = websocket.read().unwrap();

        // We do not want to send back ping/pong messages.
        if msg.is_binary() || msg.is_text() {
          websocket.send(msg).unwrap();
        }
      }
    });
  }
}
