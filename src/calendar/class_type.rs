use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClassType {
    Lecture,
    Lab,
    Exercise,
    Seminar,
    PE,
    Languages,
    Project,
}

impl ClassType {
    pub fn to_name(self) -> &'static str {
        match self {
            ClassType::Lecture => "Wykład",
            ClassType::Lab => "Laboratorium",
            ClassType::Exercise => "Ćwiczenia",
            ClassType::Seminar => "Seminarium",
            ClassType::PE => "Wychowanie Fizyczne",
            ClassType::Languages => "Lektorat",
            ClassType::Project => "Projekt",
        }
    }

    pub fn to_emoji(self) -> &'static str {
        match self {
            ClassType::Lecture => "📚",
            ClassType::Lab => "🧪",
            ClassType::Exercise => "🏋️",
            ClassType::Seminar => "📝",
            ClassType::PE => "🏃",
            ClassType::Languages => "🗣️",
            ClassType::Project => "🛠️",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let json = "\"lecture\"";

        assert_eq!(serde_json::from_str::<ClassType>(json).unwrap(), ClassType::Lecture);
    }
}
