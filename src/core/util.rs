pub fn year_to_season(year: i64) -> i64 {
    year
}

pub fn random_select<T>(mut i: std::slice::Iter<T>) -> Option<&T> {
    i.nth(rand::random::<usize>() % i.len())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DayOfWeek {
    Sunday,
    Monday,
    TuesDay,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl DayOfWeek {
    pub fn from_char(day_str: &str) -> Self {
        match day_str {
            "Sun" => Self::Sunday,
            "Mon" => Self::Monday,
            "Tue" => Self::TuesDay,
            "Wed" => Self::Wednesday,
            "Thu" => Self::Thursday,
            "Fri" => Self::Friday,
            "Sat" => Self::Saturday,
            _ => Self::Sunday,
        }
    }

    pub fn from_char_jp(day_str: &str) -> Self {
        match day_str {
            "日曜日" => Self::Sunday,
            "月曜日" => Self::Monday,
            "火曜日" => Self::TuesDay,
            "水曜日" => Self::Wednesday,
            "木曜日" => Self::Thursday,
            "金曜日" => Self::Friday,
            "土曜日" => Self::Saturday,
            _ => Self::Sunday,
        }
    }
}

impl ToString for DayOfWeek {
    fn to_string(&self) -> String {
        match self {
            Self::Sunday => "日曜日",
            Self::Monday => "月曜日",
            Self::TuesDay => "火曜日",
            Self::Wednesday => "水曜日",
            Self::Thursday => "木曜日",
            Self::Friday => "金曜日",
            Self::Saturday => "土曜日",
        }
        .to_string()
    }
}

#[macro_export]
macro_rules! perf_measure {
    ( $x:expr) => {{
        let start = std::time::Instant::now();
        let result = $x;
        let end = start.elapsed();
        end.subsec_nanos()
    }};
}
