pub mod map_parser;
pub mod util;
pub mod book_management;

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
use std::rc::Rc;
use std::str::FromStr;

use crate::scene;
use crate::parse_toml_file;

use std::fs;
use std::fs::File;
use std::io::{BufReader, Read, Write};

use serde::{Deserialize, Serialize};
extern crate serde_json;

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

fn read_whole_file(path: String) -> Result<String, String> {
    let mut file_content = String::new();

    let mut fr = fs::File::open(path)
        .map(|f| BufReader::new(f))
        .map_err(|e| e.to_string())?;

    fr.read_to_string(&mut file_content)
        .map_err(|e| e.to_string())?;

    Ok(file_content)
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
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum FontID {
    DEFAULT = 0,
    JpFude1,
    CorpMincho,
    Cinema,
}

impl FromStr for FontID {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "Default" => Ok(FontID::DEFAULT),
            "JpFude1" => Ok(FontID::JpFude1),
            "CorpMincho" => Ok(FontID::CorpMincho),
            "Cinema" => Ok(FontID::Cinema),
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
            _ => None,
        }
    }
}

impl TextureID {
    pub fn select_random() -> Self {
        TextureID::from_u32(rand::random::<u32>() % (Self::Unknown as u32)).unwrap()
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

#[derive(Clone)]
pub enum SoundID {
    Unknown = 0,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookInformation {
    pub name: String,
    pub pages: usize,
    pub size: String,
    pub billing_number: u16,
    pub base_price: u32,
}

impl BookInformation {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_pages(&self) -> usize {
        self.pages
    }
}

#[derive(PartialEq, Clone)]
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

    pub fn add_day(&mut self, day: u8) {
        self.day += day;
        if self.day > 31 {
            self.month += 1;
            self.day %= 31;
        }

        if self.month > 12 {
            self.season += 1;
            self.month %= 12;
        }
    }

    pub fn rental_limit_type(&self, limit: &GensoDate) -> Option<RentalLimit> {
        let month_diff = limit.month - self.month;
        let maybe_day_diff = if month_diff == 1 {
            Some(limit.day + (31 - self.day))
        } else if month_diff == 0 {
            Some(limit.day - self.day)
        } else {
            None
        };

        if let Some(day_diff) = maybe_day_diff {
            println!("day_diff: {:?}", day_diff);
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
}

pub struct ScenarioTable {
    scenario_table: HashMap<GensoDate, String>,
}

impl ScenarioTable {
    pub fn new(table_toml_path: &str) -> Self {
        let mut table = HashMap::new();

        let root = parse_toml_file!(table_toml_path);
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

        ScenarioTable {
            scenario_table: table,
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

#[derive(Debug, Deserialize)]
pub struct RawConfigFile {
    texture_paths: Vec<String>,
    font_paths: Vec<String>,
    customers_name: Vec<String>,
    books_information: Vec<BookInformation>,
    map_information: Vec<MapConstractData>,
    sprite_batch_information: Vec<SpriteBatchData>,
    scenario_table_path: String,
    sound_file_path: Vec<String>,
}

impl RawConfigFile {
    pub fn new(file_path: String) -> RawConfigFile {
        let s = match read_whole_file(file_path) {
            Ok(s) => s,
            Err(e) => panic!("Failed to read file: {}", e),
        };

        let raw_data: Result<RawConfigFile, toml::de::Error> = toml::from_str(&s);
        match raw_data {
            Ok(p) => p,
            Err(e) => panic!("Failed to parse toml: {}", e),
        }
    }
}

pub struct GameResource {
    texture_resource_paths: HashMap<TextureID, String>,
    textures: HashMap<TextureID, Rc<ggraphics::Image>>,
    fonts: Vec<ggraphics::Font>,
    tile_batchs: Vec<TileBatch>,
    customers_name: Vec<String>,
    books_information: Vec<BookInformation>,
    map_data: Vec<MapConstractData>,
    scenario_table: ScenarioTable,
    sounds: Vec<sound::SoundData>,
    sound_manager: sound::SoundManager,
}

impl GameResource {
    pub fn new(ctx: &mut ggez::Context, file_path: String) -> GameResource {
        let init_display = InitialDisplay::new(ctx);
        init_display.draw(ctx);

        let src_file = RawConfigFile::new(file_path);

        let textures = HashMap::new();
        let mut fonts = Vec::new();
        let mut sprite_batchs = Vec::new();
        let mut sounds = Vec::new();
	let mut texture_paths_map = HashMap::new();

	print!("Setup textures delay loading ... ");
        for (index, texture_path) in src_file.texture_paths.iter().enumerate() {
	    texture_paths_map.insert(TextureID::from_u32(index as u32).unwrap(), texture_path.clone());
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
            sounds.push(sound_data);
        }

        let scenario_table = ScenarioTable::new(&src_file.scenario_table_path);

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
            sound_manager: sound::SoundManager::new(),
        }
    }

    fn load_texture_delay(&mut self, ctx: &mut ggez::Context, id: TextureID) -> Rc<ggraphics::Image> {
	let path = self.texture_resource_paths.get(&id).expect("Delay texture load: Invalid TextureID");	
	print!("delay texture loading -> {} ... ", path);
	let texture = Rc::new(
	    ggraphics::Image::new(
		ctx, path
	    ).expect("Delay texture load: Invalid Path")
	);
	self.textures.insert(
	    id,
	    texture.clone()
	);
	println!("done!");

	texture
    }
    
    pub fn ref_texture(&mut self, ctx: &mut ggez::Context, id: TextureID) -> Rc<ggraphics::Image> {
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

    pub fn clone_available_books(&self) -> Vec<BookInformation> {
        self.books_information.clone()
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

    pub fn play_sound(
        &mut self,
        ctx: &mut ggez::Context,
        sound_id: SoundID,
        flags: Option<sound::SoundPlayFlags>,
    ) -> sound::SoundHandler {
        let sound_data = self.sounds.get(sound_id as usize).unwrap();
        self.sound_manager.play(ctx, sound_data.clone(), flags)
    }

    pub fn ref_sound(&self, handler: sound::SoundHandler) -> &sound::PlayableSound {
        self.sound_manager.ref_sound(handler)
    }

    pub fn ref_sound_mut(&mut self, handler: sound::SoundHandler) -> &mut sound::PlayableSound {
        self.sound_manager.ref_sound_mut(handler)
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
            panic!("No such a mouse button")
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
            total_money: 0,
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
}

pub enum ReputationEvent {
    DoneDeskTask,
}

impl SuzunaAnStatus {
    pub fn new() -> Self {
        SuzunaAnStatus {
            jinyou_balance: 0.0,
            reputation: 50.0,
        }
    }

    pub fn eval_reputation(&mut self, event_type: ReputationEvent) {
	match event_type {
	    ReputationEvent::DoneDeskTask => self.reputation += 2.0,
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
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SuzunaBookPool {
    books: Vec<BookInformation>,
}

impl SuzunaBookPool {
    pub fn new(game_data: &GameResource) -> Self {
        SuzunaBookPool {
            books: game_data.clone_available_books(),
        }
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
        let mut borrowing_books = Vec::new();
        for _ in 0..((rand::random::<u32>() % 5) + 1) {
            if self.books.is_empty() {
                break;
            }

            let book_info = self
                .books
                .swap_remove(rand::random::<usize>() % self.books.len());
            borrowing_books.push(book_info);
        }

        println!(
            "generated books count: {}, books_len = {}",
            borrowing_books.len(),
            self.books.len()
        );

        BorrowingInformation::new(borrowing_books, customer_name, borrow_date, rental_limit)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ReturningRequestPool {
    returning_request: Vec<ReturnBookInformation>,
}

impl ReturningRequestPool {
    pub fn new(game_data: &GameResource, today: GensoDate) -> Self {
        let mut returning_request = Vec::new();

        for _ in 1..=5 {
            let mut return_date = today.clone();

            match rand::random::<u32>() % 2 {
                0 => return_date.add_day(14),
                1 => return_date.add_day(7),
                _ => (),
            }

            returning_request.push(ReturnBookInformation::new_random(
                game_data,
                today,
                return_date,
            ));
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
    pub returning_request_pool: ReturningRequestPool,
    pub date: GensoDate,
    pub task_result: TaskResult,
    pub suzunaan_status: SuzunaAnStatus,
}

impl SavableData {
    pub fn new(game_data: &GameResource) -> Self {
        let date = GensoDate::new(112, 7, 23);

        SavableData {
            date: date.clone(),
            task_result: TaskResult::new(),
            suzunaan_status: SuzunaAnStatus::new(),
            suzuna_book_pool: SuzunaBookPool::new(game_data),
            returning_request_pool: ReturningRequestPool::new(game_data, date),
        }
    }

    pub fn save(&self, slot: u8) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(&format!("./resources/save{}.json", slot))?;

        write!(file, "{}", serde_json::to_string(self).unwrap())?;
        file.flush()?;

        Ok(())
    }

    pub fn delete(slot: u8) {
        std::fs::remove_file(&format!("./resources/save{}.json", slot)).unwrap();
    }

    pub fn new_load(slot: u8) -> Result<SavableData, ()> {
        let content = fs::read_to_string(&format!("./resources/save{}.json", slot));

        if content.is_err() {
            return Err(());
        }

        let savable_data = serde_json::from_str(&content.unwrap());

        if savable_data.is_err() {
            Err(())
        } else {
            Ok(savable_data.unwrap())
        }
    }

    pub fn replace(&mut self, data: SavableData) {
        self.date = data.date;
        self.task_result = data.task_result;
    }
}

pub struct ResultReportStringTable {
    pub total_customers_waiting_time: String,
    pub shelving_is_done: String,
}

#[derive(Clone)]
pub struct ResultReport {
    total_customers_waiting_time: Clock,
    shelving_is_done: bool,
}

impl ResultReport {
    pub fn new() -> Self {
	ResultReport {
	    total_customers_waiting_time: 0,
	    shelving_is_done: false,
	}
    }

    pub fn add_customers_waiting_time(&mut self, additional: Clock) {
	self.total_customers_waiting_time += additional;
    }

    pub fn mark_shelving_is_done(&mut self) {
	self.shelving_is_done = true;
    }

    pub fn create_table(&self) -> ResultReportStringTable {
	ResultReportStringTable::new(self)
    }
}

impl ResultReportStringTable {
    pub fn new(result_report: &ResultReport) -> Self {
	ResultReportStringTable {
	    total_customers_waiting_time: number_to_jk::number_to_jk(result_report.total_customers_waiting_time / 60),
	    shelving_is_done: if result_report.shelving_is_done { "達成" } else { "未達成" }.to_string(),
	}
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameConfig {
    bgm_volume: f32,
    se_volume: f32,
    minute_per_clock: Clock,
}

impl GameConfig {
    pub fn new_from_toml(path: &str) -> Self {
        let s = match read_whole_file(path.to_string()) {
            Ok(s) => s,
            Err(e) => panic!("Failed to read file: {}", e),
        };

        let raw_data: Result<GameConfig, toml::de::Error> = toml::from_str(&s);
        match raw_data {
            Ok(p) => p,
            Err(e) => panic!("Failed to parse toml: {}", e),
        }
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
    pub savable_data: &'ctx mut SavableData,
    pub config: &'ctx mut GameConfig,
    pub process_utility: ProcessUtility<'ctx>,
}

impl<'ctx> SuzuContext<'ctx> {
    pub fn ref_texture(&mut self, id: TextureID) -> Rc<ggraphics::Image> {
	self.resource.ref_texture(self.context, id)
    }
}

pub enum TopScene {
    ScenarioScene(scene::scenario_scene::ScenarioScene),
    SuzunaScene(scene::suzuna_scene::SuzunaScene),
    SaveScene(scene::save_scene::SaveScene),
    TitleScene(scene::title_scene::TitleScene),
    Null(scene::NullScene),
}

impl TopScene {
    pub fn abs(&self) -> &dyn scene::SceneManager {
        match self {
            TopScene::ScenarioScene(scene) => scene,
            TopScene::SuzunaScene(scene) => scene,
            TopScene::SaveScene(scene) => scene,
            TopScene::TitleScene(scene) => scene,
            TopScene::Null(scene) => scene,
        }
    }

    pub fn abs_mut(&mut self) -> &mut dyn scene::SceneManager {
        match self {
            TopScene::ScenarioScene(scene) => scene,
            TopScene::SuzunaScene(scene) => scene,
            TopScene::SaveScene(scene) => scene,
            TopScene::TitleScene(scene) => scene,
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
    game_status: SavableData,
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

        let mut game_status = SavableData::new(game_data);
        let mut game_config = GameConfig::new_from_toml("./resources/default_game_config.toml");

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
	    process_utility: ProcessUtility { redraw_request: &mut _redraw_request },
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
	    process_utility: ProcessUtility { redraw_request: &mut self.redraw_request },
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
                _ => (),
            },
	    scene::SceneID::Title => self.current_scene = TopScene::TitleScene(scene::title_scene::TitleScene::new(&mut ctx)),
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
	    process_utility: ProcessUtility { redraw_request: &mut self.redraw_request },
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
		process_utility: ProcessUtility { redraw_request: &mut self.redraw_request },
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
	    process_utility: ProcessUtility { redraw_request: &mut self.redraw_request },
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
		process_utility: ProcessUtility { redraw_request: &mut self.redraw_request },
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
		process_utility: ProcessUtility { redraw_request: &mut self.redraw_request },
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
		process_utility: ProcessUtility { redraw_request: &mut self.redraw_request },
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
		process_utility: ProcessUtility { redraw_request: &mut self.redraw_request },
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
		process_utility: ProcessUtility { redraw_request: &mut self.redraw_request },
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
		process_utility: ProcessUtility { redraw_request: &mut self.redraw_request },
            },
            numeric::Point2f::new(point.x, point.y),
            x,
            y,
        );
    }

    fn redraw_request_status(&self) -> scene::DrawRequest {
	self.redraw_request
    }
}

pub struct State<'data> {
    clock: Clock,
    fps: f64,
    scene_controller: SceneController,
    game_data: &'data mut GameResource,
}

impl<'data> ggez::event::EventHandler for State<'data> {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.scene_controller.run_pre_process(ctx, self.game_data);

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
	    },
	    _ => (),
	}

	graphics::present(ctx)?;
	
        self.scene_controller.run_post_process(ctx, self.game_data);

        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        keycode: KeyCode,
        keymods: KeyMods,
        repeat: bool,
    ) {
        self.scene_controller
            .key_down_event(ctx, self.game_data, keycode, keymods, repeat);
    }

    fn key_up_event(&mut self, ctx: &mut ggez::Context, keycode: KeyCode, keymods: KeyMods) {
        self.scene_controller
            .key_up_event(ctx, self.game_data, keycode, keymods);
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        self.scene_controller.mouse_motion_event(
            ctx,
            self.game_data,
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
            self.game_data,
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
            self.game_data,
            button,
            numeric::Point2f::new(x, y),
        );
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        self.scene_controller
            .mouse_wheel_scroll_event(ctx, self.game_data, x, y);
    }
}

impl<'data> State<'data> {
    pub fn new(ctx: &mut Context, game_data: &'data mut GameResource) -> GameResult<State<'data>> {
        let s = State {
            clock: 0,
            fps: 0.0,
	    scene_controller: SceneController::new(ctx, game_data),
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
