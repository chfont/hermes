use std::io;
use zmq::{self, Message};
use crate::message;

/*
* This module contains methods used in communication with Hermes, the background daemon
*/

// D:01:12:2022:Yo listen up heres a story about a little guy who lives in a blue world
pub fn add_reminder(){
    println!("Enter D, O, or W, for DAILY, ONCE, or WEEKLY, respectively");
    let std_in = io::stdin();
    let mut buffer = String::new();
    let _ = std_in.read_line(&mut buffer);

    let mut freq: Option<message::Frequency> = None;
    match buffer.trim() {
	"D" => { freq = Some(message::Frequency::DAILY);},
	"O" => { freq = Some(message::Frequency::ONCE);},
	"W" => { freq = Some(message::Frequency::WEEKLY);},
	_ => {
	    println!("Invalid input received");
	    return;
	}
    }

    let freq = freq.unwrap();
    println!("Enter a message body for the reminder:");
    buffer.clear();
    let _ = std_in.read_line(&mut buffer);
    println!("Message");
    let msg = message::Message::new(freq, buffer);
    
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::REQ);
    if let Err(err) = socket {
	println!("Error initializing socket: {}", err);
	return;
    }
    let socket = socket.unwrap();
    
    let success = socket.connect("ipc:///tmp/hermesd");
    if let Err(err) = success {
	println!("Error connecting: {}", err);
	return;
    }
    
    let _ = socket.send_multipart(vec!("HERMES".as_bytes().to_vec(), msg.serialize()), 0);
    let mut msg = Message::new();
    let result = socket.recv(&mut msg, 0);
    if let Err(err) = result {
	println!("Error in receiving response from Hermes: {}", err);
    }
}
