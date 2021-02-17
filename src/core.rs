pub mod book_management;
pub mod crypt;
pub mod game_system;
pub mod map_parser;
pub mod util;

use game_system::WeekWorkSchedule;
use ggez::graphics as ggraphics;
use ggez::*;

use tdev::ProgramableKey;
use torifune::core::Clock;
use torifune::debug;
use torifune::device as tdev;
use torifune::graphics::drawable::*;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::tile_batch::*;
use torifune::graphics::object::{FontInformation, TextureObject};
use torifune::hash;
use torifune::numeric;
use torifune::sound;

use ggez::input as ginput;
use ggez::input::keyboard::*;
use ginput::mouse::MouseButton;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::str::FromStr;

use crate::scene;
use crate::{
    object::{
        scenario_object::SuzunaAdAgencyType, task_object::tt_sub_component::BorrowingRecordBookData,
    },
    parse_toml_file,
};

use std::fs::File;
use std::io::{BufReader, Read, Write};

use serde::{Deserialize, Serialize};
extern crate serde_json;

use crate::object::scenario_object::SuzunaAdType;
use number_to_jk::number_to_jk;

extern crate num;

pub const WINDOW_SIZE_X: i16 = 1366;
pub const WINDOW_SIZE_Y: i16 = 768;

pub struct InitialDisplay {
    texture: Vec<ggraphics::Image>,
    index: usize,
}

impl InitialDisplay {
    pub fn new(ctx: &mut ggez::Context) -> Self {
        InitialDisplay {
            texture: vec![ggraphics::Image::new(ctx, "/textures/sumire_logo.png").unwrap()],
            index: 0,
        }
    }

    pub fn draw(&self, ctx: &mut ggez::Context) {
        ggraphics::clear(ctx, [0.0, 0.0, 0.0, 0.0].into());
        let texture = self.texture.get(self.index);

        ggraphics::draw(ctx, texture.unwrap(), ggraphics::DrawParam::default()).unwrap();

        ggraphics::present(ctx).unwrap();
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TextureID {
    Ghost1 = 0,
    LotusPink,
    LotusBlue,
    LotusYellow,
    TextBackground,
    Paper1,
    Paper2,
    LargeBook1,
    LargeBook2,
    LargeBook3,
    MiddleBook1,
    MiddleBook2,
    MiddleBook3,
    Wood1,
    WafuTexture1,
    WafuTexture2,
    Chobo1,
    ChoicePanel1,
    ChoicePanel2,
    ChoicePanel3,
    ChoicePanel4,
    ChoicePanel5,
    JunkoTachieDefault,
    SightBackground1,
    ArrowRight,
    ArrowLeft,
    KosuzuDotFront1,
    KosuzuDotFront2,
    KosuzuDotFront3,
    KosuzuDotBack1,
    KosuzuDotBack2,
    KosuzuDotBack3,
    KosuzuDotRight1,
    KosuzuDotRight2,
    KosuzuDotRight3,
    KosuzuDotLeft1,
    KosuzuDotLeft2,
    KosuzuDotLeft3,
    StoreButton,
    ResetButton,
    MenuArt1,
    MenuArt2,
    JpHouseTexture,
    BookBoxFront,
    BookBoxBack,
    Paper3,
    Paper4,
    Paper5,
    Paper6,
    Paper7,
    Clock1,
    ClockNeedle1,
    ShortClockNeedle1,
    Hanko,
    NextLineIcon,
    LargeBookScratchFair1,
    LargeBookScratchFair2,
    LargeBookScratchFair3,
    LargeBookScratchFair4,
    LargeBookScratchBad1,
    LargeBookScratchBad2,
    LargeBookScratchBad3,
    LargeBookScratchBad4,
    ManualPageBookTitles,
    ManualPageBorrowingFlow,
    ManualPageReturnFlow,
    GoNextPageLeft,
    GoNextPageRight,
    Library,
    KosuzuTachie1,
    CheckCircle,
    MoneyBox,
    KosuzuSmile1,
    Coin100Yen,
    Coin50Yen,
    Coin500Yen,
    BaraBG,
    SuzunaanMap,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum FontID {
    DEFAULT = 0,
    JpFude1,
    CorpMincho,
    Cinema,
    BitMap1,
}

impl FromStr for FontID {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "Default" => Ok(FontID::DEFAULT),
            "JpFude1" => Ok(FontID::JpFude1),
            "CorpMincho" => Ok(FontID::CorpMincho),
            "Cinema" => Ok(FontID::Cinema),
            "BitMap1" => Ok(FontID::BitMap1),
            _ => Err(()),
        }
    }
}

impl FromStr for TextureID {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "Ghost1" => Ok(Self::Ghost1),
            "LotusPink" => Ok(Self::LotusPink),
            "LotusBlue" => Ok(Self::LotusBlue),
            "LotusYellow" => Ok(Self::LotusYellow),
            "TextBackground" => Ok(Self::TextBackground),
            "Paper1" => Ok(Self::Paper1),
            "Paper2" => Ok(Self::Paper2),
            "LargeBook1" => Ok(Self::LargeBook1),
            "LargeBook2" => Ok(Self::LargeBook2),
            "LargeBook3" => Ok(Self::LargeBook3),
            "MiddleBook1" => Ok(Self::MiddleBook1),
            "MiddleBook2" => Ok(Self::MiddleBook2),
            "MiddleBook3" => Ok(Self::MiddleBook3),
            "Wood1" => Ok(Self::Wood1),
            "WafuTexture1" => Ok(Self::WafuTexture1),
            "WafuTexture2" => Ok(Self::WafuTexture2),
            "Chobo1" => Ok(Self::Chobo1),
            "ChoicePanel1" => Ok(Self::ChoicePanel1),
            "ChoicePanel2" => Ok(Self::ChoicePanel2),
            "ChoicePanel3" => Ok(Self::ChoicePanel3),
            "ChoicePanel4" => Ok(Self::ChoicePanel4),
            "ChoicePanel5" => Ok(Self::ChoicePanel5),
            "JunkoTachieDefault" => Ok(Self::JunkoTachieDefault),
            "SightBackground1" => Ok(Self::SightBackground1),
            "ArrowRight" => Ok(Self::ArrowRight),
            "ArrowLeft" => Ok(Self::ArrowLeft),
            "KosuzuDotFront1" => Ok(Self::KosuzuDotFront1),
            "KosuzuDotFront2" => Ok(Self::KosuzuDotFront2),
            "KosuzuDotFront3" => Ok(Self::KosuzuDotFront3),
            "KosuzuDotBack1" => Ok(Self::KosuzuDotBack1),
            "KosuzuDotBack2" => Ok(Self::KosuzuDotBack2),
            "KosuzuDotBack3" => Ok(Self::KosuzuDotBack3),
            "KosuzuDotRight1" => Ok(Self::KosuzuDotRight1),
            "KosuzuDotRight2" => Ok(Self::KosuzuDotRight2),
            "KosuzuDotRight3" => Ok(Self::KosuzuDotRight3),
            "KosuzuDotLeft1" => Ok(Self::KosuzuDotLeft1),
            "KosuzuDotLeft2" => Ok(Self::KosuzuDotLeft2),
            "KosuzuDotLeft3" => Ok(Self::KosuzuDotLeft3),
            "StoreButton" => Ok(Self::StoreButton),
            "ResetButton" => Ok(Self::ResetButton),
            "MenuArt1" => Ok(Self::MenuArt1),
            "MenuArt2" => Ok(Self::MenuArt2),
            "JpHouseTexture" => Ok(Self::JpHouseTexture),
            "BookBoxFront" => Ok(Self::BookBoxFront),
            "BookBoxBack" => Ok(Self::BookBoxBack),
            "Paper3" => Ok(Self::Paper3),
            "Paper4" => Ok(Self::Paper4),
            "Paper5" => Ok(Self::Paper5),
            "Paper6" => Ok(Self::Paper6),
            "Paper7" => Ok(Self::Paper7),
            "Clock1" => Ok(Self::Clock1),
            "ClockNeedle1" => Ok(Self::ClockNeedle1),
            "ShortClockNeedle1" => Ok(Self::ShortClockNeedle1),
            "Hanko" => Ok(Self::Hanko),
            "NextLineIcon" => Ok(Self::NextLineIcon),
            "LargeBookScratchFair1" => Ok(Self::LargeBookScratchFair1),
            "LargeBookScratchFair2" => Ok(Self::LargeBookScratchFair2),
            "LargeBookScratchFair3" => Ok(Self::LargeBookScratchFair3),
            "LargeBookScratchFair4" => Ok(Self::LargeBookScratchFair4),
            "LargeBookScratchBad1" => Ok(Self::LargeBookScratchBad1),
            "LargeBookScratchBad2" => Ok(Self::LargeBookScratchBad2),
            "LargeBookScratchBad3" => Ok(Self::LargeBookScratchBad3),
            "LargeBookScratchBad4" => Ok(Self::LargeBookScratchBad4),
            "ManualPageBookTitles" => Ok(Self::ManualPageBookTitles),
            "ManualPageBorrowingFlow" => Ok(Self::ManualPageBorrowingFlow),
            "ManualPageReturnFlow" => Ok(Self::ManualPageReturnFlow),
            "GoNextPageLeft" => Ok(Self::GoNextPageLeft),
            "GoNextPageRight" => Ok(Self::GoNextPageRight),
            "Library" => Ok(Self::Library),
            "KosuzuTachie1" => Ok(Self::KosuzuTachie1),
            "CheckCircle" => Ok(Self::CheckCircle),
            "MoneyBox" => Ok(Self::MoneyBox),
            "KosuzuSmile1" => Ok(Self::KosuzuSmile1),
            "Coin100Yen" => Ok(Self::Coin100Yen),
            "Coin50Yen" => Ok(Self::Coin50Yen),
            "Coin500Yen" => Ok(Self::Coin500Yen),
	    "BaraBG" => Ok(Self::BaraBG),
	    "SuzunaanMap" => Ok(Self::SuzunaanMap),
            _ => Err(()),
        }
    }
}

