use super::{Map, Rect, TileType};
use std::cmp;


pub fn apply_room_to_map(map: &mut Map, room: &Rect) {
    for y in room.y1 .. room.y2 {
        for x in room.x1 .. room.x2 {
            map.set_tile(x, y, TileType::Floor);
        }
    }
}

pub fn apply_vertical_corridor(map: &mut Map, x:i32, y1: i32, y2:i32) {
    for y in cmp::min(y1, y2)..=cmp::max(y1, y2) {
        map.set_tile(x, y, TileType::Floor);
    }
}

pub fn apply_horizontal_corridor(map: &mut Map, x1: i32, x2:i32, y: i32) {
    for x in cmp::min(x1, x2)..=cmp::max(x1, x2) {
        map.set_tile(x, y, TileType::Floor);
    }
}


pub fn remove_useless_walls(map: &mut Map) {
    let mut to_remove: Vec<(i32, i32)> = Vec::new();

    for i in 0..map.tiles.len() {
        let (x, y) = map.idx_xy(i);

        if x < 1 || x > map.width - 2 || y < 1 || y > map.height - 2 { continue }

        if map.is_wall(x - 1, y - 1) && map.is_wall(x, y - 1) && map.is_wall(x + 1, y - 1) &&
           map.is_wall(x - 1, y    ) && map.is_wall(x, y    ) && map.is_wall(x + 1, y    ) &&
           map.is_wall(x - 1, y + 1) && map.is_wall(x, y + 1) && map.is_wall(x + 1, y + 1) {
            to_remove.push((x, y));
        }
    }

    for (x, y) in to_remove {
        map.set_tile(x, y, TileType::Floor);
    }
}
