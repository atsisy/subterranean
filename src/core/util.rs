use std::cell::RefCell;
use torifune::numeric;

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

pub fn max<T>(a: T, b: T) -> T
where T: PartialOrd {
    if a > b {
	a
    } else {
	b
    }
}

pub fn min<T>(a: T, b: T) -> T
where T: PartialOrd {
    if a < b {
	a
    } else {
	b
    }
}

#[macro_export]
macro_rules! perf_measure {
    ( $x:expr) => {{
        let start = std::time::Instant::now();
        let _ = $x;
        let end = start.elapsed();
        end.subsec_nanos()
    }};
}

#[macro_export]
macro_rules! parse_toml_file {
    ( $path:expr) => {{
	let content = match std::fs::read_to_string($path) {
            Ok(c) => c,
            Err(_) => panic!("Failed to read: {}", $path),
        };

	content.parse::<toml::Value>().expect("Failed to parse toml file")
    }};
}

pub fn clock_needle_angle(hour: u8, minute: u8) -> (f32, f32) {
    let hour = hour % 12;

    let angle_per_hour = 2.0 * std::f32::consts::PI / (12.0 * 60.0);
    let angle_per_minute = 2.0 * std::f32::consts::PI / 60.0;
    
    (((hour as f32 * 60.0) + minute as f32) * angle_per_hour, minute as f32 * angle_per_minute)
}

pub fn clock_needle_angle_inverse(hour: u8, minute: u8) -> (f32, f32) {
    let mut t = clock_needle_angle(hour, minute);

    t.0 += std::f32::consts::PI;
    t.1 += std::f32::consts::PI;

    t
}

pub fn find_proper_window_position(window_rect: numeric::Rect, outer_rect: numeric::Rect) -> numeric::Point2f {
    let rect_pos = window_rect.point();
    let mut found_pos = numeric::Point2f::new(rect_pos.x, rect_pos.y - window_rect.h);
    let window_rect = numeric::Rect::new(window_rect.x, rect_pos.y - window_rect.h, window_rect.w, window_rect.h);
    
    if window_rect.right() > outer_rect.right() {
	found_pos.x -= window_rect.right() - outer_rect.right();
    } else if window_rect.left() < outer_rect.left() {
	found_pos.x += outer_rect.left() - window_rect.right();
    }

    if window_rect.bottom() > outer_rect.bottom() {
	found_pos.y -= window_rect.bottom() - outer_rect.bottom();
    } else if window_rect.top() < outer_rect.top() {
	found_pos.y += outer_rect.top() - window_rect.top();
    }

    found_pos.into()
}

thread_local! {
    static unique_id: RefCell<u64> = RefCell::new(0);
}

pub fn get_unique_id() -> u64 {
    unique_id.with(|id| {
        // インクリメント
        *id.borrow_mut() += 1;
	id.borrow().clone()
    })
}

pub fn random_point_in_rect(rect: numeric::Rect) -> numeric::Point2f {
    let begin_x = rect.left() as usize;
    let begin_y = rect.top() as usize;

    numeric::Point2f::new(
	(begin_x + rand::random::<usize>() % rect.w as usize) as f32,
	(begin_y + rand::random::<usize>() % rect.h as usize) as f32,
    )
}
