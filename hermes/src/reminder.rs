use chrono::{prelude::*, ParseError};
use std::{fs::File, io::Write};

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Frequency {
    DAILY,
    ONCE,
    WEEKLY,
    NDAYS,
}

const MIN_REMINDER_LENGTH_BYTES: usize = 14;

// Note: Byte Order is expected to be Big Endian, as is standard in networking
#[derive(PartialEq, Debug)]
pub struct Reminder {
    pub frequency: Frequency,
    month: u8,
    day: u8,
    year: u32,
    hour: u8,
    minute: u8,
    pub n: Option<u32>,
    message: String,
}

pub fn deserialize_frequency(byte: u8) -> Option<Frequency> {
    return match byte {
        1 => Some(Frequency::DAILY),
        2 => Some(Frequency::ONCE),
        3 => Some(Frequency::WEEKLY),
        4 => Some(Frequency::NDAYS),
        _ => None,
    };
}

pub fn serialize_frequency(freq: Frequency) -> u8 {
    return match freq {
        Frequency::DAILY => 1,
        Frequency::ONCE => 2,
        Frequency::WEEKLY => 3,
        Frequency::NDAYS => 4,
    };
}

fn deserialize_u32(bytes: Vec<u8>, mut log: &File) -> Option<u32> {
    if bytes.len() != 4 {
        let _ = log.write_all(b"Received Invalid value for n");
        return None;
    }
    // Bytes are specified to come as BIG ENDIAN
    return Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
}

impl Reminder {
    pub fn get_message(&self) -> &String {
        return &self.message;
    }

    // For use in database, without making fields public
    pub fn as_tuple(&self) -> (Frequency, u8, u8, u32, u8, u8, Option<u32>, &String) {
        return (
            self.frequency,
            self.month,
            self.day,
            self.year,
            self.hour,
            self.minute,
            self.n,
            &self.message,
        );
    }

    pub fn new(
        frequency: Frequency,
        month: u8,
        day: u8,
        year: u32,
        hour: u8,
        minute: u8,
        n: Option<u32>,
        message: String,
    ) -> Self {
        return Reminder {
            frequency,
            month,
            day,
            year,
            hour,
            minute,
            n,
            message,
        };
    }

    pub fn deserialize_reminder(vec: &Vec<u8>, mut log: &File) -> Option<Reminder> {
        if vec.len() < MIN_REMINDER_LENGTH_BYTES {
            // Message has 13 bytes for non body component
            let fmt_str = format!("Invalid Message Length: {}\n", vec.len());
            let _ = log.write_all(fmt_str.as_bytes());
            return None;
        }

        let freq = deserialize_frequency(vec[0]);
        if freq.is_none() {
            let fmt_str = format!("Invalid Frequency byte: {}\n", vec[0]);
            let _ = log.write_all(fmt_str.as_bytes());
            return None;
        }
        let freq = freq.unwrap();

        let month = vec[1];
        let day = vec[2];

        let year = deserialize_u32(vec![vec[3], vec[4], vec[5], vec[6]], log)?;

        let hour = vec[7];
        let minute = vec[8];

        let n = deserialize_u32(vec![vec[9], vec[10], vec[11], vec[12]], log);
        if n.is_none() && freq == Frequency::NDAYS {
            let _ = log.write_all(b"N not specified, but N days is the frequency\n");
        }

        // Only body remains
        let (_, body) = vec.split_at(13);
        let message = std::str::from_utf8(body);
        if let Err(e) = message {
            let fmt_str = format!("Error decoding message body: {}\n", e);
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
            message: message.to_string(),
        };
        return Some(reminder);
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        let freq: u8 = match &self.frequency {
            Frequency::DAILY => 1,
            Frequency::ONCE => 2,
            Frequency::WEEKLY => 3,
            Frequency::NDAYS => 4,
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
            }
            Some(value) => {
                for byte in value.to_be_bytes() {
                    vec.push(byte);
                }
            }
        }

        let mut v = self.message.as_bytes().to_vec();
        vec.append(&mut v);
        return vec;
    }

    pub fn to_datetime(&self) -> Option<chrono::DateTime<FixedOffset>> {
        let time = chrono::Local::now();
        let offset = time.offset();
        let datetime = offset.datetime_from_str(
            format!(
                "{:02} {:02} {} {:02} {:02}",
                self.day, self.month, self.year, self.minute, self.hour
            )
            .as_str(),
            "%d %m %Y %M %H",
        );
        if datetime.is_err() {
            return None;
        }
        return Some(datetime.unwrap());
    }
}

#[cfg(test)]
mod tests {
    use crate::reminder;
    #[test]
    fn fails_deserialize_if_vec_short() {
        let vec: Vec<u8> = vec![3, 4, 5, 6];
        let mut file = std::fs::File::open("/dev/null").unwrap(); //TODO: mock logger interface so I don't need to do this
        assert_eq!(
            reminder::Reminder::deserialize_reminder(&vec, &mut file),
            None
        );
    }

    #[test]
    fn successful_deserialize() {
        let vec: Vec<u8> = vec![1, 1, 1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 72, 69, 76, 76, 79];
        let mut file = std::fs::File::open("/dev/null").unwrap();
        let reminder = reminder::Reminder::deserialize_reminder(&vec, &mut file);
        assert_ne!(&reminder, &None);
        assert_eq!(reminder.unwrap().get_message(), "HELLO");
    }
}
