use rltk;
use rltk::{RGB, RandomNumberGenerator, Rltk};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

pub fn new_random_map() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; 80 * 50];
    
    let mut rng = RandomNumberGenerator::new();
    
    for _i in 0..400 {
        let idx = xy_idx(rng.range(1, 79), rng.range(1, 49));
        map[idx] = TileType::Wall;
    }

    map
}

pub fn draw_map(map: &[TileType], ctx : &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        match tile {
            TileType::Floor => {
                ctx.set(x, y, RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.), rltk::to_cp437(' '));
            }
            TileType::Wall => {
                ctx.set(x, y, RGB::from_f32(1.0, 1.0, 1.0), RGB::from_f32(0., 0., 0.), rltk::to_cp437('#'));
            }
        }

        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}

