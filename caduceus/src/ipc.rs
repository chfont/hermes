use std::{io, str::FromStr};
use std::fmt::Debug;
use zmq::{self, Message};
use crate::reminder;

/*
* This module contains methods used in communication with Hermes, the background daemon
*/


pub fn add_reminder(){

    let reminder = build_message_interactive();
    if let None = reminder {
	return;
    }
    let reminder = reminder.unwrap();
    
    let zmq_conn = construct_socket();
    if let None = zmq_conn {
	return;
    }
    let (ctx, socket) = zmq_conn.unwrap();
    
    let success = socket.connect("ipc:///tmp/hermesd");
    if let Err(err) = success {
	println!("Error connecting: {}", err);
	return;
    }
    
    let _ = socket.send_multipart(vec!("HERMES".as_bytes().to_vec(),vec!(1_u8), reminder.serialize()), 0);
    let mut msg = Message::new();
    let result = socket.recv(&mut msg, 0);
    if let Err(err) = result {
	println!("Error in receiving response from Hermes: {}", err);
    }
}

pub fn list_reminders(){
    let comm = construct_socket();
    if comm.is_none() {
	return;
    }
    let (ctx, socket) = comm.unwrap();
    let success = socket.connect("ipc:///tmp/hermesd");
    if let Err(err) = success {
	println!("Error connecting: {}", err);
	return;
    }

    let _ = socket.send_multipart(vec!("HERMES".as_bytes().to_vec(), vec!(2_u8)),0);
    let data = socket.recv_multipart(0);
    if let Err(err) = data {
	println!("Error while receiving data: {}\n", err);
	return;
    }
    let mut data = data.unwrap();
    
    // Let server know data is sent
    let mut res = socket.send("RECEIVED",0);
    while res.is_err() {
	res = socket.send("RECEIVED",0);
    }

    if data.is_empty() || std::str::from_utf8(&data[0]).unwrap() != "HERMES" {
	println!("Malformed message received");
	return;
    }
    data.remove(0); // TODO: Better way?
    for reminder in data {
	let rem = reminder::Reminder::deserialize_reminder(reminder);
	if let Some(value) = rem {
	    value.print();
	}
    }
}

pub fn delete_reminder() {
    let comm = construct_socket();
    if comm.is_none() {
	return;
    }
    let (_, socket) = comm.unwrap();
    let success = socket.connect("ipc:///tmp/hermesd");
    if let Err(err) = success {
	println!("Error connecting: {}", err);
	return;
    }

    let _ = socket.send_multipart(vec!("HERMES".as_bytes().to_vec(), vec!(3_u8)), 0);
    let data = socket.recv_multipart(0);
    if let Err(err) = data {
	println!("Error while receiving data: {}\n", err);
	return;
    }
    let mut data = data.unwrap();
    if data.is_empty() || std::str::from_utf8(&data[0]).unwrap() != "HERMES" {
	println!("Malformed message received");
	return;
    }
    data.remove(0); // TODO: Better way?
    let reminders: Vec<(u32, reminder::Reminder)> = Vec::new(); 
    for msg in data {
	// Should be (u32, Reminder)
	if msg.len() < 18 {
	    println!("Malformed Message received");
	} else {
	    let id = u32::from_be_bytes([msg[0],msg[1], msg[2], msg[3]]);
	    let (_, rem) = msg.split_at(4);
	    let reminder = reminder::Reminder::deserialize_reminder(rem.to_vec());
	    if reminder.is_none(){
		println!("Malformed Message received");
	    } else {
		print!("ID: {} |", id);
		let val = reminder.unwrap();
		val.print();
	    }
	}

	
    }

    println!("Enter the id of a reminder to delete:");
	//TODO: get ID, send, get response from server
	let std_in = io::stdin();
	let mut buffer = String::new();
	let _ = std_in.read_line(&mut buffer);
	let id = buffer.trim().parse::<u32>();
	if id.is_err() {
	    println!("Invalid id entered");
	} else {
	    let _ =  socket.send(buffer.trim(),0);
	    let mut buff = zmq::Message::new();
	    let _ = socket.recv(&mut buff,0);
	    println!("{}",buff.as_str().unwrap());
	}
    
}

fn construct_socket() -> Option<(zmq::Context, zmq::Socket)> {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::REQ);
    if let Err(err) = socket {
	println!("Error initializing socket: {}", err);
	return None;
    }
    let socket = socket.unwrap();
    return Some((ctx, socket));
}

fn read_in_integer<T: Debug + FromStr>() -> Option<T> where <T as FromStr>::Err : Debug{
    let stdin = io::stdin();
    let mut buffer = String::new();
    let _ = stdin.read_line(&mut buffer);
    
    let input = buffer.trim().parse::<T>();
    match input {
	Err(err) => {
	    println!("Error, invalid input received: {:?}", err);
	    return None;
	},
	Ok(value) => {
	    return Some(value);
	}
    }
}

fn build_message_interactive() -> Option<reminder::Reminder> {
    println!("Enter D, O, W, or N, for DAILY, ONCE, WEEKLY, or every N DAYS,  respectively");
    let std_in = io::stdin();
    let mut buffer = String::new();
    let _ = std_in.read_line(&mut buffer);

    let mut freq: Option<reminder::Frequency> = None;
    match buffer.trim() {
	"D" => { freq = Some(reminder::Frequency::DAILY);},
	"O" => { freq = Some(reminder::Frequency::ONCE);},
	"W" => { freq = Some(reminder::Frequency::WEEKLY);},
	"N" => { freq = Some(reminder::Frequency::NDAYS);},
	_ => {
	    println!("Invalid input received");
	}
    }

    let freq = freq?;

    let mut n : Option<u32> = None;
    if freq == reminder::Frequency::NDAYS {
	println!("Enter a number for how many days between notifications");
	buffer.clear();
	let _ = std_in.read_line(&mut buffer);
	let input = buffer.trim().parse::<u32>();
	if let Err(_) = input {
	    println!("Error, invalid input received");
	    return None;
	} else {
	    n = Some(input.unwrap());
	}
    }

    println!("Enter a month (1 - 12)");
    let month = read_in_integer::<u8>();
    month?;
    let month = month.unwrap();
    if !(1..=12).contains(&month) {
	println!("Input not in range");
	return None;
    }

    println!("Enter a day (numeric)");
    let day = read_in_integer::<u8>();
    day?;
    
    let day = day.unwrap();

    println!("Enter a year");
    let year = read_in_integer::<u32>();
    year?;
    let year = year.unwrap();

    println!("Enter an hour (0 - 24)");
    let hour = read_in_integer::<u8>();
    hour?;
    
    let hour = hour.unwrap();
    if hour > 24 { // as a u8 it cant be under 0
	println!("Input not in range");
	return None;
    }

    println!("Enter a minute (0-60)");
    let minute = read_in_integer::<u8>();
    minute?;
    let minute = minute.unwrap();
    if minute > 60 {
	println!("Input not in range");
	return None;
    }
    
    println!("Enter a message body for the reminder:");
    buffer.clear();
    let _ = std_in.read_line(&mut buffer);

    // Can construct Message object
    let message: reminder::Reminder = reminder::Reminder::new(
	freq,
	buffer,
	month,
	day,
	year,
	hour,
	minute,
	n
    );

    return Some(message);
    
}
