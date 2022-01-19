#[derive(PartialEq)]
pub enum Frequency {
    DAILY,
    ONCE,
    WEEKLY,
    NDAYS
}

impl ToString for Frequency {
    fn to_string(&self) -> String {
        match self {
	    Self::DAILY => "Daily".to_string(),
	    Self::ONCE => "Once".to_string(),
	    Self::WEEKLY => "Weekly".to_string(),
	    Self::NDAYS => "Every N Days".to_string()
	}
    }
}

pub struct Reminder {
    frequency: Frequency,
    content: String,
    month: u8,
    day: u8,
    year: u32,
    hour: u8,
    minute: u8,
    n: Option<u32>
}

fn deserialize_u32(vec: Vec<u8>) -> Option<u32> {
	if vec.len() != 4 {
	    return None;
	}
	return Some(u32::from_be_bytes([vec[0],vec[1], vec[2],vec[3]]));
}

impl Reminder {

    pub fn deserialize_reminder(vec: Vec<u8>) -> Option<Reminder> {
	if vec.len() < 14 {
	    return None;
	}
	let frequency: Option<Frequency> = match vec[0] {
	    1 => Some(Frequency::DAILY),
	    2 => Some(Frequency::ONCE),
	    3 => Some(Frequency::WEEKLY),
	    4 => Some(Frequency::NDAYS),
	    _ => None
	};
	if frequency.is_none() {
	    return None;
	}
	let frequency = frequency.unwrap();
	let month = vec[1];
	let day = vec[2];
	// TODO: year
	let year = deserialize_u32(vec!(vec[3],vec[4], vec[5], vec[6]));
	if year.is_none() {
	    return None;
	}
	let year = year.unwrap();
	
	let hour = vec[7];
	let minute = vec[8];
	let n_internal = deserialize_u32(vec!(vec[9], vec[10], vec[11], vec[12]));
	if n_internal.is_none() {
	    return None;
	}
	let n_internal = n_internal.unwrap();
	let mut n = Some(n_internal);
	if n_internal == 0 {
	    n = None;
	}
	let (_, msg) = vec.split_at(13);
	let data = std::str::from_utf8(msg);
	if data.is_err() {
	    return None;
	}
	let data = data.unwrap();

	Some(Reminder::new(
	    frequency,
	    data.to_string(),
	    month,
	    day,
	    year,
	    hour,
	    minute,
	    n
	))
    }

    pub fn print(&self) {
	let mut fmt_str = format!("REMINDER: {} | {}/{}/{} {:02}:{:02} | Frequency: {}", self.content, self.month, self.day, self.year, self.hour, self.minute, self.frequency.to_string());
	if self.frequency == Frequency::NDAYS {
	    fmt_str += format!(": {}", self.n.unwrap()).as_str();
	}

	println!("{}", fmt_str);
    }
    
    pub fn new(freq: Frequency, msg: String, month: u8, day: u8,year: u32, hour: u8, minute: u8, n: Option<u32>) -> Self {
	Self {
	    frequency: freq,
	    content: msg,
	    month,
	    day,
	    year,
	    hour,
	    minute,
	    n
	}
    }
    
    pub fn serialize(&self) -> Vec<u8> {
	let mut vec: Vec<u8> = Vec::new();
	let freq: u8 = match &self.frequency {
	    Frequency::DAILY => 1,
	    Frequency::ONCE => 2,
	    Frequency::WEEKLY => 3,
	    Frequency::NDAYS => 4
	};
	vec.push(freq);
	vec.push(self.month);
	vec.push(self.day);
	
	for byte in self.year.to_be_bytes() {
	    vec.push(byte);
	}

	vec.push(self.hour);
	vec.push(self.minute);
	// n
	match self.n {
	    None => {
		let mut i = 0;
		while i < 4 {
		    vec.push(0);
		    i += 1;
		}
	    },
	    Some(value) => {
		for byte in value.to_be_bytes() {
		    vec.push(byte);
		}
	    }
	}
	
	let mut v = self.content.as_bytes().to_vec();
	vec.append(&mut v);
	vec
    }
}
