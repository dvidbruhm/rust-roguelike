use rltk;
use legion::*;
use crate::components::{Position, Droplet, Renderable};
use crate::map::{TileType, xy_idx};

#[system(for_each)]
pub fn rain(pos: &mut Position, ren: &mut Renderable, _rm: &Droplet, #[resource] map: &Vec<TileType>) {
    let mut rng = rltk::RandomNumberGenerator::new();
    pos.x += -1;
    pos.y += 2;

    if pos.y > 49 || rng.range(0, 100) < 3 {
        pos.y = rng.range(-50, 0);
        pos.x = rng.range(0, 105);
    }
    if pos.x < 80 && pos.x >= 0 && pos.y < 50 && pos.y >= 0 {
        if map[xy_idx(pos.x, pos.y)] == TileType::Wall {
            ren.bg = rltk::RGB::named(rltk::WHITE);
        }
        else if map[xy_idx(pos.x, pos.y)] == TileType::Floor {
            ren.bg = rltk::RGB::named(rltk::BLACK);
        }
    }
}
