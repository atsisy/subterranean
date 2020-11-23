use super::*;

#[derive(Clone)]
pub enum GoingOutEvent {
    AkyuTei,
    Dangoya,
    Terakoya,
}

#[derive(Clone)]
pub enum DayWorkType {
    ShopWork,
    GoingOut(GoingOutEvent),
    TakingRest,
}

impl DayWorkType {
    pub fn to_string_jp(&self) -> String {
	match self {
	    DayWorkType::ShopWork => "店番",
	    DayWorkType::GoingOut(dest) => {
		match dest {
		    GoingOutEvent::AkyuTei => "外出（阿求亭）",
		    GoingOutEvent::Dangoya => "外出（団子屋）",
		    GoingOutEvent::Terakoya => "外出（寺子屋）",
		}
	    },
	    DayWorkType::TakingRest => "休憩"
	}.to_string()
    }
}

pub struct EventProgressTable {
    
}

#[derive(Clone)]
pub struct DayWorkSchedule {
    work_type: DayWorkType,
}

pub struct WeekWorkSchedule {
    first_day: GensoDate,
    schedule: [DayWorkSchedule; 7],
}

impl WeekWorkSchedule {
    pub fn get_first_day(&self) -> GensoDate {
	self.first_day.clone()
    }

    pub fn get_schedule_at(&self, index: usize) -> DayWorkSchedule {
	if index >= 7 {
	    panic!("invalid index, greater or equal to 7");
	}
	self.schedule[index].clone()
    }
}
