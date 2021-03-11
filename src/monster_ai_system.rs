use legion::*;
use legion::world::SubWorld;
use rltk;
use rltk::{Point};
use crate::components::{Position, Monster, Viewshed, Player, Name};
use crate::map::{Map};

#[system]
#[read_component(Monster)]
#[write_component(Position)]
#[write_component(Viewshed)]
#[read_component(Name)]
#[read_component(Player)]
pub fn monster_ai(sworld: &mut SubWorld, #[resource] ppos: &Point, #[resource] map: &mut Map) {
    // Find player name to shout insults
    let player_name = <(&Player, &Name)>::query().iter(sworld).next().unwrap().1.name.to_string();

    // Monster ai
    let mut query = <(&Monster, &mut Position, &mut Viewshed, &Name)>::query();
    for (_mon, pos, vs, name) in query.iter_mut(sworld) {
        if vs.visible_tiles.contains(ppos) {

            let distance = rltk::DistanceAlg::Pythagoras.distance2d(*ppos, Point::new(pos.x, pos.y));
            if distance < 1.5 {
                println!("{} shouts insults at {}!", name.name, player_name);
                return;
            }

            let path = rltk::a_star_search(
                map.xy_idx(pos.x, pos.y) as i32,
                map.xy_idx(ppos.x, ppos.y) as i32,
                &mut *map
            );

            if path.success && path.steps.len() > 1 {
                let (new_x, new_y) = map.idx_xy(path.steps[1]);
                pos.x = new_x;
                pos.y = new_y;
                vs.dirty = true;
            }
        }
    }
}
