use chrono_tz::Tz;
use iana_time_zone::GetTimezoneError;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TryGetLocalTimezoneError {
    #[error("could not find timezone: {0}")]
    ChronoTzNotFound(String),
    #[error("could not get local timezone: {0}")]
    GetTimezone(GetTimezoneError),
}

pub fn get_timezone_by_name(name: &str) -> Option<Tz> {
    Tz::from_str(name).ok()
}

pub fn try_get_local_timezone() -> Result<Tz, TryGetLocalTimezoneError> {
    let timezone_str = match iana_time_zone::get_timezone() {
        Ok(timezone) => Ok(timezone),
        Err(e) => Err(TryGetLocalTimezoneError::GetTimezone(e)),
    }?;

    let timezone = get_timezone_by_name(&timezone_str);
    
    timezone.ok_or_else(|| TryGetLocalTimezoneError::ChronoTzNotFound(timezone_str))
}
