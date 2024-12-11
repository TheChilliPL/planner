pub mod vcalendar;
pub mod vevent;

use std::io::Write;
use std::path::Path;

/// Trait for serializing an object to iCal format.
pub trait SerializeToICal {
    /// Serializes the object to iCal format and writes it to the provided writer.
    ///
    /// All implementations must ensure that the output is valid iCal format.
    /// All written lines should be UTF-8-encoded and end with CRLF (`\r\n`).
    ///
    /// The writer might have to be flushed after writing for the data to be fully written.
    fn serialize_to_ical(&self, write: &mut dyn Write) -> eyre::Result<()>;
}

impl dyn SerializeToICal {
    /// Serializes the object to iCal format and writes it to a file at the specified path.
    ///
    /// Creates the file if it does not exist, or overwrites it if it does.
    /// All written lines will be UTF-8-encoded and end with CRLF (`\r\n`).
    pub fn serialize_to_ical_file(&self, path: &Path) -> eyre::Result<()> {
        let mut file = std::fs::File::create(path)?;
        self.serialize_to_ical(&mut file)?;
        file.flush()?;
        Ok(())
    }
}
