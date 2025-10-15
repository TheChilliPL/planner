pub mod class;
pub mod class_type;
pub mod periods;
pub mod schedule;

use serde::Deserialize;
use std::num::NonZero;

#[derive(Default, Deserialize, Eq, PartialEq, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum WeekParity {
    #[default]
    All,
    Odd,
    Even,
}

impl WeekParity {
    pub fn of_week(week: NonZero<usize>) -> WeekParity {
        let mod2 = (week.get() - 1) % 2;

        match mod2 {
            0 => WeekParity::Odd,
            1 => WeekParity::Even,
            _ => panic!("usize mod 2 didn't return 0 nor 1?"),
        }
    }

    pub fn includes(self, week: NonZero<usize>) -> bool {
        if self == WeekParity::All {
            return true;
        }

        self == WeekParity::of_week(week)
    }
}

#[derive(Default, Deserialize, PartialEq, Eq, Debug)]
pub struct Weeks {
    from: Option<NonZero<usize>>,
    to: Option<NonZero<usize>>,
    #[serde(default)]
    parity: WeekParity,
}

impl Weeks {
    pub fn happens_in_week(&self, week: NonZero<usize>) -> bool {
        if self.from.is_some() && self.from.unwrap() > week {
            return false;
        }

        if self.to.is_some() && self.to.unwrap() < week {
            return false;
        }

        self.parity.includes(week)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_parity() {
        assert_eq!(
            serde_json::from_str::<WeekParity>("\"all\"").unwrap(),
            WeekParity::All
        );
        assert_eq!(
            serde_json::from_str::<WeekParity>("\"even\"").unwrap(),
            WeekParity::Even
        );
        assert_eq!(
            serde_json::from_str::<WeekParity>("\"odd\"").unwrap(),
            WeekParity::Odd
        );
    }

    #[test]
    fn deserialize_weeks() {
        let json1 = json!({
            "from": 5,
            "to": 10,
            "parity": "odd",
        });

        assert_eq!(
            serde_json::from_value::<Weeks>(json1).unwrap(),
            Weeks {
                from: Some(NonZero::new(5).unwrap()),
                to: Some(NonZero::new(10).unwrap()),
                parity: WeekParity::Odd,
            }
        );

        let json2 = json!({});

        assert_eq!(
            serde_json::from_value::<Weeks>(json2).unwrap(),
            Default::default()
        );
    }
}
