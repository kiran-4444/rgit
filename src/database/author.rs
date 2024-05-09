use chrono;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Author {
    pub name: String,
    pub email: String,
    pub time: chrono::DateTime<chrono::Local>,
}

impl Author {
    pub fn new(name: &str, email: &str) -> Self {
        Self {
            name: name.to_owned(),
            email: email.to_owned(),
            time: chrono::Local::now(),
        }
    }

    pub fn parse(line: &str) -> Self {
        let mut parts = line.splitn(2, ' ');
        let name = parts.next().unwrap();
        let email = parts.next().unwrap();
        let email = &email[1..email.len() - 1];
        let time = chrono::Local::now();
        Self {
            name: name.to_owned(),
            email: email.to_owned(),
            time,
        }
    }
}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let timestamp = self.time.format("%s %z").to_string();
        write!(f, "{} <{}> {}", self.name, self.email, timestamp)
    }
}
