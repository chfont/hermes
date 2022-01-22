use std::fs::File;
use std::io::Write;

use rusqlite::{self, Statement, Connection, params};
use rusqlite::Error;
use crate::reminder::{Reminder, self};

// Struct will hold prepared statements for necessary API
pub struct PreparedStatements<'c> {
    list_statement: Statement<'c>,
    insert_statement: Statement<'c>,
    delete_statement: Statement<'c>
}

impl<'c> PreparedStatements<'c> {
    pub fn new<'a>(conn: &'a Connection, log: &mut File) -> Option<PreparedStatements<'a>> {
	let list_stmt = conn.prepare("SELECT * FROM reminder");
	if let Err(e) = list_stmt {
	    let fmt_str = format!("Failed to prepare list_stmt {}\n", e);
	    let _ = log.write_all(fmt_str.as_bytes());
	    return None;
	}
	let list_stmt = list_stmt.unwrap();
	
	let insert_stmt = conn.prepare("INSERT INTO reminder (frequency, message, month, day, year, hour, minute,\
					n) VALUES (?,?,?,?,?,?,?,?)");
	if let Err(e) = insert_stmt {
	    let fmt_str = format!("Failed to prepare insert statement: {}\n",e);
	    let _ = log.write_all(fmt_str.as_bytes());
	    return None;
	}
	let insert_stmt = insert_stmt.unwrap();

	let delete_stmt = conn.prepare("DELETE FROM reminder WHERE id = ?");

	if let Err(e) = delete_stmt {
	    let fmt_str = format!("Failed to prepare delete statement: {}\n",e);
	    let _ = log.write_all(fmt_str.as_bytes());
	    return None;
	}
	let delete_stmt = delete_stmt.unwrap();

	return Some(PreparedStatements {
	    list_statement: list_stmt,
	    insert_statement: insert_stmt,
	    delete_statement: delete_stmt
	});
    }

    pub fn list(&mut self, mut log: &File) -> Option<Vec<(u32,Reminder)>> {
	let reminders = self.list_statement
	    .query_map([], |row: &rusqlite::Row | -> Result<(u32,Reminder), Error> {

		let mut n : Option<u32> = row.get(8).unwrap();
		if let Some(v) = n {
		    n = Some(u32::from_be(v));
		}
		
		Ok((row.get(0)?,Reminder::new(
		reminder::deserialize_frequency(row.get(1).unwrap()).unwrap(),
		row.get(3)?,
		row.get(4)?,
		u32::from_be(row.get(5)?),
		row.get(6)?,
		row.get(7)?,
		n,
		row.get(2)?
	    )))}

	    );
	if let Err(err) = reminders {
	    let fmt_str = format!("Error retrieving data: {}\n", err);
	    let _ = log.write_all(fmt_str.as_bytes());
	    return None;
	}
	let reminders = reminders.unwrap();
	let mut rem_vec: Vec<(u32,Reminder)> = Vec::new();
	for reminder in reminders {
	    if let Err(e) = reminder {
		let fmt_str = format!("Error retrieving reminder: {}\n", e);
		let _ = log.write_all(fmt_str.as_bytes());
	    } else {
		let reminder = reminder.unwrap();
		rem_vec.push(reminder);
	    }
	    
	}
	return Some(rem_vec);
    }

    pub fn add(&mut self, reminder: Reminder, mut log: &File) -> bool {
	let params = reminder.as_tuple();
	// Handle n to big endian.
	let mut n = params.6;
	if let Some(value) = n {
	    n = Some(value.to_be());
	}
	
	let res = self.insert_statement.execute(
	    params!(
		reminder::serialize_frequency(params.0),
		*(params.7),
		params.1,
		params.2,
		params.3.to_be(),
		params.4,
		params.5,
		n
	    )
	);

	if let Err(err) = res {
	    let fmt_str = format!("Failed to insert reminder: {}\n", err);
	    let _ = log.write_all(fmt_str.as_bytes());
	    return false;
	} else {
	    let res = res.unwrap();
	    if res != 1 {
		let fmt_str = format!("Row count changed: {}\n", res);
		let _ = log.write_all(fmt_str.as_bytes());
		return false;
	    }
	}
	return true;
    }

    pub fn delete(&mut self, id: u32, mut log: &File) -> bool {
	let rows = self.delete_statement.execute(params!(id));
	if let Err(err) = rows {
	    let fmt_str = format!("Error deleting id {}: {}\n", id, err);
	    let _ = log.write_all(fmt_str.as_bytes());
	    return false;
	}
	let count = rows.unwrap();
	if count == 0 {
	    return false;
	}
	return true;
    }
}