impl TextureID {
    pub fn from_u32(n: u32) -> Option<Self> {
        match n {
            0 => Some(Self::Ghost1),
            1 => Some(Self::LotusPink),
            2 => Some(Self::LotusBlue),
            3 => Some(Self::LotusYellow),
            4 => Some(Self::TextBackground),
            5 => Some(Self::Paper1),
            6 => Some(Self::Paper2),
            7 => Some(Self::LargeBook1),
            8 => Some(Self::LargeBook2),
            9 => Some(Self::LargeBook3),
            10 => Some(Self::MiddleBook1),
            11 => Some(Self::MiddleBook2),
            12 => Some(Self::MiddleBook3),
            13 => Some(Self::Wood1),
            14 => Some(Self::WafuTexture1),
            15 => Some(Self::WafuTexture2),
            16 => Some(Self::Chobo1),
            17 => Some(Self::ChoicePanel1),
            18 => Some(Self::ChoicePanel2),
            19 => Some(Self::ChoicePanel3),
            20 => Some(Self::ChoicePanel4),
            21 => Some(Self::ChoicePanel5),
            22 => Some(Self::JunkoTachieDefault),
            23 => Some(Self::SightBackground1),
            24 => Some(Self::ArrowRight),
            25 => Some(Self::ArrowLeft),
            26 => Some(Self::KosuzuDotFront1),
            27 => Some(Self::KosuzuDotFront2),
            28 => Some(Self::KosuzuDotFront3),
            29 => Some(Self::KosuzuDotBack1),
            30 => Some(Self::KosuzuDotBack2),
            31 => Some(Self::KosuzuDotBack3),
            32 => Some(Self::KosuzuDotRight1),
            33 => Some(Self::KosuzuDotRight2),
            34 => Some(Self::KosuzuDotRight3),
            35 => Some(Self::KosuzuDotLeft1),
            36 => Some(Self::KosuzuDotLeft2),
            37 => Some(Self::KosuzuDotLeft3),
            38 => Some(Self::StoreButton),
            39 => Some(Self::ResetButton),
            40 => Some(Self::MenuArt1),
            41 => Some(Self::MenuArt2),
            42 => Some(Self::JpHouseTexture),
            43 => Some(Self::BookBoxFront),
            44 => Some(Self::BookBoxBack),
            45 => Some(Self::Paper3),
            46 => Some(Self::Paper4),
            47 => Some(Self::Paper5),
            48 => Some(Self::Paper6),
            49 => Some(Self::Paper7),
            50 => Some(Self::Clock1),
            51 => Some(Self::ClockNeedle1),
            52 => Some(Self::ShortClockNeedle1),
            53 => Some(Self::Hanko),
            54 => Some(Self::NextLineIcon),
            55 => Some(Self::LargeBookScratchFair1),
            56 => Some(Self::LargeBookScratchFair2),
            57 => Some(Self::LargeBookScratchFair3),
            58 => Some(Self::LargeBookScratchFair4),
            59 => Some(Self::LargeBookScratchBad1),
            60 => Some(Self::LargeBookScratchBad2),
            61 => Some(Self::LargeBookScratchBad3),
            62 => Some(Self::LargeBookScratchBad4),
            63 => Some(Self::ManualPageBookTitles),
            64 => Some(Self::ManualPageBorrowingFlow),
            65 => Some(Self::ManualPageReturnFlow),
            66 => Some(Self::GoNextPageLeft),
            67 => Some(Self::GoNextPageRight),
            68 => Some(Self::Library),
            69 => Some(Self::KosuzuTachie1),
            70 => Some(Self::CheckCircle),
            71 => Some(Self::MoneyBox),
            72 => Some(Self::KosuzuSmile1),
            73 => Some(Self::Coin100Yen),
            74 => Some(Self::Coin50Yen),
            75 => Some(Self::Coin500Yen),
	    76 => Some(Self::BaraBG),
	    77 => Some(Self::SuzunaanMap),
            _ => None,
        }
    }
}

impl TextureID {
    pub fn select_random() -> Self {
        TextureID::from_u32(rand::random::<u32>() % (Self::Unknown as u32)).unwrap()
    }

    pub fn random_large_book_scratch_fair() -> TextureID {
        let candidate = [
            TextureID::LargeBookScratchFair1,
            TextureID::LargeBookScratchFair2,
            TextureID::LargeBookScratchFair3,
            TextureID::LargeBookScratchFair4,
        ];

        util::random_select(candidate.iter()).unwrap().clone()
    }

