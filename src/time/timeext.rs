use chrono::{DateTime, TimeZone};

pub trait TimeExt {
    fn to_stamp(&self) -> String;
}

impl<Tz: TimeZone> TimeExt for DateTime<Tz> {
    fn to_stamp(&self) -> String {
        self.naive_local().format("%Y%m%dT%H%M%S").to_string()
    }
}

pub trait TimeDeltaExt {
    fn to_human_readable(&self) -> String;
}

impl TimeDeltaExt for chrono::TimeDelta {
    fn to_human_readable(&self) -> String {
        let total_seconds = self.num_seconds().abs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}
