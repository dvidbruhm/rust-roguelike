use std::cmp as cmp;
use rltk;
use rltk::{RGB, RandomNumberGenerator, Rltk};
use crate::rect::{Rect};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

fn apply_room_to_map(room: &Rect, map: &mut [TileType]) {
    for y in room.y1 .. room.y2 {
        for x in room.x1 .. room.x2 {
            map[xy_idx(x, y)] = TileType::Floor;
        }
    }
}

fn apply_vertical_corridor(x:i32, y1: i32, y2:i32, map: &mut [TileType]) {
    for y in cmp::min(y1, y2)..=cmp::max(y1, y2) {
        map[xy_idx(x, y)] = TileType::Floor;
    }
}

fn apply_horizontal_corridor(x1: i32, x2:i32, y: i32, map: &mut [TileType]) {
    for x in cmp::min(x1, x2)..=cmp::max(x1, x2) {
        map[xy_idx(x, y)] = TileType::Floor;
    }
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

pub fn new_map_rooms_corridors() -> (Vec<TileType>, Vec<Rect>) {
    let mut map = vec![TileType::Wall; 80 * 50];
    let mut rng = RandomNumberGenerator::new();

    let mut rooms : Vec<Rect> = Vec::new();
    const MAX_ROOMS : i32 = 30;
    const MIN_SIZE : i32 = 4;
    const MAX_SIZE : i32 = 15;

    for _ in 0..MAX_ROOMS {
        let w : i32 = rng.range(MIN_SIZE, MAX_SIZE);
        let h : i32 = rng.range(MIN_SIZE, MAX_SIZE);
        let x : i32 = rng.range(1, 80 - w - 1);
        let y : i32 = rng.range(1, 50 - h - 1);

        let new_room = Rect::new(x, y, w, h);
        let mut place_room = true;

        for other_room in rooms.iter() {
            if new_room.intersect(&other_room) {
                place_room = false;
            }
        }

        if place_room {
            apply_room_to_map(&new_room, &mut map);
            rooms.push(new_room);
        }
    }

    for i in 1..rooms.len() {
        let (x1, y1) = rooms[i].center();
        let (x2, y2) = rooms[i - 1].center();

        apply_horizontal_corridor(x1, x2, y1, &mut map);
        apply_vertical_corridor(x2, y1, y2, &mut map);
        apply_vertical_corridor(x1, y1, y2, &mut map);
        apply_horizontal_corridor(x1, x2, y2, &mut map);
    }

    (map, rooms)
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
                ctx.set(x, y, RGB::from_f32(1.0, 1.0, 1.0), RGB::from_f32(0., 0., 0.), rltk::to_cp437('█'));
            }
        }

        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}