    pub fn random_large_book_scratch_bad() -> TextureID {
        let candidate = [
            TextureID::LargeBookScratchBad1,
            TextureID::LargeBookScratchBad2,
            TextureID::LargeBookScratchBad3,
            TextureID::LargeBookScratchBad4,
        ];

        util::random_select(candidate.iter()).unwrap().clone()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TileBatchTextureID {
    OldStyleFrame,
    RedOldStyleFrame,
    TaishoStyle1,
    Suzu1,
    Shoji,
    BlackFrame,
    BlackFrame2,
}

pub const LARGE_BOOK_TEXTURE: [TextureID; 3] = [
    TextureID::LargeBook1,
    TextureID::LargeBook2,
    TextureID::LargeBook3,
];

pub const MIDDLE_BOOK_TEXTURE: [TextureID; 3] = [
    TextureID::MiddleBook1,
    TextureID::MiddleBook2,
    TextureID::MiddleBook3,
];

#[derive(Clone)]
pub enum SoundID {
    Title = 0,
    SeTurnThePage,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BookCondition {
    Good,
    Fair,
    Bad,
}

impl From<i32> for BookCondition {
    fn from(integer: i32) -> Self {
        match integer {
            0 => BookCondition::Good,
            1 => BookCondition::Fair,
            2 => BookCondition::Bad,
            _ => panic!("Not reserved number"),
        }
    }
}

impl BookCondition {
    fn from_u32(value: u32) -> Self {
        match value {
            0 => Self::Good,
            1 => Self::Fair,
            2 => Self::Bad,
            _ => panic!(""),
        }
    }

    pub fn probability_random(pb: &[u8]) -> Self {
        let mut random = rand::random::<usize>() % 100;

        for (index, p) in pb.iter().enumerate() {
            if random < *p as usize {
                return Self::from_u32(index as u32);
            }

            random -= *p as usize;
        }

        panic!("The summation of pb is over 100");
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Good => "良",
            Self::Fair => "可",
            Self::Bad => "悪",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookInformation {
    pub name: String,
    pub pages: usize,
    pub size: String,
    pub billing_number: u16,
    pub base_price: u32,
    condition: BookCondition,
    unique_id: u64,
}

impl BookInformation {
    pub fn new(
        name: String,
        pages: usize,
        size: String,
        billing_number: u16,
        base_price: u32,
    ) -> Self {
        BookInformation {
            name: name,
            pages: pages,
            size: size,
            billing_number: billing_number,
            base_price: base_price,
            condition: BookCondition::probability_random(&[70, 20, 10]),
            unique_id: util::get_unique_id(),
        }
    }

    pub fn clone_with_new_id_condition(&self) -> Self {
        let mut cloned = self.clone();

        cloned.condition = BookCondition::probability_random(&[70, 20, 10]);
        cloned.unique_id = util::get_unique_id();

        return cloned;
    }

    pub fn get_condition_string(&self) -> String {
        self.condition.to_string()
    }

    pub fn get_condition(&self) -> BookCondition {
        self.condition.clone()
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_pages(&self) -> usize {
        self.pages
    }

    pub fn get_unique_id(&self) -> u64 {
        self.unique_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RentalLimit {
    ShortTerm = 0,
    LongTerm,
    Today,
}

impl RentalLimit {
    pub fn random() -> RentalLimit {
        match rand::random::<u32>() % 2 {
            0 => RentalLimit::ShortTerm,
            1 => RentalLimit::LongTerm,
            _ => panic!("Exception"),
        }
    }

    pub fn fee_rate(&self) -> f32 {
        match self {
            RentalLimit::ShortTerm => 1.0,
            RentalLimit::LongTerm => 1.5,
            RentalLimit::Today => 0.0,
        }
    }

    pub fn to_str(&self) -> &str {
	match self {
	    RentalLimit::ShortTerm => "短期",
	    RentalLimit::LongTerm => "長期",
	    RentalLimit::Today => "本日",
	}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GensoDate {
    pub season: u32,
    pub month: u8,
    pub day: u8,
}

impl GensoDate {
    pub fn new(season: u32, month: u8, day: u8) -> Self {
        GensoDate {
            season: season,
            month: month,
            day: day,
        }
    }

    pub fn new_empty() -> Self {
        GensoDate {
            season: 0,
            month: 0,
            day: 0,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}季 {}月 {}日",
            number_to_jk(self.season as u64),
            number_to_jk(self.month as u64),
            number_to_jk(self.day as u64)
        )
    }

    pub fn to_short_string(&self) -> String {
        format!(
            "{}月{}日",
            number_to_jk(self.month as u64),
            number_to_jk(self.day as u64)
        )
    }

    pub fn to_month_string_eng_short(&self) -> String {
        match self.month {
            1 => "Jan.",
            2 => "Feb.",
            3 => "Mar.",
            4 => "Apr.",
            5 => "May",
            6 => "Jun.",
            7 => "Jul.",
            8 => "Aug.",
            9 => "Sep.",
            10 => "Oct.",
            11 => "Nov.",
            12 => "Dec.",
            _ => panic!("Invalid month"),
        }
        .to_string()
    }

    pub fn add_day_chain(mut self, day: i32) -> Self {
	self.add_day(day);
	self
    }

    pub fn add_day(&mut self, mut day: i32) {
        static MONTH: [i32; 12] = [31, 28, 31, 30, 30, 30, 31, 31, 30, 31, 30, 31];

        while self.day as i32 + day > MONTH[self.month as usize] {
            day -= MONTH[self.month as usize] - self.day as i32;
            self.day = 0;
            self.month += 1;

            if self.month > 12 {
                self.season += 1;
                self.month %= 12;
            }
        }

        self.day += day as u8;
    }

    ///
    /// self -> 7/1
    /// date2 -> 7/8
    /// return 7
    ///
    pub fn diff_day(&self, date2: &Self) -> i32 {
        static MONTH: [i32; 12] = [31, 28, 31, 30, 30, 30, 31, 31, 30, 31, 30, 31];

        let greater_self = self.month.partial_cmp(&date2.month).unwrap();

        match greater_self {
            std::cmp::Ordering::Less => {
                let mut diff = MONTH[self.month as usize] - self.day as i32;
                for month_index in (self.month + 1)..date2.month {
                    diff += MONTH[month_index as usize];
                }
                diff + date2.day as i32
            }
            std::cmp::Ordering::Equal => {
                let diff = if self.day > date2.day {
                    self.day - date2.day
                } else {
                    date2.day - self.day
                };

                diff as i32
            }
            std::cmp::Ordering::Greater => {
                let mut diff = MONTH[date2.month as usize] - date2.day as i32;
                for month_index in (date2.month + 1)..self.month {
                    diff += MONTH[month_index as usize];
                }
                -(diff + date2.day as i32)
            }
        }
    }

    pub fn rental_limit_type(&self, limit: &GensoDate) -> Option<RentalLimit> {
        let day_diff = self.diff_day(&limit);

        if day_diff >= 0 {
            if day_diff == 0 {
                Some(RentalLimit::Today)
            } else if day_diff == 7 {
                Some(RentalLimit::ShortTerm)
            } else if day_diff == 14 {
                Some(RentalLimit::LongTerm)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn is_past(&self, date: &GensoDate) -> bool {
	match self.season.cmp(&date.season) {
	    std::cmp::Ordering::Less => false,
	    std::cmp::Ordering::Greater => true,
	    std::cmp::Ordering::Equal => match self.month.cmp(&date.month) {
		std::cmp::Ordering::Less => false,
		std::cmp::Ordering::Greater => true,
		std::cmp::Ordering::Equal => match self.day.cmp(&date.day) {
		    std::cmp::Ordering::Less | std::cmp::Ordering::Equal => false,
		    std::cmp::Ordering::Greater => true,
		}
	    }
	}
    }

    pub fn is_week_first(&self) -> bool {
	let diff = self.diff_day(&GensoDate::new(112, 7, 23));
	diff % 7 == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GeneralScenarioID {
    NoEnoughMoney,
}

impl GeneralScenarioID {
    pub fn from_str(s: &str) -> Self {
        match s {
            "NoEnoughMoney" => Self::NoEnoughMoney,
            _ => panic!("Invalid General Scenario String"),
        }
    }
}

pub struct ScenarioTable {
    scenario_table: HashMap<GensoDate, String>,
    general_scenario: HashMap<GeneralScenarioID, String>,
}

impl ScenarioTable {
    pub fn new(ctx: &mut ggez::Context, table_toml_path: &str) -> Self {
        let mut table = HashMap::new();

        let root = parse_toml_file!(ctx, table_toml_path);
        let array = root["scenario-table"].as_array().unwrap();

        for elem in array {
            let date_data = elem.get("date").unwrap().as_table().unwrap();
            let genso_date = GensoDate::new(
                date_data.get("season").unwrap().as_integer().unwrap() as u32,
                date_data.get("month").unwrap().as_integer().unwrap() as u8,
                date_data.get("day").unwrap().as_integer().unwrap() as u8,
            );

            let path = elem.get("path").unwrap().as_str().unwrap();

            table.insert(genso_date, path.to_string());
        }

        let mut general_scenario = HashMap::new();
        let array = root["general-scenario-table"].as_array().unwrap();

        for elem in array {
            let ty = elem.get("type").unwrap().as_str().unwrap();
            let id = GeneralScenarioID::from_str(ty);
            let path = elem.get("path").unwrap().as_str().unwrap();

            general_scenario.insert(id, path.to_string());
        }

        ScenarioTable {
            scenario_table: table,
            general_scenario: general_scenario,
        }
    }

    pub fn get_day_scenario_path(&self, date: &GensoDate) -> Option<String> {
        if let Some(s) = self.scenario_table.get(&date) {
            Some(s.to_string())
        } else {
            println!("Error: Invalid Date => {:?}", date);
            None
        }
    }

    pub fn get_general_scenario_path(&self, id: &GeneralScenarioID) -> Option<String> {
        if let Some(s) = self.general_scenario.get(id) {
            Some(s.to_string())
        } else {
            println!("Error: Invalid General Scenario ID => {:?}", id);
            None
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct BookShelfInformation {
    billing_number_begin: u16,
    billing_number_end: u16,
}

impl BookShelfInformation {
    pub fn new(begin: u16, end: u16) -> Self {
        BookShelfInformation {
            billing_number_begin: begin,
            billing_number_end: end,
        }
    }

    pub fn contains_number(&self, inquire_number: u16) -> bool {
        self.billing_number_begin <= inquire_number && inquire_number <= self.billing_number_end
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct MapConstractData {
    pub id: u32,
    pub comment: String,
    pub map_file_path: String,
    pub event_map_file_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SpriteBatchData {
    pub sprite_x_size: u16,
    pub sprite_y_size: u16,
    pub path: String,
}

#[derive(Deserialize, Serialize)]
pub struct AdCostTable {
    ad_cost_table: HashMap<crate::object::scenario_object::SuzunaAdType, u32>,
    ad_gain_table: HashMap<crate::object::scenario_object::SuzunaAdType, u32>,
}

impl AdCostTable {
    pub fn from_data(cost_data: HashMap<String, u32>, gain_data: HashMap<String, u32>) -> Self {
        let mut cost_table = HashMap::new();
        let mut gain_table = HashMap::new();

        for (s, c) in cost_data.iter() {
            cost_table.insert(
                crate::object::scenario_object::SuzunaAdType::from_str(s),
                *c,
            );
        }

        for (s, c) in gain_data.iter() {
            gain_table.insert(
                crate::object::scenario_object::SuzunaAdType::from_str(s),
                *c,
            );
        }

        AdCostTable {
            ad_cost_table: cost_table,
            ad_gain_table: gain_table,
        }
    }

    pub fn get_cost(&self, ty: crate::object::scenario_object::SuzunaAdType) -> u32 {
        *self.ad_cost_table.get(&ty).unwrap()
    }

    pub fn get_reputation_gain(&self, ty: crate::object::scenario_object::SuzunaAdType) -> u32 {
        *self.ad_gain_table.get(&ty).unwrap()
    }
}

#[derive(Deserialize, Serialize)]
pub struct AdAgencyCostTable {
    cost_table: HashMap<SuzunaAdAgencyType, u32>,
    money_gain_table: HashMap<SuzunaAdAgencyType, u32>,
}

impl AdAgencyCostTable {
    pub fn from_data(cost_data: HashMap<String, u32>, gain_data: HashMap<String, u32>) -> Self {
        let mut cost_table = HashMap::new();
        let mut money_gain_table = HashMap::new();

        for (s, c) in cost_data.iter() {
            cost_table.insert(
                crate::object::scenario_object::SuzunaAdAgencyType::from_str(s),
                *c,
            );
        }

        for (s, c) in gain_data.iter() {
            money_gain_table.insert(
                crate::object::scenario_object::SuzunaAdAgencyType::from_str(s),
                *c,
            );
        }

        AdAgencyCostTable {
            cost_table: cost_table,
            money_gain_table: money_gain_table,
        }
    }

    pub fn get_cost(&self, ty: &SuzunaAdAgencyType) -> u32 {
        *self.cost_table.get(ty).unwrap()
    }

    pub fn get_money_gain(&self, ty: &SuzunaAdAgencyType) -> u32 {
        *self.money_gain_table.get(ty).unwrap()
    }
}

#[derive(Deserialize)]
pub struct RawConfigFile {
    texture_paths: Vec<String>,
    font_paths: Vec<String>,
    customers_name: Vec<String>,
    books_information: Vec<BookInformation>,
    map_information: Vec<MapConstractData>,
    sprite_batch_information: Vec<SpriteBatchData>,
    scenario_table_path: String,
    sound_file_path: Vec<String>,
    ad_cost_table: HashMap<String, u32>,
    ad_gain_table: HashMap<String, u32>,
    ad_agency_cost_table: HashMap<String, u32>,
    ad_agency_gain_table: HashMap<String, u32>,
}

impl RawConfigFile {
    pub fn new(ctx: &mut ggez::Context, file_path: String) -> RawConfigFile {
        let s = util::read_from_resources_as_string(ctx, file_path.as_str());

        let raw_data: Result<RawConfigFile, toml::de::Error> = toml::from_str(&s);
        match raw_data {
            Ok(p) => p,
            Err(e) => panic!("Failed to parse toml: {}", e),
        }
    }
}

pub struct GameResource {

    texture_resource_paths: HashMap<TextureID, String>,
    textures: HashMap<TextureID, ggraphics::Image>,
    fonts: Vec<ggraphics::Font>,
    tile_batchs: Vec<TileBatch>,
    customers_name: Vec<String>,
    books_information: Vec<BookInformation>,
    map_data: Vec<MapConstractData>,
    scenario_table: ScenarioTable,
    sounds: Vec<sound::SoundData>,
    bgm_manager: sound::SoundManager,
    se_manager: sound::SoundManager,
    ad_info: AdCostTable,
    ad_agency_info: AdAgencyCostTable,
}

impl GameResource {
    pub fn new(ctx: &mut ggez::Context, file_path: String) -> GameResource {
        let init_display = InitialDisplay::new(ctx);
        init_display.draw(ctx);

        let src_file = RawConfigFile::new(ctx, file_path);

        let textures = HashMap::new();
        let mut fonts = Vec::new();
        let mut sprite_batchs = Vec::new();
        let mut sounds = Vec::new();
        let mut texture_paths_map = HashMap::new();

        print!("Setup textures delay loading ... ");
        for (index, texture_path) in src_file.texture_paths.iter().enumerate() {
            texture_paths_map.insert(
                TextureID::from_u32(index as u32).unwrap(),
                texture_path.clone(),
            );
        }
        println!("done");

        for font_path in &src_file.font_paths {
            print!("Loading font {}...", font_path);
            fonts.push(ggraphics::Font::new(ctx, font_path).unwrap());
            println!(" done!");
        }

        for sb_data in &src_file.sprite_batch_information {
            print!("Loading font {}...", sb_data.path);
            sprite_batchs.push(TileBatch::new(
                ggraphics::Image::new(ctx, &sb_data.path).unwrap(),
                numeric::Vector2u::new(sb_data.sprite_x_size as u32, sb_data.sprite_y_size as u32),
                numeric::Point2f::new(0.0, 0.0),
                0,
            ));
            println!(" done!");
        }

        for sound_path in &src_file.sound_file_path {
            let sound_data = sound::SoundData::new(ctx, sound_path).unwrap();
            println!(
                "sound path -> {}, canplay? => {:?}",
                sound_path,
                sound_data.can_play()
            );
            sounds.push(sound_data);
        }

        let scenario_table = ScenarioTable::new(ctx, &src_file.scenario_table_path);

        GameResource {
            texture_resource_paths: texture_paths_map,
            textures: textures,
            fonts: fonts,
	    
            tile_batchs: sprite_batchs,
            customers_name: src_file.customers_name,
            books_information: src_file.books_information,
            map_data: src_file.map_information,
            scenario_table: scenario_table,
            sounds: sounds,
            bgm_manager: sound::SoundManager::new(),
            se_manager: sound::SoundManager::new(),
            ad_info: AdCostTable::from_data(src_file.ad_cost_table, src_file.ad_gain_table),
            ad_agency_info: AdAgencyCostTable::from_data(
                src_file.ad_agency_cost_table,
                src_file.ad_agency_gain_table,
            ),
        }
    }

    fn load_texture_delay(&mut self, ctx: &mut ggez::Context, id: TextureID) -> ggraphics::Image {
        let path = self
            .texture_resource_paths
            .get(&id)
            .expect("Delay texture load: Invalid TextureID");
        print!("delay texture loading -> {} ... ", path);
        let texture = ggraphics::Image::new(ctx, path).expect("Delay texture load: Invalid Path");
        self.textures.insert(id, texture.clone());
        println!("done!");

        texture
    }

    pub fn ref_texture(&mut self, ctx: &mut ggez::Context, id: TextureID) -> ggraphics::Image {
        let maybe_texture = self.textures.get(&id);

        if let Some(texture) = maybe_texture {
            texture.clone()
        } else {
            self.load_texture_delay(ctx, id)
        }
    }

    pub fn get_font(&self, id: FontID) -> ggraphics::Font {
        match self.fonts.get(id as usize) {
            Some(font) => *font,
            None => panic!("Unknown Font ID: {}", id as i32),
        }
    }

    pub fn get_map_data(&self, _id: u32) -> Option<MapConstractData> {
        for map_data in &self.map_data {
            println!("FIXME!!");
            return Some(map_data.clone());
        }

        None
    }

    pub fn book_random_select(&self) -> &BookInformation {
        &self
            .books_information
            .get(rand::random::<usize>() % self.books_information.len())
            .unwrap()
    }

    pub fn search_book_with_title(&self, title: &str) -> Option<&BookInformation> {
        for book_info in self.books_information.iter() {
            if book_info.name == title {
                return Some(book_info);
            }
        }

        None
    }

    pub fn iter_available_books(&self) -> std::slice::Iter<BookInformation> {
        self.books_information.iter()
    }

    pub fn customer_random_select(&self) -> &str {
        &self
            .customers_name
            .get(rand::random::<usize>() % self.customers_name.len())
            .unwrap()
    }

    pub fn ref_tile_batch(&self, id: TileBatchTextureID) -> TileBatch {
        let maybe_tile_batch = self.tile_batchs.get(id as usize);

        if let Some(tile_batch) = maybe_tile_batch {
            tile_batch.clone()
        } else {
            panic!("Unknown TileBatchTexture ID: {}", id as i32)
        }
    }

    pub fn get_day_scenario_path(&self, date: &GensoDate) -> Option<String> {
        self.scenario_table.get_day_scenario_path(date)
    }

    pub fn get_general_scenario_path(&self, id: &GeneralScenarioID) -> Option<String> {
        self.scenario_table.get_general_scenario_path(id)
    }

    pub fn play_sound_as_bgm(
        &mut self,
        ctx: &mut ggez::Context,
        sound_id: SoundID,
        flags: Option<sound::SoundPlayFlags>,
    ) -> sound::SoundHandler {
        let sound_data = self.sounds.get(sound_id as usize).unwrap();
        self.bgm_manager.play(ctx, sound_data.clone(), flags)
    }

    pub fn play_sound_as_se(
        &mut self,
        ctx: &mut ggez::Context,
        sound_id: SoundID,
        flags: Option<sound::SoundPlayFlags>,
    ) -> sound::SoundHandler {
        let sound_data = self.sounds.get(sound_id as usize).unwrap();
        self.se_manager.play(ctx, sound_data.clone(), flags)
    }

    pub fn stop_bgm(&mut self, ctx: &mut ggez::Context, handler: sound::SoundHandler) {
        self.bgm_manager.stop(ctx, handler);
    }

    pub fn stop_se(&mut self, ctx: &mut ggez::Context, handler: sound::SoundHandler) {
        self.bgm_manager.stop(ctx, handler);
    }

    pub fn ref_bgm(&self, handler: sound::SoundHandler) -> &sound::PlayableSound {
        self.bgm_manager.ref_sound(handler)
    }

    pub fn ref_bgm_mut(&mut self, handler: sound::SoundHandler) -> &mut sound::PlayableSound {
        self.bgm_manager.ref_sound_mut(handler)
    }

    pub fn ref_se(&self, handler: sound::SoundHandler) -> &sound::PlayableSound {
        self.se_manager.ref_sound(handler)
    }

    pub fn ref_se_mut(&mut self, handler: sound::SoundHandler) -> &mut sound::PlayableSound {
        self.se_manager.ref_sound_mut(handler)
    }

    pub fn change_bgm_volume(&mut self, volume: f32) {
        self.bgm_manager.change_global_volume(volume);
    }

    pub fn change_se_volume(&mut self, volume: f32) {
        self.se_manager.change_global_volume(volume);
    }

    pub fn get_default_ad_cost(&self, ty: crate::object::scenario_object::SuzunaAdType) -> u32 {
        self.ad_info.get_cost(ty)
    }

    pub fn get_default_ad_reputation_gain(
        &self,
        ty: crate::object::scenario_object::SuzunaAdType,
    ) -> u32 {
        self.ad_info.get_reputation_gain(ty)
    }

    pub fn get_default_ad_agency_cost(&self, ty: &SuzunaAdAgencyType) -> u32 {
        self.ad_agency_info.get_cost(ty)
    }

    pub fn get_default_ad_agency_money_gain(&self, ty: &SuzunaAdAgencyType) -> u32 {
        self.ad_agency_info.get_money_gain(ty)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct MouseActionRecord {
    pub point: numeric::Point2f,
    pub t: Clock,
}

impl MouseActionRecord {
    fn new(point: numeric::Point2f, t: Clock) -> MouseActionRecord {
        MouseActionRecord { point: point, t: t }
    }

    fn new_empty() -> MouseActionRecord {
        MouseActionRecord {
            point: numeric::Point2f::new(0.0, 0.0),
            t: 0,
        }
    }
}

pub struct MouseInformation {
    pub last_clicked: HashMap<MouseButton, MouseActionRecord>,
    pub last_dragged: HashMap<MouseButton, MouseActionRecord>,
    pub last_down: HashMap<MouseButton, MouseActionRecord>,
    pub last_up: HashMap<MouseButton, MouseActionRecord>,
    pub dragging: HashMap<MouseButton, bool>,
}

impl MouseInformation {
    pub fn new() -> MouseInformation {
        MouseInformation {
            last_clicked: hash![
                (MouseButton::Left, MouseActionRecord::new_empty()),
                (MouseButton::Right, MouseActionRecord::new_empty()),
                (MouseButton::Middle, MouseActionRecord::new_empty())
            ],
            last_dragged: hash![
                (MouseButton::Left, MouseActionRecord::new_empty()),
                (MouseButton::Right, MouseActionRecord::new_empty()),
                (MouseButton::Middle, MouseActionRecord::new_empty())
            ],
            last_down: hash![
                (MouseButton::Left, MouseActionRecord::new_empty()),
                (MouseButton::Right, MouseActionRecord::new_empty()),
                (MouseButton::Middle, MouseActionRecord::new_empty())
            ],
            last_up: hash![
                (MouseButton::Left, MouseActionRecord::new_empty()),
                (MouseButton::Right, MouseActionRecord::new_empty()),
                (MouseButton::Middle, MouseActionRecord::new_empty())
            ],
            dragging: hash![
                (MouseButton::Left, false),
                (MouseButton::Right, false),
                (MouseButton::Middle, false)
            ],
        }
    }

    pub fn get_last_clicked(&self, button: MouseButton) -> numeric::Point2f {
        match self.last_clicked.get(&button) {
            Some(x) => x.point,
            None => panic!("No such a mouse button"),
        }
    }

    pub fn set_last_clicked(&mut self, button: MouseButton, point: numeric::Point2f, t: Clock) {
        if self
            .last_clicked
            .insert(button, MouseActionRecord::new(point, t))
            == None
        {
            eprintln!("Not basic button is clicked.");
        }
    }

    pub fn get_last_dragged(&self, button: MouseButton) -> numeric::Point2f {
        match self.last_dragged.get(&button) {
            Some(x) => x.point,
            None => panic!("No such a mouse button"),
        }
    }

    pub fn set_last_dragged(&mut self, button: MouseButton, point: numeric::Point2f, t: Clock) {
        if self
            .last_dragged
            .insert(button, MouseActionRecord::new(point, t))
            == None
        {
            panic!("No such a mouse button")
        }
    }

    pub fn get_last_down(&self, button: MouseButton) -> numeric::Point2f {
        match self.last_down.get(&button) {
            Some(x) => x.point,
            None => panic!("No such a mouse button"),
        }
    }

    pub fn set_last_down(&mut self, button: MouseButton, point: numeric::Point2f, t: Clock) {
        if self
            .last_down
            .insert(button, MouseActionRecord::new(point, t))
            == None
        {
            panic!("No such a mouse button")
        }
    }

    pub fn get_last_up(&self, button: MouseButton) -> numeric::Point2f {
        match self.last_up.get(&button) {
            Some(x) => x.point,
            None => panic!("No such a mouse button"),
        }
    }

    pub fn set_last_up(&mut self, button: MouseButton, point: numeric::Point2f, t: Clock) {
        if self
            .last_up
            .insert(button, MouseActionRecord::new(point, t))
            == None
        {
            panic!("No such a mouse button")
        }
    }

    pub fn is_dragging(&self, button: ginput::mouse::MouseButton) -> bool {
        match self.dragging.get(&button) {
            Some(x) => *x,
            None => panic!("No such a mouse button"),
        }
    }

    pub fn update_dragging(&mut self, button: MouseButton, drag: bool) {
        if self.dragging.insert(button, drag) == None {
            panic!("No such a mouse button")
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeAttackModeData {
    goal: u32,
    limit: GensoDate,
}

impl TimeAttackModeData {
    pub fn new(goal: u32, limit: GensoDate) -> Self {
	TimeAttackModeData {
	    goal: goal,
	    limit: limit,
	}
    }

    pub fn get_goal(&self) -> u32 {
	self.goal
    }

    pub fn get_limit(&self) -> &GensoDate {
	&self.limit
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameMode {
    Story,
    TimeAttack(TimeAttackModeData),
}

impl GameMode {
    pub fn story() -> Self {
	Self::Story
    }

    pub fn time_attack(goal: u32) -> Self {
	match goal {
	    300000 => {
		let date = GensoDate::new(112, 7, 23).add_day_chain(60);
		Self::TimeAttack(TimeAttackModeData::new(goal, date))
	    }
	    500000 => {
		let date = GensoDate::new(112, 7, 23).add_day_chain(90);
		Self::TimeAttack(TimeAttackModeData::new(goal, date))
	    }
	    1000000 => {
		let date = GensoDate::new(112, 7, 23).add_day_chain(120);
		Self::TimeAttack(TimeAttackModeData::new(goal, date))
	    }
	    _ => panic!("invalid goal"),
	}
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskResult {
    pub done_works: u32,                         // 総仕事数
    pub not_shelved_books: Vec<BookInformation>, // 返却済, 未配架
    pub borrowing_books: Vec<BookInformation>,   // 貸出中
    pub total_money: i32,                        // 稼いだ金額
}

impl TaskResult {
    pub fn new() -> Self {
        TaskResult {
            done_works: 0,
            not_shelved_books: Vec::new(),
            total_money: 1000,
            borrowing_books: Vec::new(),
        }
    }

    pub fn add_result(&mut self, task_result: &TaskResult) -> &mut Self {
        self.done_works += task_result.done_works;
        self.not_shelved_books
            .extend(task_result.not_shelved_books.clone());
        self.borrowing_books
            .extend(task_result.borrowing_books.clone());
        self.total_money += task_result.total_money;

        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.done_works = 0;
        self.not_shelved_books.clear();
        self.borrowing_books.clear();
        self.total_money = 0;

        self
    }

    pub fn add_total_money(&mut self, money: i32) -> &mut Self {
	self.total_money += money;
	self
    }
}

struct SceneStack {
    stack: VecDeque<TopScene>,
}

impl SceneStack {
    pub fn new() -> SceneStack {
        SceneStack {
            stack: VecDeque::new(),
        }
    }

    pub fn push(&mut self, scene: TopScene) {
        self.stack.push_back(scene);
    }

    pub fn pop(&mut self) -> Option<TopScene> {
        self.stack.pop_back()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SuzunaAnStatus {
    pub jinyou_balance: f32,
    pub reputation: f32,
    pub kosuzu_hp: f32,
}

pub enum ReputationEvent {
    DoneDeskTask,
}

impl SuzunaAnStatus {
    pub fn new() -> Self {
        SuzunaAnStatus {
            jinyou_balance: 0.0,
            reputation: 50.0,
            kosuzu_hp: 100.0,
        }
    }

    pub fn eval_reputation(&mut self, event_type: ReputationEvent) {
        match event_type {
            ReputationEvent::DoneDeskTask => self.reputation += 1.0,
        }
    }
}

#[derive(Clone)]
pub struct BorrowingInformation {
    pub borrowing: Vec<BookInformation>,
    pub borrower: String,
    pub borrow_date: GensoDate,
    pub return_date: GensoDate,
    pub rental_limit: RentalLimit,
}

impl BorrowingInformation {
    pub fn new(
        borrowing: Vec<BookInformation>,
        borrower: &str,
        borrow_date: GensoDate,
        rental_limit: RentalLimit,
    ) -> Self {
        let mut return_date = borrow_date.clone();

        match rental_limit {
            RentalLimit::Today => return_date.add_day(0),
            RentalLimit::ShortTerm => return_date.add_day(7),
            RentalLimit::LongTerm => return_date.add_day(14),
        }

        BorrowingInformation {
            borrowing: borrowing,
            borrower: borrower.to_string(),
            borrow_date: borrow_date,
            return_date: return_date,
            rental_limit: rental_limit,
        }
    }

    pub fn calc_fee(&self) -> i32 {
        (self
            .borrowing
            .iter()
            .map(|info| info.base_price)
            .fold(0, |sum, price| sum + price) as f32
            * self.rental_limit.fee_rate()) as i32
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReturnBookInformation {
    pub returning: Vec<BookInformation>,
    pub borrower: String,
    pub borrow_date: GensoDate,
    pub return_date: GensoDate,
}

impl ReturnBookInformation {
    pub fn new(
        returning: Vec<BookInformation>,
        borrower: &str,
        borrow_date: GensoDate,
        return_date: GensoDate,
    ) -> Self {
        ReturnBookInformation {
            returning: returning,
            borrower: borrower.to_string(),
            borrow_date,
            return_date,
        }
    }

    pub fn new_random(
        game_data: &GameResource,
        borrow_date: GensoDate,
        return_date: GensoDate,
    ) -> Self {
        let borrowing_num = (rand::random::<u32>() % 5) + 1;
        let mut borrow_books = Vec::new();

        for _ in 0..borrowing_num {
            borrow_books.push(game_data.book_random_select().clone());
        }

        Self::new(
            borrow_books,
            game_data.customer_random_select(),
            borrow_date,
            return_date,
        )
    }

    pub fn get_rental_limit(&self) -> RentalLimit {
        self.borrow_date
            .rental_limit_type(&self.return_date)
            .unwrap()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SuzunaBookPool {
    books: Vec<BookInformation>,
}

impl SuzunaBookPool {
    pub fn new(game_data: &GameResource) -> Self {
        let mut books = Vec::new();

        for book_info in game_data.iter_available_books() {
            for _ in 0..5 {
                let cloned = book_info.clone_with_new_id_condition();
                books.push(cloned);
            }
        }

        SuzunaBookPool { books: books }
    }

    pub fn push_book(&mut self, book_info: BookInformation) {
        self.books.push(book_info);
    }

    pub fn push_book_vec(&mut self, book_info_vec: Vec<BookInformation>) {
        self.books.extend(book_info_vec);
    }

    pub fn generate_borrowing_request(
        &mut self,
        customer_name: &str,
        borrow_date: GensoDate,
        rental_limit: RentalLimit,
    ) -> BorrowingInformation {
        let mut borrowing_books: Vec<BookInformation> = Vec::new();
        for _ in 0..((rand::random::<u32>() % 5) + 1) {
            if self.books.is_empty() {
                break;
            }

            let book_info = self
                .books
                .swap_remove(rand::random::<usize>() % self.books.len());

            if borrowing_books
                .iter()
                .any(|info| info.name == book_info.name)
            {
                // 戻してloop再開
                self.books.push(book_info);
                continue;
            }

            borrowing_books.push(book_info);
        }

        println!(
            "generated books count: {}, books_len = {}",
            borrowing_books.len(),
            self.books.len()
        );

        BorrowingInformation::new(borrowing_books, customer_name, borrow_date, rental_limit)
    }

    pub fn generate_returning_request(
        &mut self,
        customer_name: &str,
        borrow_date: GensoDate,
        rental_limit: RentalLimit,
    ) -> ReturnBookInformation {
        let mut returning_books: Vec<BookInformation> = Vec::new();

        for _ in 0..((rand::random::<u32>() % 5) + 1) {
            if self.books.is_empty() {
                break;
            }

            let book_info = self
                .books
                .swap_remove(rand::random::<usize>() % self.books.len());
            if returning_books
                .iter()
                .any(|info| info.name == book_info.name)
            {
                // 既に同じ本を取り出している
                self.push_book(book_info);
                continue;
            }

            returning_books.push(book_info);
        }

        let mut return_date = borrow_date.clone();
        match rental_limit {
            RentalLimit::ShortTerm => return_date.add_day(7),
            RentalLimit::LongTerm => return_date.add_day(14),
            _ => (),
        };

        ReturnBookInformation {
            returning: returning_books,
            borrower: customer_name.to_string(),
            borrow_date: borrow_date,
            return_date: return_date,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ReturningRequestPool {
    returning_request: Vec<ReturnBookInformation>,
}

impl ReturningRequestPool {
    pub fn new(book_pool: &mut SuzunaBookPool, game_data: &GameResource, today: GensoDate) -> Self {
        let mut returning_request = Vec::new();

        let mut day = today.clone();
        day.day -= 7;

        for _ in 1..=7 {
            for _ in 1..=2 {
                let rental_limit = match rand::random::<u32>() % 2 {
                    0 => RentalLimit::ShortTerm,
                    1 => RentalLimit::LongTerm,
                    _ => panic!(""),
                };

                returning_request.push(book_pool.generate_returning_request(
                    game_data.customer_random_select(),
                    day,
                    rental_limit,
                ));
            }
            day.add_day(1);
        }

        ReturningRequestPool {
            returning_request: returning_request,
        }
    }

    pub fn add_request(&mut self, borrow_info: BorrowingInformation) {
        let returning_book_info = ReturnBookInformation::new(
            borrow_info.borrowing.clone(),
            &borrow_info.borrower,
            borrow_info.borrow_date,
            borrow_info.return_date,
        );
        self.returning_request.push(returning_book_info);
    }

    pub fn select_returning_request_random(&mut self) -> Option<ReturnBookInformation> {
        let request_len = self.returning_request.len();

        if request_len == 0 {
            return None;
        }

        Some(
            self.returning_request
                .swap_remove(rand::random::<usize>() % request_len),
        )
    }

    pub fn iter(&self) -> std::slice::Iter<ReturnBookInformation> {
        self.returning_request.iter()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SavableData {
    pub suzuna_book_pool: SuzunaBookPool,
    pub record_book_data: BorrowingRecordBookData,
    pub date: GensoDate,
    pub week_schedule: WeekWorkSchedule,
    pub task_result: TaskResult,
    pub suzunaan_status: SuzunaAnStatus,
    pub ad_status: HashMap<SuzunaAdType, bool>,
    pub agency_status: HashMap<SuzunaAdAgencyType, bool>,
    pub award_data: game_system::AwardData,
    pub game_mode: GameMode,
}

impl SavableData {
    pub fn new(game_data: &GameResource, game_mode: GameMode) -> Self {
        let date = GensoDate::new(112, 7, 23);

        let mut suzuna_book_pool = SuzunaBookPool::new(game_data);
        let returning_request_pool =
            ReturningRequestPool::new(&mut suzuna_book_pool, game_data, date);

        let ad_status = hash![
            (SuzunaAdType::ShopNobori, false),
            (SuzunaAdType::TownNobori, false),
            (SuzunaAdType::Chindon, false),
            (SuzunaAdType::NewsPaper, false),
            (SuzunaAdType::BunBunMaruPaper, false),
            (SuzunaAdType::AdPaper, false)
        ];

        let ad_agency_status = hash![
            (SuzunaAdAgencyType::HakureiJinja, false),
            (SuzunaAdAgencyType::KirisameMahoten, false),
            (SuzunaAdAgencyType::GettoDango, false),
            (SuzunaAdAgencyType::Kusuriya, false),
            (SuzunaAdAgencyType::Hieda, false),
            (SuzunaAdAgencyType::YamaJinja, false)
        ];

        let record_book_data =
            BorrowingRecordBookData::from_returning_request_pool(returning_request_pool);

        SavableData {
            date: date.clone(),
            task_result: TaskResult::new(),
            suzunaan_status: SuzunaAnStatus::new(),
            week_schedule: WeekWorkSchedule::new_empty(date),
            suzuna_book_pool: suzuna_book_pool,
            record_book_data: record_book_data,
            ad_status: ad_status,
            agency_status: ad_agency_status,
            award_data: game_system::AwardData::new(),
	    game_mode: game_mode,
        }
    }

    pub fn save(&self, slot: u8) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(&format!("./resources/save{}", slot))?;

        file.write_all(
            crypt::crypt_str(&serde_json::to_string(self).unwrap())
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        file.flush()?;

        Ok(())
    }

    pub fn delete(slot: u8) {
        match std::fs::remove_file(&format!("./resources/save{}", slot)) {
	    Ok(_) => (),
	    Err(e) => eprintln!("{}", e),
	}
    }

    pub fn new_load(slot: u8) -> Result<SavableData, ()> {
        let file = std::fs::File::open(&format!("./resources/save{}", slot));
        if file.is_err() {
            return Err(());
        }

        let mut buf = Vec::new();
        if let Ok(_) = file.unwrap()
                    .read_to_end(&mut buf) {
            ()
        } else {
            return Err(())
        }

        let content = crypt::decrypt_str(&buf);

        let savable_data = serde_json::from_str(&content.unwrap());

        if savable_data.is_err() {
            Err(())
        } else {
            Ok(savable_data.unwrap())
        }
    }

    pub fn replace(&mut self, data: SavableData) {
        self.suzuna_book_pool = data.suzuna_book_pool;
        self.record_book_data = data.record_book_data;
        self.date = data.date;
        self.task_result = data.task_result;
        self.suzunaan_status = data.suzunaan_status;
        self.ad_status = data.ad_status;
        self.agency_status = data.agency_status;
        self.week_schedule = data.week_schedule;
	self.award_data = data.award_data;
	self.game_mode = data.game_mode;
    }

    pub fn change_ad_status(&mut self, ad_type: SuzunaAdType, status: bool) {
        self.ad_status.insert(ad_type, status);
    }

    pub fn change_ad_agency_status(&mut self, ad_type: SuzunaAdAgencyType, status: bool) {
        self.agency_status.insert(ad_type, status);
    }

    pub fn get_ad_status(&self, ad_type: SuzunaAdType) -> bool {
        *self.ad_status.get(&ad_type).unwrap()
    }

    pub fn get_ad_agency_status(&self, agency_type: &SuzunaAdAgencyType) -> bool {
        *self.agency_status.get(agency_type).unwrap()
    }

    pub fn pay_ad_cost(&mut self, resource: &GameResource) -> i32 {
        let mut total_cost = 0;

        for (ad_type, used) in self.ad_status.iter() {
            if *used {
                let cost = resource.get_default_ad_cost(*ad_type) as i32;
                self.task_result.total_money -= cost;
                total_cost += cost;
            }
        }

        total_cost
    }

    pub fn update_week_schedule(&mut self, sched: WeekWorkSchedule) {
        self.week_schedule = sched;
    }

    pub fn get_todays_schedule(&self) -> Option<game_system::DayWorkType> {
        self.week_schedule.get_schedule_of(&self.date).clone()
    }
}

pub struct ResultReportStringTable {
    pub total_customers_waiting_time: String,
    pub shelving_is_done: String,
    pub condition_eval_mistakes: String,
    pub total_ad_cost: String,
}

#[derive(Clone, Debug)]
pub struct ResultReport {
    new_books_id: Vec<u64>,
    yet_shelved_books_id: Vec<u64>,
    total_customers_waiting_time: Clock,
    condition_eval_mistakes: usize,
    total_ad_cost: i32,
}

impl ResultReport {
    pub fn new() -> Self {
        ResultReport {
            new_books_id: Vec::new(),
            yet_shelved_books_id: Vec::new(),
            total_customers_waiting_time: 0,
            condition_eval_mistakes: 0,
            total_ad_cost: 0,
        }
    }

    pub fn add_new_book_id(&mut self, id: u64) {
        self.new_books_id.push(id);
    }

    pub fn add_condition_eval_mistakes(&mut self, mistakes: usize) {
        self.condition_eval_mistakes += mistakes;
    }

    pub fn add_yet_shelved_book_id(&mut self, id: u64) {
        self.yet_shelved_books_id.push(id);
    }

    pub fn add_customers_waiting_time(&mut self, additional: Clock) {
        self.total_customers_waiting_time += additional;
    }

    pub fn add_ad_cost(&mut self, cost: i32) {
        self.total_ad_cost += cost;
    }

    pub fn create_table(&self) -> ResultReportStringTable {
        ResultReportStringTable::new(self)
    }

    pub fn new_books_shelving_is_done(&self) -> bool {
        for new_books_id in self.new_books_id.iter() {
            for yet_id in self.yet_shelved_books_id.iter() {
                if new_books_id == yet_id {
                    return false;
                }
            }
        }

        return true;
    }

    pub fn get_conition_eval_mistakes(&self) -> usize {
        self.condition_eval_mistakes
    }

    fn number_of_yet_shelved_and_new_books(&self) -> usize {
	let mut count = 0;
	
	for yet_id in self.yet_shelved_books_id.iter() {
	    if self.new_books_id.contains(yet_id) {
		count += 1;
	    }
        }

	count
    }
    
    pub fn generate_eval_str(&self) -> &str {
	let missed_books_num = self.number_of_yet_shelved_and_new_books();
	let total_waiting_minute = self.total_customers_waiting_time / 60;
	
	if missed_books_num == 0 &&
	    total_waiting_minute < 30 &&
	    self.condition_eval_mistakes == 0 {
		return "鈴奈庵の主";
	    }

	if missed_books_num == 0 &&
	    total_waiting_minute < 60 &&
	    self.condition_eval_mistakes == 0 {
		return "占い師より向いてる";
	    }

	if missed_books_num <= 3 &&
	    total_waiting_minute < 120 &&
	    self.condition_eval_mistakes == 1 {
		return "見習い";
	    }

	if missed_books_num <= 3 &&
	    total_waiting_minute < 120 {
		return "霊夢より働く";
	    }

	return "素人";
    }
}

impl ResultReportStringTable {
    pub fn new(result_report: &ResultReport) -> Self {
        ResultReportStringTable {
            total_customers_waiting_time: number_to_jk::number_to_jk(
                result_report.total_customers_waiting_time / 60,
            ),
            shelving_is_done: if result_report.new_books_shelving_is_done() {
                "達成"
            } else {
                "未達成"
            }
            .to_string(),
            condition_eval_mistakes: number_to_jk::number_to_jk(
                result_report.condition_eval_mistakes as u64,
            ),
            total_ad_cost: number_to_jk::number_to_jk(result_report.total_ad_cost as u64),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameConfig {
    bgm_volume: f32,
    se_volume: f32,
    minute_per_clock: Clock,
    pause_when_inactive: bool,
}

impl GameConfig {
    pub fn new_from_toml(ctx: &mut ggez::Context, path: &str) -> Self {
	match File::open("./game_config") {
	    Ok(mut file) => {
		let mut buf = Vec::new();
		match file
		    .read_to_end(&mut buf) {
			Ok(_) => (),
			Err(_) => return Self::load_default_config(ctx, path),
		    }
		
		let content = crypt::decrypt_str(&buf);
		
		let game_config = serde_json::from_str(&content.unwrap());

		match game_config {
		    Ok(game_config) => game_config,
		    Err(_) => Self::load_default_config(ctx, path)
		}
	    },
	    Err(_) => Self::load_default_config(ctx, path)
	}
    }

    fn load_default_config(ctx: &mut ggez::Context, path: &str) -> Self {
	let s = util::read_from_resources_as_string(ctx, path);

        let raw_data: Result<GameConfig, toml::de::Error> = toml::from_str(&s);
        match raw_data {
            Ok(p) => p,
            Err(e) => panic!("Failed to parse toml: {}", e),
        }
    }

    pub fn set_bgm_volume_100(&mut self, volume: f32) {
        self.bgm_volume = volume / 100.0;
    }

    pub fn set_se_volume_100(&mut self, volume: f32) {
        self.se_volume = volume / 100.0;
    }

    pub fn get_bgm_volume(&self) -> f32 {
        self.bgm_volume
    }

    pub fn get_se_volume(&self) -> f32 {
        self.se_volume
    }

    pub fn is_pause_when_inactive(&self) -> bool {
	self.pause_when_inactive
    }

    pub fn set_pause_when_inactive(&mut self, flag: bool) {
	self.pause_when_inactive = flag;
    }

    pub fn save_config(&self) {
	let mut file = File::create("./game_config").expect("failed to create game config file.");

        file.write_all(
            crypt::crypt_str(&serde_json::to_string(self).unwrap())
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        file.flush().expect("failed to flush game config file");
    }
}

pub struct ProcessUtility<'ctx> {
    pub redraw_request: &'ctx mut scene::DrawRequest,
}

impl<'ctx> ProcessUtility<'ctx> {
    pub fn redraw(&mut self) {
        *self.redraw_request = scene::DrawRequest::Draw;
    }

    pub fn redraw_or(&mut self, request: scene::DrawRequest) {
        *self.redraw_request |= request;
    }
}

pub struct SuzuContext<'ctx> {
    pub context: &'ctx mut ggez::Context,
    pub resource: &'ctx mut GameResource,
    pub savable_data: &'ctx mut Option<SavableData>,
    pub config: &'ctx mut GameConfig,
    pub process_utility: ProcessUtility<'ctx>,
}

impl<'ctx> SuzuContext<'ctx> {
    pub fn take_save_data_mut(&mut self) -> &mut SavableData {
	self.savable_data.as_mut().expect("save data not found")
    }

    pub fn take_save_data(&self) -> &SavableData {
	self.savable_data.as_ref().expect("save data not found")
    }
    
    pub fn ref_texture(&mut self, id: TextureID) -> ggraphics::Image {
        self.resource.ref_texture(self.context, id)
    }

    pub fn play_sound_as_bgm(
        &mut self,
        sound_id: SoundID,
        flags: Option<sound::SoundPlayFlags>,
    ) -> sound::SoundHandler {
        self.resource
            .play_sound_as_bgm(self.context, sound_id, flags)
    }

    pub fn play_sound_as_se(
        &mut self,
        sound_id: SoundID,
        flags: Option<sound::SoundPlayFlags>,
    ) -> sound::SoundHandler {
        self.resource
            .play_sound_as_se(self.context, sound_id, flags)
    }

    pub fn change_bgm_volume(&mut self, volume: f32) {
        self.resource.change_bgm_volume(volume / 100.0);
        self.config.set_bgm_volume_100(volume);
    }

    pub fn change_se_volume(&mut self, volume: f32) {
        self.resource.change_se_volume(volume / 100.0);
        self.config.set_se_volume_100(volume);
    }

    pub fn pay_ad_cost(&mut self) -> i32 {
        self.savable_data.as_mut().expect("save data not found").pay_ad_cost(self.resource)
    }

    pub fn holding_week_schedule_is_available(&self) -> bool {
        self.take_save_data()
            .week_schedule
            .update_is_not_required(&self.take_save_data().date)
    }

    pub fn go_next_day(&mut self) {
        self.take_save_data_mut().date.add_day(1);
    }

    pub fn current_total_ad_cost(&self) -> i32 {
        let mut total_ad_cost = 0;

        for ad_type in vec![
            SuzunaAdType::AdPaper,
            SuzunaAdType::Chindon,
            SuzunaAdType::ShopNobori,
            SuzunaAdType::TownNobori,
            SuzunaAdType::NewsPaper,
            SuzunaAdType::BunBunMaruPaper,
        ] {
            if *self.take_save_data().ad_status.get(&ad_type).unwrap() {
                total_ad_cost += self.resource.get_default_ad_cost(ad_type);
            }
        }

        return total_ad_cost as i32;
    }

    pub fn current_total_ad_reputation_gain(&self) -> i32 {
        let mut total_ad_reputation_gain = 0;

        for ad_type in vec![
            SuzunaAdType::AdPaper,
            SuzunaAdType::Chindon,
            SuzunaAdType::ShopNobori,
            SuzunaAdType::TownNobori,
            SuzunaAdType::NewsPaper,
            SuzunaAdType::BunBunMaruPaper,
        ] {
            if *self.take_save_data().ad_status.get(&ad_type).unwrap() {
                total_ad_reputation_gain += self.resource.get_default_ad_reputation_gain(ad_type);
            }
        }

        return total_ad_reputation_gain as i32;
    }

    pub fn current_total_ad_agency_money_gain(&self) -> i32 {
        let mut total_ad_agency_money_gain = 0;

        for ad_type in vec![
            SuzunaAdAgencyType::HakureiJinja,
            SuzunaAdAgencyType::KirisameMahoten,
            SuzunaAdAgencyType::GettoDango,
            SuzunaAdAgencyType::Kusuriya,
            SuzunaAdAgencyType::Hieda,
            SuzunaAdAgencyType::YamaJinja,
        ] {
            if *self.take_save_data().agency_status.get(&ad_type).unwrap() {
                total_ad_agency_money_gain +=
                    self.resource.get_default_ad_agency_money_gain(&ad_type);
            }
        }

        return total_ad_agency_money_gain as i32;
    }

    pub fn reset_save_data(&mut self, game_mode: GameMode) {
	*self.savable_data = Some(SavableData::new(&self.resource, game_mode));
    }

    pub fn save(&mut self, slot_id: u8) -> Result<(), ()> {
	if let Some(save_data) = self.savable_data.as_mut() {
	    match save_data.save(slot_id) {
		Ok(_) => Ok(()),
		Err(_) => Err(()),
	    }
	} else {
	    Err(())
	}
    }

    pub fn change_ad_status(&mut self, ad_type: SuzunaAdType, status: bool) {
	self.take_save_data_mut().change_ad_status(ad_type, status);
    }

    pub fn change_ad_agency_status(&mut self, ad_type: SuzunaAdAgencyType, status: bool) {
	self.take_save_data_mut().change_ad_agency_status(ad_type, status);
    }

    pub fn update_week_schedule(&mut self, sched: WeekWorkSchedule) {
	self.take_save_data_mut().update_week_schedule(sched);
    }
}

pub enum TopScene {
    ScenarioScene(scene::scenario_scene::ScenarioScene),
    SuzunaScene(scene::suzuna_scene::SuzunaScene),
    SaveScene(scene::save_scene::SaveScene),
    TitleScene(scene::title_scene::TitleScene),
    EndScene(scene::end_scene::EndScene),
    Null(scene::NullScene),
}

impl TopScene {
    pub fn abs(&self) -> &dyn scene::SceneManager {
        match self {
            TopScene::ScenarioScene(scene) => scene,
            TopScene::SuzunaScene(scene) => scene,
            TopScene::SaveScene(scene) => scene,
            TopScene::TitleScene(scene) => scene,
            TopScene::EndScene(scene) => scene,
            TopScene::Null(scene) => scene,
        }
    }

    pub fn abs_mut(&mut self) -> &mut dyn scene::SceneManager {
        match self {
            TopScene::ScenarioScene(scene) => scene,
            TopScene::SuzunaScene(scene) => scene,
            TopScene::SaveScene(scene) => scene,
            TopScene::TitleScene(scene) => scene,
            TopScene::EndScene(scene) => scene,
            TopScene::Null(scene) => scene,
        }
    }

    pub fn to_suzuna_scene(&self) -> Option<&scene::suzuna_scene::SuzunaScene> {
        match self {
            TopScene::SuzunaScene(scene) => Some(scene),
            _ => None,
        }
    }
}

struct SceneController {
    current_scene: TopScene,
    scene_stack: SceneStack,
    key_map: tdev::ProgramableGenericKey,
    global_clock: u64,
    root_screen: SubScreen,
    game_status: Option<SavableData>,
    game_config: GameConfig,
    redraw_request: scene::DrawRequest,
}

impl SceneController {
    pub fn new<'a>(ctx: &mut ggez::Context, game_data: &'a mut GameResource) -> SceneController {
        let window_size = ggraphics::drawable_size(ctx);

        let mut root_screen = SubScreen::new(
            ctx,
            numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
            0,
            ggraphics::Color::from_rgba_u32(0),
        );

        root_screen.fit_scale(
            ctx,
            numeric::Vector2f::new(window_size.0.round(), window_size.1.round()),
        );

        debug::debug_screen_init(
            ctx,
            numeric::Rect::new(940.0, 0.0, 420.0, 300.0),
            FontInformation::new(
                game_data.get_font(FontID::DEFAULT),
                numeric::Vector2f::new(12.0, 12.0),
                ggraphics::Color::from_rgba_u32(0xffffffa0),
            ),
        );
        debug::debug_screen_hide();

        let mut game_status = None;
        let mut game_config = GameConfig::new_from_toml(ctx, "/default_game_config.toml");

        let mut _redraw_request = scene::DrawRequest::Draw;

        // let current_scene = scene::scenario_scene::ScenarioScene::new(&mut SuzuContext {
        //     context: ctx,
        //     resource: game_data,
        //     savable_data: &mut game_status,
        // });
        let current_scene = scene::title_scene::TitleScene::new(&mut SuzuContext {
            context: ctx,
            resource: game_data,
            savable_data: &mut game_status,
            config: &mut game_config,
            process_utility: ProcessUtility {
                redraw_request: &mut _redraw_request,
            },
        });

        SceneController {
            //current_scene: TopScene::ScenarioScene(current_scene),
            current_scene: TopScene::TitleScene(current_scene),
            scene_stack: SceneStack::new(),
            key_map: tdev::ProgramableGenericKey::new(),
            global_clock: 0,
            root_screen: root_screen,
            game_status: game_status,
            game_config: game_config,
            redraw_request: scene::DrawRequest::Draw,
        }
    }

    fn switch_scene_with_swap<'a>(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &'a mut GameResource,
        next_scene_id: scene::SceneID,
    ) {
        let mut ctx = SuzuContext {
            context: ctx,
            resource: game_data,
            savable_data: &mut self.game_status,
            config: &mut self.game_config,
            process_utility: ProcessUtility {
                redraw_request: &mut self.redraw_request,
            },
        };

        match next_scene_id {
            scene::SceneID::SuzunaShop => {
                self.current_scene =
                    TopScene::SuzunaScene(scene::suzuna_scene::SuzunaScene::new(&mut ctx, 0))
            }
            scene::SceneID::Scenario => match self.current_scene {
                TopScene::SuzunaScene(_) => {
                    self.current_scene =
                        TopScene::ScenarioScene(scene::scenario_scene::ScenarioScene::new(
                            &mut ctx,
                            scene::scenario_scene::ScenarioSelect::DayBegin,
                        ))
                }
                TopScene::SaveScene(_) => {
                    self.current_scene =
                        TopScene::ScenarioScene(scene::scenario_scene::ScenarioScene::new(
                            &mut ctx,
                            scene::scenario_scene::ScenarioSelect::DayBegin,
                        ))
                }
                TopScene::TitleScene(_) => {
                    self.current_scene =
                        TopScene::ScenarioScene(scene::scenario_scene::ScenarioScene::new(
                            &mut ctx,
                            scene::scenario_scene::ScenarioSelect::DayBegin,
                        ))
                }
                TopScene::ScenarioScene(_) => {
                    self.current_scene =
                        TopScene::ScenarioScene(scene::scenario_scene::ScenarioScene::new(
                            &mut ctx,
                            scene::scenario_scene::ScenarioSelect::DayBegin,
                        ))
                }
                _ => (),
            },
            scene::SceneID::Title => {
                self.current_scene =
                    TopScene::TitleScene(scene::title_scene::TitleScene::new(&mut ctx))
            }
            scene::SceneID::Save => {
                self.current_scene =
                    TopScene::SaveScene(scene::save_scene::SaveScene::new(&mut ctx));
            }
            scene::SceneID::End => {
                self.current_scene = TopScene::EndScene(scene::end_scene::EndScene::new(&mut ctx))
            }
            scene::SceneID::Null => self.current_scene = TopScene::Null(scene::NullScene::new()),
            _ => (),
        }
    }

    fn switch_scene_with_stacking<'a>(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &'a mut GameResource,
        next_scene_id: scene::SceneID,
    ) {
        let mut ctx = SuzuContext {
            context: ctx,
            resource: game_data,
            savable_data: &mut self.game_status,
            config: &mut self.game_config,
            process_utility: ProcessUtility {
                redraw_request: &mut self.redraw_request,
            },
        };

        let next_scene = match next_scene_id {
            scene::SceneID::SuzunaShop => Some(TopScene::SuzunaScene(
                scene::suzuna_scene::SuzunaScene::new(&mut ctx, 0),
            )),
            scene::SceneID::Save => Some(TopScene::SaveScene(scene::save_scene::SaveScene::new(
                &mut ctx,
            ))),
            scene::SceneID::Null => Some(TopScene::Null(scene::NullScene::new())),
            _ => None,
        };

        if let Some(mut scene) = next_scene {
            std::mem::swap(&mut self.current_scene, &mut scene);
            self.scene_stack.push(scene);
        }
    }

    fn run_pre_process(&mut self, ctx: &mut ggez::Context, game_data: &mut GameResource) {
        //println!("{}", perf_measure!(
        {
            self.current_scene.abs_mut().pre_process(&mut SuzuContext {
                context: ctx,
                resource: game_data,
                savable_data: &mut self.game_status,
                config: &mut self.game_config,
                process_utility: ProcessUtility {
                    redraw_request: &mut self.redraw_request,
                },
            });
        }
        //));
    }

    fn run_drawing_process(&mut self, ctx: &mut ggez::Context) {
        //println!("{}", perf_measure!(
        {
            sub_screen::stack_screen(ctx, &self.root_screen);

            self.current_scene.abs_mut().drawing_process(ctx);

            debug::debug_screen_draw(ctx);

            sub_screen::pop_screen(ctx);
            self.root_screen.draw(ctx).unwrap();
        }
        //) as f32 / 1000000.0);
    }

    fn run_post_process<'a>(&mut self, ctx: &mut ggez::Context, game_data: &'a mut GameResource) {
        let mut suzu_ctx = SuzuContext {
            context: ctx,
            resource: game_data,
            savable_data: &mut self.game_status,
            config: &mut self.game_config,
            process_utility: ProcessUtility {
                redraw_request: &mut self.redraw_request,
            },
        };

        match self.current_scene.abs_mut().post_process(&mut suzu_ctx) {
            scene::SceneTransition::Keep => (),
            scene::SceneTransition::Reset => (),
            scene::SceneTransition::SwapTransition => {
                self.switch_scene_with_swap(ctx, game_data, self.current_scene.abs().transition())
            }
            scene::SceneTransition::StackingTransition => {
                self.switch_scene_with_stacking(
                    ctx,
                    game_data,
                    self.current_scene.abs().transition(),
                );
            }
            scene::SceneTransition::PoppingTransition => {
                if let Some(scene) = self.scene_stack.pop() {
                    self.current_scene = scene;
                    self.current_scene
                        .abs_mut()
                        .scene_popping_return_handler(&mut suzu_ctx);
                } else {
                    eprintln!("Scene Stack is Empty!!");
                }
            }
        }

        if self.global_clock % 120 == 0 {
            println!("fps: {}", ggez::timer::fps(ctx));
        }
        self.global_clock += 1;
        self.redraw_request = scene::DrawRequest::Skip;
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        game_data: &mut GameResource,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        if keycode == KeyCode::F1 {
            debug::debug_screen_appear();
        }

        if keycode == KeyCode::F2 {
            debug::debug_screen_hide();
        }

        self.current_scene.abs_mut().key_down_event(
            &mut SuzuContext {
                context: ctx,
                resource: game_data,
                savable_data: &mut self.game_status,
                config: &mut self.game_config,
                process_utility: ProcessUtility {
                    redraw_request: &mut self.redraw_request,
                },
            },
            self.key_map.real_to_virtual(keycode),
        );

        self.redraw_request = scene::DrawRequest::Draw;
    }

    fn key_up_event(
        &mut self,
        ctx: &mut Context,
        game_data: &mut GameResource,
        keycode: KeyCode,
        _keymods: KeyMods,
    ) {
        self.current_scene.abs_mut().key_up_event(
            &mut SuzuContext {
                context: ctx,
                resource: game_data,
                savable_data: &mut self.game_status,
                config: &mut self.game_config,
                process_utility: ProcessUtility {
                    redraw_request: &mut self.redraw_request,
                },
            },
            self.key_map.real_to_virtual(keycode),
        );

        self.redraw_request = scene::DrawRequest::Draw;
    }

    fn mouse_motion_event<'a>(
        &mut self,
        ctx: &mut Context,
        game_data: &mut GameResource,
        point: numeric::Point2f,
        offset: numeric::Vector2f,
    ) {
        self.current_scene.abs_mut().mouse_motion_event(
            &mut SuzuContext {
                context: ctx,
                resource: game_data,
                savable_data: &mut self.game_status,
                config: &mut self.game_config,
                process_utility: ProcessUtility {
                    redraw_request: &mut self.redraw_request,
                },
            },
            point,
            offset,
        );

        self.redraw_request = scene::DrawRequest::Draw;
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &mut GameResource,
        button: ginput::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        self.current_scene.abs_mut().mouse_button_down_event(
            &mut SuzuContext {
                context: ctx,
                resource: game_data,
                savable_data: &mut self.game_status,
                config: &mut self.game_config,
                process_utility: ProcessUtility {
                    redraw_request: &mut self.redraw_request,
                },
            },
            button,
            point,
        );
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &mut GameResource,
        button: ginput::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        self.current_scene.abs_mut().mouse_button_up_event(
            &mut SuzuContext {
                context: ctx,
                resource: game_data,
                savable_data: &mut self.game_status,
                config: &mut self.game_config,
                process_utility: ProcessUtility {
                    redraw_request: &mut self.redraw_request,
                },
            },
            button,
            point,
        );
    }

    fn mouse_wheel_scroll_event<'a>(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &mut GameResource,
        x: f32,
        y: f32,
    ) {
        let point = ggez::input::mouse::position(ctx);
        self.current_scene.abs_mut().mouse_wheel_event(
            &mut SuzuContext {
                context: ctx,
                resource: game_data,
                savable_data: &mut self.game_status,
                config: &mut self.game_config,
                process_utility: ProcessUtility {
                    redraw_request: &mut self.redraw_request,
                },
            },
            numeric::Point2f::new(point.x, point.y),
            x,
            y,
        );
    }

    fn redraw_request_status(&self) -> scene::DrawRequest {
        self.redraw_request
    }

    pub fn focus_event(&mut self, ctx: &mut ggez::Context, game_data: &mut GameResource) {
        self.current_scene.abs_mut().focus_event(&mut SuzuContext {
            context: ctx,
            resource: game_data,
            savable_data: &mut self.game_status,
            config: &mut self.game_config,
            process_utility: ProcessUtility {
                redraw_request: &mut self.redraw_request,
            },
        });
    }

    pub fn unfocus_event(&mut self, ctx: &mut ggez::Context, game_data: &mut GameResource) {
        self.current_scene
            .abs_mut()
            .unfocus_event(&mut SuzuContext {
                context: ctx,
                resource: game_data,
                savable_data: &mut self.game_status,
                config: &mut self.game_config,
                process_utility: ProcessUtility {
                    redraw_request: &mut self.redraw_request,
                },
            });
    }
}

pub struct State {
    clock: Clock,
    fps: f64,
    scene_controller: SceneController,
    game_data: GameResource,
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.scene_controller.run_pre_process(ctx, &mut self.game_data);

        self.clock += 1;
        if (self.clock % 100) == 0 {
            self.fps = timer::fps(ctx);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        match self.scene_controller.redraw_request_status() {
            scene::DrawRequest::Draw | scene::DrawRequest::InitDraw => {
                graphics::clear(ctx, [0.0, 0.0, 0.0, 0.0].into());
                self.scene_controller.run_drawing_process(ctx);
            }
            _ => (),
        }

        graphics::present(ctx)?;

        self.scene_controller.run_post_process(ctx, &mut self.game_data);

        Ok(())
    }

    fn quit_event(&mut self, ctx: &mut Context) -> bool {
	ggez::event::quit(ctx);
	false
    }

    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        keycode: KeyCode,
        keymods: KeyMods,
        repeat: bool,
    ) {
        self.scene_controller
            .key_down_event(ctx, &mut self.game_data, keycode, keymods, repeat);
    }

    fn key_up_event(&mut self, ctx: &mut ggez::Context, keycode: KeyCode, keymods: KeyMods) {
        self.scene_controller
            .key_up_event(ctx, &mut self.game_data, keycode, keymods);
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        self.scene_controller.mouse_motion_event(
            ctx,
            &mut self.game_data,
            numeric::Point2f::new(x, y),
            numeric::Vector2f::new(dx, dy),
        );
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: ginput::mouse::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.scene_controller.mouse_button_down_event(
            ctx,
            &mut self.game_data,
            button,
            numeric::Point2f::new(x, y),
        );
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut Context,
        button: ginput::mouse::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.scene_controller.mouse_button_up_event(
            ctx,
            &mut self.game_data,
            button,
            numeric::Point2f::new(x, y),
        );
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        self.scene_controller
            .mouse_wheel_scroll_event(ctx, &mut self.game_data, x, y);
    }

    fn focus_event(&mut self, ctx: &mut Context, gained: bool) {
        if gained {
            self.scene_controller.focus_event(ctx, &mut self.game_data);
        } else {
            self.scene_controller.unfocus_event(ctx, &mut self.game_data);
        }
    }
}

impl State {
    pub fn new(ctx: &mut Context, mut game_data: GameResource) -> GameResult<State> {
        let scene_controller = SceneController::new(ctx, &mut game_data);

        game_data
            .bgm_manager
            .change_global_volume(scene_controller.game_config.bgm_volume);
        game_data
            .se_manager
            .change_global_volume(scene_controller.game_config.se_volume);

        let s = State {
            clock: 0,
            fps: 0.0,
            scene_controller: scene_controller,
            game_data: game_data,
        };

        Ok(s)
    }
}

pub fn font_information_from_toml_value<'a>(
    game_data: &'a GameResource,
    toml_value: &toml::Value,
) -> FontInformation {
    let font_str = toml_value["FontID"].as_str().unwrap();

    let scale_table = toml_value["scale"].as_table().unwrap();

    let scale = numeric::Vector2f::new(
        scale_table["x"].as_float().unwrap() as f32,
        scale_table["y"].as_float().unwrap() as f32,
    );

    let color_hex_code = toml_value["color"].as_integer().unwrap() as u32;

    FontInformation::new(
        game_data.get_font(FontID::from_str(font_str).unwrap()),
        scale,
        ggraphics::Color::from_rgba_u32(color_hex_code),
    )
}
