use std::{fs::File, io::Write};

#[derive(PartialEq, Debug)]
enum Frequency {
    DAILY,
    ONCE,
    WEEKLY,
    NDAYS
}

const MIN_REMINDER_LENGTH_BYTES: usize = 14;

// Note: Byte Order is expected to be Big Endian, as is standard in networking
#[derive(PartialEq, Debug)]
pub struct Reminder {
    frequency: Frequency,
    month: u8,
    day: u8,
    year: u32,
    hour: u8,
    minute: u8,
    n: Option<u32>,
    message: String
}

fn deserialize_frequency(byte: u8) -> Option<Frequency> {
    return match byte {
	1 => Some(Frequency::DAILY),
	2 => Some(Frequency::ONCE),
	3 => Some(Frequency::WEEKLY),
	4 => Some(Frequency::NDAYS),
	_ => None
    };
}

fn deserialize_u32(bytes: Vec<u8>, mut log: &File) -> Option<u32> {
    if bytes.len() != 4 {
	let _ = log.write_all(b"Received Invalid value for n");
	return None;
    }
    // Bytes are specified to come as BIG ENDIAN
    return Some((bytes[3] as u32) << 24
	+  (bytes[2] as u32) << 16
	+  (bytes[1] as u32) << 8
	+  (bytes[0] as u32));
}

impl Reminder {

    pub fn get_message(&self) -> &String {
	return &self.message;
    }

    pub fn deserialize_reminder(vec: &Vec<u8>, mut log: &File) -> Option<Reminder> {
	if vec.len() < MIN_REMINDER_LENGTH_BYTES { // Message has 13 bytes for non body component
	    let fmt_str = format!("Invalid Message Length: {}\n", vec.len());
	    let _ = log.write_all(fmt_str.as_bytes());
	    return None;
	}
	
	let freq = deserialize_frequency(vec[0]);
	if let None = freq {
	    let fmt_str = format!("Invalid Frequency byte: {}\n", vec[0]);
	    let _ = log.write_all(fmt_str.as_bytes());
	    return None;
	}
	let freq = freq.unwrap();

	let month = vec[1];
	let day = vec[2];
	
	let year = deserialize_u32(vec![vec[3], vec[4], vec[5], vec[6]], &log);
	if let None = year {
	    return None; // Already logged
	}
	let year = year.unwrap();

	let hour = vec[7];
	let minute = vec[8];
	
	let n = deserialize_u32(vec![vec[9], vec[10], vec[11], vec[12]], &log);
	if let None = n {
	    match freq {
		Frequency::NDAYS => {
		    let _ = log.write_all(b"N not specified, but N days is the frequency\n");
		    return None;
		},
		_ => {}
	    }
	}

	// Only body remains
	let (_, body) = vec.split_at(13);
	let message = std::str::from_utf8(body);
	if let Err(e) = message {
	    let fmt_str = format!("Error decoding message body: {}\n", e.to_string());
	    let _ = log.write_all(fmt_str.as_bytes());
	    return None;
	}
	
	let message = message.unwrap();

	let reminder = Reminder {
	    frequency: freq,
	    month,
	    day,
	    year,
	    hour,
	    minute,
	    n,
	    message: message.to_string()
	};
	return Some(reminder);
    }
}

#[cfg(test)]
mod tests{
    use crate::reminder;
    #[test]
    fn fails_deserialize_if_vec_short(){
	let vec: Vec<u8> = vec!(3,4,5,6);
	let mut file = std::fs::File::open("/dev/null").unwrap(); //TODO: mock logger interface so I don't need to do this
	assert_eq!(reminder::Reminder::deserialize_reminder(&vec, &mut file), None);
	
    }

    #[test]
    fn successful_deserialize(){

	let vec: Vec<u8> = vec!(
	    1,
	    1,
	    1,
	    2,2,2,2,
	    1,
	    1,
	    2,2,2,2,
	    72,69,76,76,79
	);
	let mut file = std::fs::File::open("/dev/null").unwrap();
	let reminder = reminder::Reminder::deserialize_reminder(&vec, &mut file);
	assert_ne!(&reminder, &None);
	assert_eq!(reminder.unwrap().get_message(), "HELLO");
    }
}
