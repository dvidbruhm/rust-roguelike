use legion::*;
use rltk;
use rltk::{Point};
use crate::map::{Map};
use crate::components::{Position, Viewshed, Player};

#[system(for_each)]
pub fn visibility(pos: &Position, vs: &mut Viewshed, player: Option<&Player>, #[resource] map: &mut Map) {
    if vs.dirty {
        vs.dirty = false;
        vs.visible_tiles.clear();
        vs.visible_tiles = rltk::field_of_view(Point::new(pos.x, pos.y), vs.range, map);
        vs.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

        if let Some(_player) = player {
            for t in map.visible_tiles.iter_mut() {
                *t = false;
            }
            for vis in vs.visible_tiles.iter() {
                let idx = map.xy_idx(vis.x, vis.y);
                map.revealed_tiles[idx] = true;
                map.visible_tiles[idx] = true;
            }
        }
    }
}
