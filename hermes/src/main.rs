use std::fs::File;
use std::io::Write;
use dbus::blocking::Connection;
pub mod config;
pub mod comm;
pub mod reminder;
pub mod db;

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
    if file.is_err() {
	return; // can't log
    }
    let mut log = file.unwrap();

    let db_conn = config::initialize_environment(&log);
    if db_conn.is_none() { //Somewhere, initialization failed, and an issue should be logged
	return;
    }
    let db_conn = db_conn.unwrap();
    let _ = log.write_all(b"SUCCESSFUL DB CONN\n");
    let api_statements = db::PreparedStatements::new(&db_conn, &mut log);
    if api_statements.is_none() {
	return;
    }
    let mut api_statements = api_statements.unwrap();
	
    let _ = log.write_all(b"SUCCESSFUL API SETUP\n");
    //Now Daemon is in proper environment, with a database connection

    let socket = comm::init_zeromq(&log);
    if socket.is_none() {
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
	let _ = log.write_all(b"RECEIVED MESSAGE\n");
	if let Err(err) = data {
	    let fmt_str = format!("Error while receiving data: {}\n", err);
	    let _ = log.write_all(fmt_str.as_bytes());
	}
	let data = data.unwrap();
	comm::handle_message(&data, &conn, &mut log, &mut api_statements, &socket);
	 // Send message to put socket into valid state to receive next command
    }
}
