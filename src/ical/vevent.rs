use std::io::Write;
use chrono::{DateTime};
use chrono_tz::Tz;
use crate::ical::SerializeToICal;
use crate::time::timeext::TimeExt;

pub struct VEvent {
    /// Unique identifier of the event
    ///
    /// Corresponds to the `UID` property in iCalendar.
    pub uid: String,
    /// Timestamp of when the event was created/last modified.
    ///
    /// Corresponds to the `DTSTAMP` property in iCalendar.
    pub created: DateTime<Tz>,
    /// Start time of the event.
    ///
    /// Corresponds to the `DTSTART` property in iCalendar.
    pub start: DateTime<Tz>,
    /// End time of the event.
    ///
    /// Corresponds to the `DTEND` property in iCalendar.
    pub end: DateTime<Tz>,
    /// Summary or title of the event.
    ///
    /// Corresponds to the `SUMMARY` property in iCalendar.
    pub summary: String,
    /// Description of the event.
    ///
    /// Corresponds to the `DESCRIPTION` property in iCalendar.
    pub description: String,
    /// Location of the event.
    ///
    /// Corresponds to the `LOCATION` property in iCalendar.
    pub location: String,
}

impl SerializeToICal for VEvent {
    fn serialize_to_ical(&self, write: &mut dyn Write) -> eyre::Result<()> {
        fn datetime_to_ical_string(dt: &DateTime<Tz>) -> String {
            let tz = dt.timezone().name();
            format!(";TZID={}:{}", tz, dt.to_stamp())
        }

        write!(write, "BEGIN:VEVENT\r\n")?;
        write!(write, "UID:{}\r\n", self.uid)?;
        write!(write, "DTSTAMP{}\r\n", datetime_to_ical_string(&self.created))?;
        write!(write, "DTSTART{}\r\n", datetime_to_ical_string(&self.start))?;
        write!(write, "DTEND{}\r\n", datetime_to_ical_string(&self.end))?;
        write!(write, "SUMMARY:{}\r\n", self.summary.replace("\n", "\\n"))?;
        write!(write, "LOCATION:{}\r\n", self.location.replace("\n", "\\n"))?;
        write!(write, "DESCRIPTION:{}\r\n", self.description.replace("\n", "\\n"))?;
        write!(write, "END:VEVENT\r\n")?;

        Ok(())
    }
}
