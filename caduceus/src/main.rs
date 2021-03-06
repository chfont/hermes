use std::env;
pub mod info;
pub mod ipc;
pub mod reminder;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Invalid Argument count. Run with -h to see help");
        return;
    }
    match args[1].as_str() {
        "-h" => {
            info::help_info();
        }
        "help" => {
            info::help_info();
        }
        "-v" => {
            info::version_info();
        }
        "version" => {
            info::version_info();
        }
        "add" => {
            ipc::add_reminder();
        }
        "list" => {
            ipc::list_reminders();
        }
        "remove" => {
            ipc::delete_reminder();
        }
        _ => {
            println!("Argument not recognized");
        }
    }
}
