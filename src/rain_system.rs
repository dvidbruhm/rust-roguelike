use rltk;
use legion::*;
use crate::components::{Position, Droplet};

#[system(for_each)]
pub fn rain(pos: &mut Position, _rm: &Droplet) {
    let mut rng = rltk::RandomNumberGenerator::new();
    pos.x += -1;
    pos.y += 2;

    if pos.y > 49 || rng.range(0, 100) < 3 {
        pos.y = rng.range(-50, 0);
        pos.x = rng.range(0, 105);
    }
}
