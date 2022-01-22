use zmq::{Context, Socket};
use std::fs::File;
use std::io::Write;

pub fn set_socket(mut log: &File) -> Option<Socket> {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::REP);
    if let Err(err) = socket {
	return None;
    }

    let socket = socket.unwrap();
    let success = socket.bind("ipc:///tmp/hermesd");
    if let Err(err) = success {
	let fmt_str = format!("Error binding socket: {}", err);
        let _ = log.write_all(fmt_str.as_bytes());
        return None;
    } else {
        return Some(socket);
    }
}
