use std::fmt::Formatter;
use chrono::{DateTime, MappedLocalTime, NaiveDate, NaiveTime, TimeDelta};
use chrono_tz::Tz;
use serde::{de, Deserialize, Deserializer};
use serde::de::{Error, Visitor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NaiveTimePeriod {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

impl NaiveTimePeriod {
    fn new(start: NaiveTime, end: NaiveTime) -> Self {
        Self { start, end }
    }
    
    pub(crate) fn from_hm_hm(start_hour: u32, start_min: u32, end_hour: u32, end_min: u32) -> Self {
        Self::new(
            NaiveTime::from_hms_opt(start_hour, start_min, 0).unwrap(),
            NaiveTime::from_hms_opt(end_hour, end_min, 0).unwrap(),
        )
    }

    fn get_duration(&self) -> TimeDelta {
        self.end.signed_duration_since(self.start)
    }
    
    fn on_day(&self, day: NaiveDate, tz: Tz) -> Option<(DateTime<Tz>, DateTime<Tz>)> {
        let start = day.and_time(self.start).and_local_timezone(tz);
        let end = day.and_time(self.end).and_local_timezone(tz);

        use MappedLocalTime as MLT;
        match (start, end) {
            (MLT::Single(start), MLT::Single(end)) => Some((start, end)),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for NaiveTimePeriod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NaiveTimePeriodVisitor;

        impl<'de> Visitor<'de> for NaiveTimePeriodVisitor {
            type Value = NaiveTimePeriod;

            fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                f.write_str("a string in the format 'HH:MM-HH:MM'")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let mut parts = v.split('-');
                let start_str = parts.next().ok_or_else(|| {
                    E::invalid_value(de::Unexpected::Str(v), &"expected format 'HH:MM-HH:MM'")
                })?;
                let end_str = parts.next().ok_or_else(|| {
                    E::invalid_value(de::Unexpected::Str(v), &"expected format 'HH:MM-HH:MM'")
                })?;

                if parts.next().is_some() {
                    return Err(E::invalid_value(
                        de::Unexpected::Str(start_str),
                        &"too many '-' separators"
                    ))
                }

                let start = NaiveTime::parse_from_str(start_str.trim(), "%H:%M")
                    .or_else(|_| NaiveTime::parse_from_str(start_str.trim(), "%-H:%M"))
                    .map_err(|_| E::invalid_value(de::Unexpected::Str(start_str), &"invalid start time"))?;

                let end = NaiveTime::parse_from_str(end_str.trim(), "%H:%M")
                    .or_else(|_| NaiveTime::parse_from_str(end_str.trim(), "%-H:%M"))
                    .map_err(|_| E::invalid_value(de::Unexpected::Str(end_str), &"invalid end time"))?;

                Ok(NaiveTimePeriod::new(start, end))
            }
        }

        deserializer.deserialize_str(NaiveTimePeriodVisitor)
    }
}
