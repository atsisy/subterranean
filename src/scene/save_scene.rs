use torifune::core::Clock;
use torifune::device as tdev;
use torifune::graphics::drawable::*;
use torifune::graphics::object::*;

use crate::core::{SavableData, SuzuContext, TextureID};
use crate::scene::*;
use crate::object::save_scene_object::*;

pub struct SaveScene {
    background: UniTexture,
    save_entry_table: SaveEntryTable,
    scene_transition: SceneID,
    clock: Clock,
}

impl SaveScene {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>) -> Self {
	let save_data_list = (1..=4)
	    .map(|slot_index|
		 match SavableData::new_load(slot_index) {
		     Ok(savable_data) => Some(savable_data) ,
		     Err(_) => None,
		 })
	    .collect();
	    
	let save_entry_table = SaveEntryTable::new(
	    ctx,
	    numeric::Rect::new(50.0, 50.0, 1266.0, 668.0),
	    save_data_list,
	    0
	);

	let background = UniTexture::new(
	    ctx.resource.ref_texture(TextureID::JpHouseTexture),
	    numeric::Point2f::new(0.0, 0.0),
	    numeric::Vector2f::new(0.7, 0.7),
	    0.0,
	    0
	);
	
        SaveScene {
	    background: background,
	    save_entry_table: save_entry_table,
            scene_transition: SceneID::Save,
            clock: 0,
        }
    }
}

impl SceneManager for SaveScene {
    fn key_down_event<'a>(&mut self, _ctx: &mut SuzuContext, _vkey: tdev::VirtualKey) {
    }

    fn pre_process<'a>(&mut self, _ctx: &mut SuzuContext<'a>) {
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
	self.background.draw(ctx).unwrap();
	self.save_entry_table.draw(ctx).unwrap();
    }

    fn post_process<'a>(&mut self, _ctx: &mut SuzuContext<'a>) -> SceneTransition {
        self.update_current_clock();

        SceneTransition::Keep
    }

    fn transition(&self) -> SceneID {
        self.scene_transition
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }

    fn update_current_clock(&mut self) {
        self.clock += 1;
    }
}
