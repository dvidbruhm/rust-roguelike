mod simple_map;
use self::simple_map::SimpleMapBuilder;

mod common;
use common::*;
use hecs::World;
use resources::Resources;

use crate::rect::Rect;
use crate::map::{Map, TileType};
use crate::components::Position;


pub struct MapGenData {
    pub history: Vec<Map>,
    pub index: usize,
    pub timer: f32
}

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, world: &mut World, res: &mut Resources);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;
    fn get_map_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    Box::new(SimpleMapBuilder::new(new_depth))
}
