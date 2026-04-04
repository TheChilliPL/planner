use crate::ical::SerializeToICal;
use crate::time::timeext::TimeExt;
use chrono::{DateTime, NaiveDate};
use chrono_tz::Tz;
use std::io::Write;

pub enum EventTime {
    FullDay(NaiveDate),
    Timed {
        start: DateTime<Tz>,
        /// Start time of the event.
        ///
        /// Corresponds to the `DTSTART` property in iCalendar.
        /// End time of the event.
        ///
        /// Corresponds to the `DTEND` property in iCalendar.
        end: DateTime<Tz>,
    }
}

pub struct VEvent {
    /// Unique identifier of the event
    ///
    /// Corresponds to the `UID` property in iCalendar.
    pub uid: String,
    /// Timestamp of when the event was created/last modified.
    ///
    /// Corresponds to the `DTSTAMP` property in iCalendar.
    pub created: DateTime<Tz>,
    pub time: EventTime,
    /// Summary or title of the event.
    ///
    /// Corresponds to the `SUMMARY` property in iCalendar.
    pub summary: String,
    /// Description of the event.
    ///
    /// Corresponds to the `DESCRIPTION` property in iCalendar.
    pub description: Option<String>,
    /// Location of the event.
    ///
    /// Corresponds to the `LOCATION` property in iCalendar.
    pub location: Option<String>,
}

impl SerializeToICal for VEvent {
    fn serialize_to_ical(&self, write: &mut dyn Write) -> eyre::Result<()> {
        fn datetime_to_ical_string(dt: &DateTime<Tz>) -> String {
            let tz = dt.timezone().name();
            format!(";TZID={}:{}", tz, dt.to_stamp())
        }

        write!(write, "BEGIN:VEVENT\r\n")?;
        write!(write, "UID:{}\r\n", self.uid)?;
        write!(
            write,
            "DTSTAMP{}\r\n",
            datetime_to_ical_string(&self.created)
        )?;
        match &self.time {
            EventTime::FullDay(date) => {
                write!(write, "DTSTART;VALUE=DATE:{}\r\n", date.format("%Y%m%d"))?;
            }
            EventTime::Timed { start, end } => {
                write!(write, "DTSTART{}\r\n", datetime_to_ical_string(start))?;
                write!(write, "DTEND{}\r\n", datetime_to_ical_string(end))?;
            }
        }
        write!(write, "SUMMARY:{}\r\n", self.summary.replace("\n", "\\n"))?;
        if let Some(location) = &self.location {
            write!(write, "LOCATION:{}\r\n", location.replace("\n", "\\n"))?;
        }
        if let Some(description) = &self.description {
            write!(write, "DESCRIPTION:{}\r\n", description.replace("\n", "\\n"))?;
        }
        write!(write, "END:VEVENT\r\n")?;

        Ok(())
    }
}
