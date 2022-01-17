#[derive(PartialEq)]
pub enum Frequency {
    DAILY,
    ONCE,
    WEEKLY,
    NDAYS
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

impl Reminder {

    pub fn new(freq: Frequency, msg: String, month: u8, day: u8,year: u32, hour: u8, minute: u8, n: Option<u32>) -> Self {
	return Self {
	    frequency: freq,
	    content: msg,
	    month,
	    day,
	    year,
	    hour,
	    minute,
	    n
	};
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
	return vec;
    }
}
