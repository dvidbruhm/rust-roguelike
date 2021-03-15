use hecs::*;
use resources::Resources;
use rltk;
use rltk::{Point};
use crate::{RunState};
use crate::components::{Position, Monster, Viewshed, WantsToAttack};
use crate::map::{Map};


//pub fn monster_ai(sworld: &mut SubWorld, #[resource] ppos: &Point, #[resource] map: &mut Map) {
pub fn monster_ai(world: &mut World, res: &mut Resources) {
    // Check if it is the monsters turn
    let runstate: &RunState = &res.get::<RunState>().unwrap();
    if *runstate != RunState::MonsterTurn { return; }
    

    let player_id: &Entity = &res.get::<Entity>().unwrap();
    let map: &mut Map = &mut res.get_mut::<Map>().unwrap();
    let ppos: &Point = &res.get::<Point>().unwrap();

    let mut needs_wants_to_attack: Vec<Entity> = vec![];

    // Monster ai
    for (id, (_mon, pos, vs)) in world.query::<(&Monster, &mut Position, &mut Viewshed)>().iter() {

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
                println!("moving");
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
        let _res = world.insert_one(*id, WantsToAttack {target: *player_id});
    }
}
