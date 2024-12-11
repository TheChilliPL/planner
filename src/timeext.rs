use chrono::{DateTime, TimeZone};

pub trait TimeExt {
    fn to_stamp(&self) -> String;
}

impl <Tz: TimeZone> TimeExt for DateTime<Tz> {
    fn to_stamp(&self) -> String {
        self.naive_local().format("%Y%m%dT%H%M%S").to_string()
    }
}