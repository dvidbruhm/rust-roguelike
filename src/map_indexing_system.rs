use legion::*;
use legion::world::{SubWorld};
use crate::map::{Map};
use crate::components::{BlocksTile, Position};


#[system]
#[read_component(BlocksTile)]
#[read_component(Position)]
pub fn map_indexing(sworld: &SubWorld, #[resource] map: &mut Map) {
    let mut query = <(Entity, Option<&BlocksTile>, &Position)>::query();

    map.set_blocked();
    map.clear_tile_content();

    for (ent, _bt, pos) in query.iter(sworld) {
        let idx = map.xy_idx(pos.x, pos.y);

        if let Some(_bt) = _bt {
            map.blocked[idx] = true;
        }

        map.tile_content[idx].push(*ent);
    }
}
