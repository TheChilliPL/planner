use std::num::NonZero;
use super::class_type::ClassType;
use crate::calendar::periods::NaiveTimePeriod;
use crate::calendar::Weeks;
use chrono::Weekday;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(remote = "Weekday")]
enum WeekdayDef {
    #[serde(rename = "monday")]
    Mon = 0,
    #[serde(rename = "tuesday")]
    Tue = 1,
    #[serde(rename = "wednesday")]
    Wed = 2,
    #[serde(rename = "thursday")]
    Thu = 3,
    #[serde(rename = "friday")]
    Fri = 4,
    #[serde(rename = "saturday")]
    Sat = 5,
    #[serde(rename = "sunday")]
    Sun = 6,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Location {
    pub building: String,
    pub room: String,
}

#[derive(Debug, Deserialize)]
pub struct Class {
    pub subject: String,
    #[serde(rename = "type")]
    pub class_type: ClassType,
    #[serde(with = "WeekdayDef")]
    pub day: Weekday,
    pub time: NaiveTimePeriod,
    pub location: Option<Location>,
    pub teachers: Option<Vec<String>>,
    pub weeks: Option<Weeks>,
}

impl Class {
    pub fn happens_on(&self, week_number: NonZero<usize>, weekday: Weekday) -> bool {
        if self.day != weekday {
            return false;
        }

        if let Some(weeks) = &self.weeks {
            weeks.happens_in_week(week_number)
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_class() {
        let json = json!({
            "subject": "subj",
            "type": "lecture",
            "day": "wednesday",
            "time": "9:30-11:00",
            "location": {
                "building": "A",
                "room": "123",
            },
            "teachers": ["teacher1"]
        });

        let class = serde_json::from_value::<Class>(json).unwrap();

        assert_eq!(class.subject, "subj");
        assert_eq!(class.class_type, ClassType::Lecture);
        assert_eq!(class.day, Weekday::Wed);
        assert_eq!(class.time, NaiveTimePeriod::from_hm_hm(9, 30, 11, 0));
        assert_eq!(
            class.location,
            Some(Location {
                building: "A".into(),
                room: "123".into()
            })
        );
        assert_eq!(class.teachers, Some(vec!["teacher1".into()]));
    }
}
