use std::collections::HashMap;

use crate::parse_toml_file;

use super::{GameResource, GensoDate};
use super::{BookInformation, SuzuContext};

#[derive(Clone)]
pub struct DayNewBooks {
    new_books: Vec<BookInformation>,
}

impl DayNewBooks {
    pub fn from_toml_value(toml_value: &toml::Value) -> Self {
        let mut new_books = Vec::new();

        for books_information in toml_value.as_array().unwrap() {
            let struct_table = books_information.as_table().unwrap();

            let book_num = struct_table["num"].as_integer().unwrap();

            let book_info = BookInformation::new(
                struct_table["name"].as_str().unwrap().to_string(),
                struct_table["pages"].as_integer().unwrap() as usize,
                struct_table["size"].as_str().unwrap().to_string(),
                struct_table["billing_number"].as_integer().unwrap() as u16,
                struct_table["base_price"].as_integer().unwrap() as u32,
            );

            for _ in 0..book_num {
                new_books.push(book_info.clone_with_new_id_condition());
            }
        }

        DayNewBooks {
            new_books: new_books,
        }
    }

    pub fn get_new_books(&self) -> Vec<BookInformation> {
        self.new_books.clone()
    }

    pub fn random(resource: &GameResource, num: usize, unit: usize) -> Self {
	let mut books = Vec::new();
	for _ in 0..num {
	    let info = resource.book_random_select();
	    for _ in 0..unit {
		books.push(info.clone_with_new_id_condition());
	    }
	}

	DayNewBooks {
	    new_books: books
	}
    }
}

pub struct NewBookSchedule {
    new_book_schedule: HashMap<GensoDate, DayNewBooks>,
}

impl NewBookSchedule {
    pub fn from_toml<'a>(ctx: &mut SuzuContext<'a>, file_path: &str) -> Self {
        let mut schedule_map = HashMap::new();

        let root = parse_toml_file!(ctx.context, file_path);

        let new_book_schedule = root["new-book-schedule"].as_array().unwrap();

        for day_new_book in new_book_schedule {
            let struct_table = day_new_book.as_table().unwrap();

            let date_data = struct_table["date"].as_table().unwrap();

            let genso_date = GensoDate::new(
                date_data["season"].as_integer().unwrap() as u32,
                date_data["month"].as_integer().unwrap() as u8,
                date_data["day"].as_integer().unwrap() as u8,
            );

            let day_new_books = DayNewBooks::from_toml_value(&struct_table["books_information"]);

            schedule_map.insert(genso_date, day_new_books);
        }

        NewBookSchedule {
            new_book_schedule: schedule_map,
        }
    }

    pub fn get_schedule_at(&self, date: &GensoDate) -> Option<&DayNewBooks> {
        self.new_book_schedule.get(date)
    }
}
