pub enum Frequency {
    DAILY,
    ONCE,
    WEEKLY
}

pub struct Message {
    frequency: Frequency,
    content: String
}

impl Message {

    pub fn new(f: Frequency, msg: String) -> Self {
	return Message {
	    frequency: f,
	    content: msg,
	};
    }
    
    pub fn serialize(&self) -> Vec<u8> {
	let freq: u8 = match &self.frequency {
	    Frequency::DAILY => 1,
	    Frequency::ONCE => 2,
	    Frequency::WEEKLY => 3
	};
	let mut v = self.content.as_bytes().to_vec();
	v.insert(0,freq);
	return v;
    }
}
