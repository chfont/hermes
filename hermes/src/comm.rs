use dbus::arg::messageitem::MessageItem;
use dbus::blocking::Connection;
use dbus::channel::Sender;
use dbus::message as msg;
use std::{fs::File, io::Write};
use zmq::{self, Socket};

pub fn init_zeromq(mut log: &File) -> Option<Socket> {
    let context = zmq::Context::new();
    let response = context.socket(zmq::REP).unwrap();
    let success = response.bind("ipc:///tmp/hermesd");
    if let Err(err) = success {
        let fmt_str = format!("Error binding socket: {}", err);
        let _ = log.write_all(fmt_str.as_bytes());
        return None;
    } else {
        return Some(response);
    }
}

// See https://specifications.freedesktop.org/notification-spec/notification-spec-latest.html for spec of commands

pub fn notify(data: &Vec<Vec<u8>>, conn: &Connection, mut log: &File) {
    if data.len() != 2 {
	let _ = log.write_all(b"Received Message of invalid part count\n");
	return;
    }
    let header = std::str::from_utf8(&data[0]);
    if let Err(e) = header {
	let fmt_str = format!("Error decoding message header: {}\n", e.to_string());
	let _ = log.write_all(fmt_str.as_bytes());
	return;
    }
    let header = header.unwrap();
    if header != "HERMES" {
	let fmt_str = format!("Invalid Header, received {}, expected: HERMES\n", header);
	let _ = log.write_all(fmt_str.as_bytes());
	return;
    }
    
    let message = std::str::from_utf8(&data[1]);
    if let Err(e) = message {
	let fmt_str = format!("Error decoding message body: {}\n", e.to_string());
	let _ = log.write_all(fmt_str.as_bytes());
	return;
    }
    let message = message.unwrap();

    let res = msg::Message::new_method_call(
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
        "Notify",
    );
    if let Err(e) = res {
        let fmt_str = format!("Error setting up dbus message: {}\n", e.to_string());
	let _ = log.write_all(fmt_str.as_bytes());
	return;
    }

    let mut dbus_msg = res.unwrap();
    dbus_msg.append_items(&[
        MessageItem::Str("Hermes".to_string()),
        MessageItem::UInt32(0),
        MessageItem::Str("".to_string()),
        MessageItem::Str("Hermes".to_string()),
        MessageItem::Str(message.to_string()),
        MessageItem::new_array(vec![MessageItem::Str("".to_string())]).unwrap(),
        MessageItem::new_dict(vec![(
            MessageItem::Str("".to_string()),
            MessageItem::Variant(Box::new(MessageItem::Str("".to_string()))),
        )])
        .unwrap(),
        MessageItem::Int32(3000),
    ]);

    let _ = conn.send(dbus_msg);
}
