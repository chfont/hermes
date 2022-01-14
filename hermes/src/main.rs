use libc;
use std::fs::File;
use std::io::Write;
use dbus::blocking::Connection;
pub mod config;
pub mod comm;

fn main() {
   
    let proc_id: libc::pid_t = unsafe{libc::fork()};
    if proc_id < 0 {
	println!("Failed to fork");
	return;
    }
    if proc_id > 0 {
	println!("Successful fork, child is: {}", proc_id);
	return;
    }

    // Now we are in child process
    unsafe {
	let session_id = libc::setsid();
	if session_id < 0 {
	    return;
	}
	libc::close(libc::STDOUT_FILENO);
	libc::close(libc::STDIN_FILENO);
	libc::close(libc::STDERR_FILENO);
    }
    // Now Process is running as a proper Unix Daemon

    let file = File::create("/tmp/hermes.log");
    if let Err(_) = file {
	return; // can't log
    }
    let mut log = file.unwrap();

    let db_conn = config::initialize_environment(&mut log);
    if let None = db_conn { //Somewhere, initialization failed, and an issue should be logged
	return;
    }

    //Now Daemon is in proper environment, with a database connection

    let socket = comm::init_zeromq(&log);
    if let None = socket {
	return; // Socket binding failed, terminate (already logged)
    }
    let socket = socket.unwrap();

    let conn = Connection::new_session();
    if let Err(e) = &conn {
	let fmt_str = format!("Error creating dbus session: {}\n", e);
	let _ = log.write_all(fmt_str.as_bytes());
    }
    let conn = conn.unwrap();

    loop{
	let data = socket.recv_multipart(0);
	if let Err(err) = data {
	    let fmt_str = format!("Error while receiving data: {}\n", err);
	    let _ = log.write_all(fmt_str.as_bytes());
	}
	let data = data.unwrap();
	comm::notify(&data, &conn, &log);
	let _ =  socket.send("RECEIVED",0); // Send message to put socket into valid state to receive next command
    }
}
