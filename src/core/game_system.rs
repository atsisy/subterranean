use super::*;

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum GoingOutEvent {
    AkyuTei,
    Dangoya,
    Terakoya,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DayWorkType {
    ShopWork,
    GoingOut(GoingOutEvent),
    TakingRest,
}

impl DayWorkType {
    pub fn to_string_jp(&self) -> String {
        match self {
            DayWorkType::ShopWork => "店番",
            DayWorkType::GoingOut(dest) => match dest {
                GoingOutEvent::AkyuTei => "外出",
                GoingOutEvent::Dangoya => "外出",
                GoingOutEvent::Terakoya => "外出",
            },
            DayWorkType::TakingRest => "休憩",
        }
        .to_string()
    }
}

pub struct EventProgressTable {}

#[derive(Clone, Serialize, Deserialize)]
pub struct WeekWorkSchedule {
    first_day: GensoDate,
    schedule: [Option<DayWorkType>; 7],
}

impl WeekWorkSchedule {
    pub fn new(first_day: GensoDate, schedule: [DayWorkType; 7]) -> Self {
        WeekWorkSchedule {
            first_day: first_day,
            schedule: [
                Some(schedule[0]),
                Some(schedule[1]),
                Some(schedule[2]),
                Some(schedule[3]),
                Some(schedule[4]),
                Some(schedule[5]),
                Some(schedule[6]),
            ],
        }
    }

    pub fn new_empty(first_day: GensoDate) -> Self {
        WeekWorkSchedule {
            first_day: first_day,
            schedule: [None, None, None, None, None, None, None],
        }
    }

    pub fn get_first_day(&self) -> GensoDate {
        self.first_day.clone()
    }

    pub fn get_schedule_at(&self, index: usize) -> Option<DayWorkType> {
        if index >= 7 {
	    None
        } else {
	    self.schedule[index].clone()
	}
    }

    pub fn update_is_not_required(&self, date: &GensoDate) -> bool {
        let diff = self.first_day.diff_day(date);
        diff < 7 && diff >= 0 && !self.schedule.contains(&None)
    }

    pub fn get_schedule_of(&self, day: &GensoDate) -> Option<DayWorkType> {
        let diff = self.first_day.diff_day(day);
        if diff < 0 {
            return None;
        }

        self.get_schedule_at(diff as usize)
    }
}
