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
            name: name.to_string(),
            email: email.to_string(),
            time: chrono::Local::now(),
        }
    }
}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let timestamp = self.time.format("%s %z").to_string();
        write!(f, "{} <{}> {}", self.name, self.email, timestamp)
    }
}