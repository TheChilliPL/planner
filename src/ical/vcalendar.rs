use crate::ical::SerializeToICal;
use crate::ical::vevent::VEvent;

pub struct VCalendar {
    pub prod_id: String,
    pub version: String,
    pub events: Vec<VEvent>,
}

impl SerializeToICal for VCalendar {
    fn serialize_to_ical(&self, write: &mut dyn std::io::Write) -> eyre::Result<()> {
        write!(write, "BEGIN:VCALENDAR\r\n")?;
        write!(write, "PRODID:{}\r\n", self.prod_id)?;
        write!(write, "VERSION:{}\r\n", self.version)?;

        for event in &self.events {
            event.serialize_to_ical(write)?;
        }

        write!(write, "END:VCALENDAR\r\n")?;

        Ok(())
    }
}
