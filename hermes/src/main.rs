use chrono;
use dbus::blocking::Connection;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub mod comm;
pub mod config;
pub mod db;
pub mod reminder;
pub mod socket;

fn main() {
    let proc_id: libc::pid_t = unsafe { libc::fork() };
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
    if db_conn.is_none() {
        //Somewhere, initialization failed, and an issue should be logged
        return;
    }
    let db_conn = db_conn.unwrap();
    let _ = log.write_all(b"SUCCESSFUL DB CONN\n");

    let _ = log.write_all(b"SUCCESSFUL API SETUP\n");
    //Now Daemon is in proper environment, with a database connection

    let socket = socket::set_socket(&log);
    if socket.is_none() {
        return; // Socket binding failed, terminate (already logged)
    }
    let mut socket = socket.unwrap();

    let database_lock = Arc::new(Mutex::new(db_conn)); // mutex to sync database use, as threads have different statements

    let db_lock_notifier = Arc::clone(&database_lock);
    let log_lock = Arc::new(Mutex::new(log));
    let log_lock_notifier = Arc::clone(&log_lock);
    thread::spawn(move || {
        let conn = Connection::new_session();
        if let Err(e) = &conn {
            return;
        }
        let conn = conn.unwrap();

        loop {
            thread::sleep(Duration::from_millis(60000));
            let mut log = log_lock_notifier.lock().unwrap();
            log.write_all(b"woke up\n");
            let mut db_lock = db_lock_notifier.lock().unwrap();
            log.write_all(b"got db lock\n");
            let notifier_statements = db::NotificationStatements::new(&*db_lock, &mut *log); //TODO: fix the log issue
            if notifier_statements.is_none() {
                log.write_all(b"failed to construct statements");
                return;
            }
            log.write_all(b"made statements");
            let mut notifier_statements = notifier_statements.unwrap();

            let reminders_to_send = notifier_statements.get_notifications(&*log);

            if !reminders_to_send.is_none() {
                let reminders = reminders_to_send.unwrap();
                let fmt_str = format!(
                    "At time {}, found {} reminders to send",
                    chrono::offset::Local::now(),
                    reminders.len()
                );
                log.write_all(fmt_str.as_bytes());
                for (id, reminder) in reminders {
                    comm::notify(&reminder.serialize(), &conn, &*log);
                    notifier_statements.update_notification((id, reminder), &*log);
                }
            } else {
                let fmt_str = format!(
                    "At time {}, found {} reminders to send",
                    chrono::offset::Local::now(),
                    0
                );
                log.write_all(fmt_str.as_bytes());
            }
        }
    });

    loop {
        let data = socket.recv_multipart(0);

        // Get access to log
        let mut log = log_lock.lock().unwrap();
        let db_conn = database_lock.lock().unwrap();

        let api_statements = db::PreparedStatements::new(&db_conn, &mut log);
        if api_statements.is_none() {
            return;
        }
        let mut api_statements = api_statements.unwrap();

        let _ = log.write_all(b"RECEIVED MESSAGE\n");
        if let Err(err) = data {
            let fmt_str = format!("Error while receiving data: {}\n", err);
            let _ = log.write_all(fmt_str.as_bytes());
        }
        let data = data.unwrap();
        comm::handle_message(&data, &mut *log, &mut api_statements, &socket);
        let mut new_sock = socket::set_socket(&*log);
        while new_sock.is_none() {
            new_sock = socket::set_socket(&*log);
        }
        socket = new_sock.unwrap(); // Reset socket
    }
}
