use std::net::TcpListener;
use std::sync::{Arc, RwLock};
use std::thread;

use tiny_http::{Header, Response, Server};

use crate::Frame;
use crate::imager::{AnonFrame, get_frame};

pub(crate) fn run_http(http_addr: &str, ws_port: usize) {
    let server = Server::http(http_addr).unwrap();

    for request in server.incoming_requests() {
        let mut str: String = String::from_utf8_lossy(include_bytes!("index.html")).into();
        str = str.replace("%WS_PORT%", &ws_port.to_string());
        let mut response = Response::from_string(str);
        response.add_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
        let _ = request.respond(response);
    }
}

pub(crate) fn run_websocket(frame: Arc<RwLock<Frame>>, ws_addr: &str) {
    let server = TcpListener::bind(ws_addr).unwrap();
    for stream in server.incoming() {
        let frame_arc = frame.clone();
        thread::spawn (move || {
            let mut websocket = tungstenite::accept(stream.unwrap()).unwrap();
            let mut frame = AnonFrame::default();
            'wsloop: loop {
                let new_frame = get_frame(&frame_arc);
                if new_frame.eq(&frame) {
                    continue
                }
                frame = new_frame;

                match websocket.write_message(tungstenite::Message::binary((&frame).pixels.to_vec())) {
                    Ok(_) => (),
                    Err(err) => {
                        eprintln!("Failed to send frame to ws with error {:?}", err);
                        break 'wsloop;
                    }
                }
            }
        });
    }
}