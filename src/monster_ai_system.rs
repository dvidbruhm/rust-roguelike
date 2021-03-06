use hecs::*;
use resources::Resources;
use rltk;
use rltk::Point;
use crate::{Palette, RunState, particle_system::ParticleBuilder};
use crate::components::{Position, Monster, Viewshed, WantsToAttack, Confusion};
use crate::map::Map;


//pub fn monster_ai(sworld: &mut SubWorld, #[resource] ppos: &Point, #[resource] map: &mut Map) {
pub fn monster_ai(world: &mut World, res: &mut Resources) {
    // Check if it is the monsters turn
    let runstate: &RunState = &res.get::<RunState>().unwrap();
    if *runstate != RunState::MonsterTurn { return; }
    

    let player_id: &Entity = &res.get::<Entity>().unwrap();
    let map: &mut Map = &mut res.get_mut::<Map>().unwrap();
    let ppos: &Point = &res.get::<Point>().unwrap();
    let mut particle_builder = res.get_mut::<ParticleBuilder>().unwrap();

    let mut needs_wants_to_attack: Vec<Entity> = Vec::new();
    let mut to_update_confusion: Vec<(Entity, Confusion)> = Vec::new();

    // Monster ai
    for (id, (_mon, pos, vs)) in world.query::<(&Monster, &mut Position, &mut Viewshed)>().iter() {
        match world.get_mut::<Confusion>(id) {
            Err(_e) => {},
            Ok(confusion) => {
                to_update_confusion.push((id, *confusion));
                particle_builder.request(pos.x, pos.y, 0.0, 0.0, Palette::COLOR_3, Palette::MAIN_BG, rltk::to_cp437('?'), 300.0);
                continue;
            }
        }

        let distance = rltk::DistanceAlg::Pythagoras.distance2d(*ppos, Point::new(pos.x, pos.y));
        if distance < 1.5 {
            needs_wants_to_attack.push(id);
        } else if vs.visible_tiles.contains(&*ppos){
            let path = rltk::a_star_search(
                map.xy_idx(pos.x, pos.y) as i32,
                map.xy_idx(ppos.x, ppos.y) as i32,
                &mut *map
            );

            if path.success && path.steps.len() > 1 {
                let (new_x, new_y) = map.idx_xy(path.steps[1]);
                let mut idx = map.xy_idx(pos.x, pos.y);
                map.blocked[idx] = false;
                pos.x = new_x;
                pos.y = new_y;
                idx = map.xy_idx(pos.x, pos.y);
                map.blocked[idx] = true;
                vs.dirty = true;
            }
        }
    }

    for id in needs_wants_to_attack.iter() {
        world.insert_one(*id, WantsToAttack {target: *player_id}).unwrap();
    }

    for (id, _confusion) in to_update_confusion.iter() {
        let mut to_remove = false;
        {
            let mut c = world.get_mut::<Confusion>(*id).unwrap();
            c.turns -= 1;
            if c.turns <= 0 { to_remove = true }
        }
        if to_remove { world.remove_one::<Confusion>(*id).unwrap(); }
    }
}
