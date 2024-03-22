use chrono;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Author<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub time: chrono::DateTime<chrono::Local>,
}

impl<'a> Author<'a> {
    pub fn new(name: &'a str, email: &'a str) -> Self {
        Self {
            name,
            email,
            time: chrono::Local::now(),
        }
    }
}

impl<'a> fmt::Display for Author<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let timestamp = self.time.format("%s %z").to_string();
        write!(f, "{} <{}> {}", self.name, self.email, timestamp)
    }
}
