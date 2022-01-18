use std::{io, str::FromStr};
use std::fmt::Debug;
use zmq::{self, Message};
use crate::reminder;

/*
* This module contains methods used in communication with Hermes, the background daemon
*/

// D:01:12:2022:MessageText
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
    
    let _ = socket.send_multipart(vec!("HERMES".as_bytes().to_vec(),vec!(1 as u8), reminder.serialize()), 0);
    let mut msg = Message::new();
    let result = socket.recv(&mut msg, 0);
    if let Err(err) = result {
	println!("Error in receiving response from Hermes: {}", err);
    }
}

pub fn list_reminders(){
    let comm = construct_socket();
    if let None = comm {
	return;
    }
    let (ctx, socket) = comm.unwrap();
    let success = socket.connect("ipc:///tmp/hermesd");
    if let Err(err) = success {
	println!("Error connecting: {}", err);
	return;
    }

    let _ = socket.send_multipart(vec!("HERMES".as_bytes().to_vec(), vec!(2 as u8)),0);
    let data = socket.recv_multipart(0);
    if let Err(err) = data {
	println!("Error while receiving data: {}\n", err);
	return;
    }
    let mut data = data.unwrap();
    
    // Let server know data is sent
    let mut res = socket.send("RECEIVED",0);
    while let Err(_) = res {
	res = socket.send("RECEIVED",0);
    }

    if data.len() == 0 || std::str::from_utf8(&data[0]).unwrap() != "HERMES" {
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
    println!("Enter D, O, or W, for DAILY, ONCE, or WEEKLY, respectively");
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

    if let None = freq {
	return None;
    }
    let freq = freq.unwrap();

    let mut n : Option<u32> = None;
    if freq == reminder::Frequency::NDAYS {
	println!("Enter a number for how many days between notifications");
	buffer.clear();
	let _ = std_in.read_line(&mut buffer);
	let input = buffer.parse::<u32>();
	if let Err(_) = input {
	    println!("Error, invalid input received");
	    return None;
	} else {
	    n = Some(input.unwrap());
	}
    }

    println!("Enter a month (1 - 12)");
    let month = read_in_integer::<u8>();
    if let None = month {
	return None;
    }
    let month = month.unwrap();
    if month < 1 || month > 12 {
	println!("Input not in range");
	return None;
    }

    println!("Enter a day (numeric)");
    let day = read_in_integer::<u8>();
    if let None = day {
	return None;
    }
    let day = day.unwrap();

    println!("Enter a year");
    let year = read_in_integer::<u32>();
    if let None = year {
	return None;
    }
    let year = year.unwrap();

    println!("Enter an hour (0 - 24)");
    let hour = read_in_integer::<u8>();
    if let None = hour {
	return None;
    }
    let hour = hour.unwrap();
    if hour > 24 { // as a u8 it cant be under 0
	println!("Input not in range");
	return None;
    }

    println!("Enter a minute (0-60)");
    let minute = read_in_integer::<u8>();
    if let None = minute {
	return None;
    }
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
