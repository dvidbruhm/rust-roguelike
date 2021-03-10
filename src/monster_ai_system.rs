use legion::*;
use rltk::{Point};
use crate::components::{Position, Monster, Viewshed, Name};

#[system(for_each)]
pub fn monster_ai(mon: &Monster, pos: &Position, vs: &Viewshed, name: &Name, #[resource] ppos: &Point) {
    if vs.visible_tiles.contains(ppos) {
        println!("{} sees you!", name.name);
    }
}
