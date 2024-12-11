use chrono::{Datelike, Local};
use std::collections::HashMap;
use std::num::NonZero;
use chrono::{NaiveDate, Weekday};
use chrono_tz::Tz;
use eyre::{eyre, OptionExt};
use log::warn;
use serde::{de, Deserialize, Deserializer};
use serde::de::IntoDeserializer;
use crate::class::Class;
use crate::ical::vcalendar::VCalendar;
use crate::ical::vevent::VEvent;

fn deserialize_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, "%Y-%m-%d")
        .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&s), &"expected YYYY-MM-DD"))
}

fn deserialize_weeks<'de, D>(deserializer: D) -> Result<Vec<[NaiveDate; 5]>, D::Error>
where
    D: Deserializer<'de>
{
    let raw_weeks: Vec<Vec<String>> = Vec::deserialize(deserializer)?;

    raw_weeks
        .into_iter()
        .map(|week| {
            if week.len() != 5 {
                return Err(de::Error::invalid_length(week.len(), &"expected 5 dates per week"));
            }
            let mut arr = [NaiveDate::from_ymd_opt(1970,1,1).unwrap(); 5]; // temporary init
            for (i, s) in week.into_iter().enumerate() {
                arr[i] = deserialize_date(s.into_deserializer())?;
            }
            Ok(arr)
        })
        .collect()
}

#[derive(Debug, Deserialize)]
pub struct Subject {
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct Teacher {
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct Schedule {
    #[serde(deserialize_with = "deserialize_weeks")]
    weeks: Vec<[NaiveDate; 5]>,
    subjects: HashMap<String, Subject>,
    teachers: HashMap<String, Teacher>,
    schedule: Vec<Class>,
}

pub fn schedule_to_ical(schedule: &Schedule, tz: &Tz) -> eyre::Result<VCalendar> {
    let mut events = Vec::new();

    for (week_index, week) in schedule.weeks.iter().enumerate() {
        for (day_index, day) in week.iter().enumerate() {
            let scheduled_weekday = Weekday::try_from(day_index as u8)?;
            let real_weekday = day.weekday();

            if scheduled_weekday != real_weekday {
                warn!("Weekday mismatch on week {}, day {}: scheduled {}, real {}", week_index + 1, day_index + 1, scheduled_weekday, real_weekday);
            }

            for class in &schedule.schedule {
                if class.weeks.is_some()
                    && !class.weeks.as_ref().unwrap().happens_in_week(NonZero::new(week_index + 1)
                    .ok_or_eyre("week number would be zero")?) {
                    continue;
                }

                if class.day != scheduled_weekday {
                    continue;
                }

                let subject = schedule.subjects.get(&class.subject)
                    .ok_or_else(|| eyre!("subject not found: {}", class.subject))?;
                let teachers: Vec<String> = match &class.teachers {
                    Some(teacher_ids) => teacher_ids.iter()
                        .map(|id| {
                            schedule.teachers.get(id)
                                .map(|t| t.name.clone())
                                .ok_or_else(|| eyre!("teacher not found: {}", id))
                        })
                        .collect::<Result<_, _>>()?,
                    None => vec![],
                };
                let location = class.location.as_ref()
                    .map(|loc| format!("{}/{}", loc.room, loc.building));

                // TODO Better UID generation
                let uid = format!(
                    "{}-{}-{}-{}-{}",
                    class.class_type.to_name(),
                    class.subject.replace(" ", "_"),
                    class.day,
                    week_index,
                    class.time.start.format("%H%M"),
                );

                let now = Local::now().with_timezone(tz);
                let start = day.and_time(class.time.start).and_local_timezone(*tz).single()
                    .ok_or_eyre("ambiguous or non-existent start time")?;
                let end = day.and_time(class.time.end).and_local_timezone(*tz).single()
                    .ok_or_eyre("ambiguous or non-existent end time")?;



                let event = VEvent {
                    uid,
                    created: now,
                    start,
                    end,
                    summary: format!("{} {}", class.class_type.to_emoji(), subject.name),
                    description: format!(
                        "{}\n{}",
                        class.class_type.to_name(),
                        teachers.join("\n"),
                    ),
                    location: location.unwrap_or_default(),
                };

                events.push(event);
            }
        }
    }

    let cal = VCalendar {
        prod_id: "-//TheChilliPL//Planner//PL".to_string(),
        version: "2.0".to_string(),
        events,
    };

    Ok(cal)
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use super::*;

    #[test]
    fn deserialize_schedule() {
        let json = json!({
            "$schema": "./schedule.schema.json",
            "weeks": [
                ["2025-01-01", "2025-01-02", "2025-01-03", "2025-01-04", "2025-01-05"]
            ],
            "subjects": {
                "subj": {
                    "name": "Subject"
                }
            },
            "teachers": {
                "teacher1": {
                    "name": "Teacher"
                }
            },
            "schedule": []
        });

        let schedule: Schedule = serde_json::from_value(json).unwrap();

        assert_eq!(schedule.weeks.len(), 1);
        assert_eq!(schedule.weeks[0][0], NaiveDate::from_ymd_opt(2025,01,01).unwrap());
        assert_eq!(schedule.subjects.len(), 1);
        assert_eq!(schedule.subjects.get("subj").unwrap().name, "Subject");
        assert_eq!(schedule.teachers.get("teacher1").unwrap().name, "Teacher");
        assert_eq!(schedule.schedule.len(), 0);
    }
}
