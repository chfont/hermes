use dbus::arg::messageitem::MessageItem;
use dbus::blocking::Connection;
use dbus::channel::Sender;
use dbus::message as msg;
use std::{fs::File, io::Write};
use zmq::{self, Socket};
use crate::{reminder, db};

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

// Invariant: Socket needs to be in state to recv after this
pub fn handle_message(data: &Vec<Vec<u8>>, conn: &Connection, log: &mut File, api_statements: &mut db::PreparedStatements, socket: &zmq::Socket) {
    if data.len() < 2 {
	let _ = log.write_all(b"Received Message of invalid part count\n");
	return;
    }

    let valid_header = validate_header(&data[0],log);
    if ! valid_header {
	return; // Already logged
    }
    
    let command = &data[1];
    if command.len() != 1 {
	let _ = log.write_all(b"Invalid Command code received\n");
    }
    match command[0] {
	1 => { // add
	    let _ =  socket.send("RECEIVED",0);
	    if data.len() != 3 {
		let _ = log.write_all(b"Received Message of invalid part count\n");
		return;
	    }
	    notify(&data[2], conn, log);
	    let reminder = reminder::Reminder::deserialize_reminder(&data[2], log);
	    if reminder.is_none() {
		let _ = log.write_all(b"Could not deserialize reminder\n");
	    }
	    let reminder = reminder.unwrap();

	    add_reminder(reminder, api_statements, log);
	    let _ = socket.send("RECEIVED",0);
	},
	2 => { // list
	    let _ = log.write_all(b"RECEIVED LIST COMMAND\n");
	    list_reminders(api_statements, socket, log);
	    let _ = socket.send("SUCCESS", 0);
	},
	_ => {} // TODO: ADD DELETE
    };
}

// See https://specifications.freedesktop.org/notification-spec/notification-spec-latest.html for spec of commands

pub fn notify(data: &Vec<u8>, conn: &Connection, mut log: &File) {

    let reminder = reminder::Reminder::deserialize_reminder(data, log);
    if reminder.is_none() {
	return;
    }
    
    let reminder = reminder.unwrap();

    let res = msg::Message::new_method_call(
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
        "Notify",
    );
    if let Err(e) = res {
        let fmt_str = format!("Error setting up dbus message: {}\n", e);
	let _ = log.write_all(fmt_str.as_bytes());
	return;
    }

    let mut dbus_msg = res.unwrap();
    dbus_msg.append_items(&[
        MessageItem::Str("Hermes".to_string()),
        MessageItem::UInt32(0),
        MessageItem::Str("".to_string()),
        MessageItem::Str("Hermes".to_string()),
        MessageItem::Str(reminder.get_message().to_string()),
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

fn validate_header(vec: &Vec<u8>, mut log: &File) -> bool {
    
    let header = std::str::from_utf8(vec);
    if let Err(e) = header {
	let fmt_str = format!("Error decoding message header: {}\n", e);
	let _ = log.write_all(fmt_str.as_bytes());
	return false;
    }
    
    let header = header.unwrap();
    if header != "HERMES" {
	let fmt_str = format!("Invalid Header, received {}, expected: HERMES\n", header);
	let _ = log.write_all(fmt_str.as_bytes());
	return false;
    }

    return true;
}

fn list_reminders(api_statements: &mut db::PreparedStatements, socket: &zmq::Socket, log: &mut File) {
    
    let reminders = api_statements.list(log);
    if reminders.is_none() {
	return;
    }
    let reminders = reminders.unwrap();
    
    let mut msg_vec: Vec<Vec<u8>> = Vec::new();
    // TODO: fill msg_vec
    msg_vec.push("HERMES".as_bytes().to_vec());
    for reminder in reminders {
	msg_vec.push(reminder.serialize());
    }
    
    let _ = socket.send_multipart(msg_vec, 0);
    let mut buff = zmq::Message::new();
    let _ = socket.recv(&mut buff, 0); //  To get response from server
}

fn add_reminder(reminder: reminder::Reminder, api_statements: &mut db::PreparedStatements, log: &File) {
    api_statements.add(reminder, log);
}
