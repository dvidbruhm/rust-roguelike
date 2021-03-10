use rltk;
use legion::*;
use crate::components::{Position, Droplet, Renderable};
use crate::map::{TileType, Map};

#[system(for_each)]
pub fn rain(pos: &mut Position, ren: &mut Renderable, _rm: &Droplet, #[resource] map: &Map) {
    let mut rng = rltk::RandomNumberGenerator::new();
    pos.x += -1;
    pos.y += 2;

    if pos.y > 49 || rng.range(0, 100) < 3 {
        pos.y = rng.range(-50, 0);
        pos.x = rng.range(0, 105);
    }
    if pos.x < 80 && pos.x >= 0 && pos.y < 50 && pos.y >= 0 {
        let idx = map.xy_idx(pos.x, pos.y);
        if map.revealed_tiles[idx] {
            if map.get_tile(pos.x, pos.y) == TileType::Wall {
                ren.render = false;
            }
            else if map.get_tile(pos.x, pos.y) == TileType::Floor {
                ren.render = true;
                ren.bg = rltk::RGB::named(rltk::BLACK);
            }
        }
        else {
            ren.render = false;
        }
    }
}
