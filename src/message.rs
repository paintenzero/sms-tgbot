use std::fmt::Display;
use serde::{Serialize, Deserialize};
use chrono::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub from: String,
    index: u32,
    pub text: String,
    timestamp: String,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Timestamp: {}\nFrom: {}\nText: {}", self.timestamp, self.from, self.text)
    }
}

impl Message {
    pub fn to_html(&self) -> String {
        format!("{}) <b>{}</b>\n<i>{}</i>\n<pre>{}</pre>", self.index, self.from, self.timestamp, self.text)
    }

    pub fn datetime(&self) -> Result<DateTime<FixedOffset>, chrono::format::ParseError> {
        DateTime::parse_from_str(&self.timestamp, "%Y-%m-%d %H:%M:%S %z")
    }
}