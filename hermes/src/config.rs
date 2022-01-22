use std::env;
use std::fs::File;
use std::fs;
use std::io::Write;
use rusqlite;

const DIRECTORY : &str = ".hermes";
const DATABASE : &str = "hermes.sqlite";

/* Setups environment: .hermes directory, and database file */
pub fn initialize_environment(log: &File) -> Option<rusqlite::Connection> {
    let directory_setup = setup_directory(log);
    if !directory_setup {
	return None;
    } else {
	return setup_database(log);
    }
}

/* Setup .hermes/ in HOME, to store database in a standard area */
fn setup_directory(mut log: &File) -> bool {
    if let Ok(v) = env::var("HOME") {
	if let Err(e) = env::set_current_dir(&v){
	    let fmt_str = format!("Failed to set directory to: {} Error: {}\n", v.as_str(), e);
	    let _ = log.write_all(fmt_str.as_bytes());
	    return false;
	}
	let in_dir = build_dir(log);
	if ! in_dir {
	    return false;
	}
	return true;
    } else {
	// Log failure
	let _ = log.write_all(b"Failed to read HOME environment variable, ensure it is set\n");
	return false;
    }
}
/* Handles construction of directory, to be used in setup_directory(). Split to improve readability */
fn build_dir(mut log: &File) -> bool {
    let attr = fs::metadata(DIRECTORY);
    if attr.is_err() {
	// Directory doesn't exist yet
	let created = create_directory(log);
	if !created {
	    return false; // Already logged
	}
    }
    
    // Now that directory exists, move into it
    let dir = env::current_dir();
    if let Err(e) = dir {
	let fmt_str = format!("Error getting current Directory: {}\n", e);
	let _ = log.write_all(fmt_str.as_bytes());
	return false;
    } else {
	let mut dir = dir.unwrap();
	dir.push(DIRECTORY);
	if let Err(error) = env::set_current_dir(&dir) {
		let fmt_str = format!("Error entering directory {} : {}\n",dir.into_os_string().into_string().unwrap(), error);
		let _ = log.write_all(fmt_str.as_bytes());
		return false;
	    };
	return true;
    }
    // Now, .hermes/ exists in HOME, and current directory is .hermes/
}

// Quick function to create a folder, if it doesn't exist. To be used in build_dir
fn create_directory(mut log: &File) -> bool {
    let _ = log.write_all(b"Creating directory for hermes configuration\n");
    let res = fs::create_dir(DIRECTORY);
    if let Err(err) = res {
	let fmt_str = format!("Error creating Directory: {}\n", err);
	let _ = log.write_all(fmt_str.as_bytes());
	return false;
    }
    return true;
}

/* Setup database connection, creating db and table if it is not present */
fn setup_database(mut log: &File)-> Option<rusqlite::Connection> {
    let db_conn = rusqlite::Connection::open(DATABASE);
    if let Err(err) = db_conn {
	let fmt_str = format!("Error opening database at {}: {}\n", DATABASE, err);
	let _ = log.write_all(fmt_str.as_bytes());
	let dir = env::current_dir().unwrap();
	let _ = log.write_all(dir.into_os_string().into_string().unwrap().as_bytes());
	return None;
    }
    let db_conn = db_conn.unwrap();
    let created_table = db_conn.execute(
	" CREATE TABLE IF NOT EXISTS reminder (\
           id INTEGER PRIMARY KEY,\
           frequency INTEGER NOT NULL,\
           message TEXT NOT NULL,\
           month INTEGER NOT NULL,\
           day INTEGER NOT NULL,\
	   year INTEGER NOT NULL,\
           hour INTEGER NOT NULL,\
           minute INTEGER NOT NULL,\
	   n INTEGER
	   );",
	[]
    );
    if let Err(err) = created_table {
	let fmt_str = format!("Error creating database table: {}\n", err);
	let _ = log.write_all(fmt_str.as_bytes());
	return None;
    }
    return Some(db_conn);
}
