use hecs::World;
use resources::Resources;
use rltk::RandomNumberGenerator;

use crate::{SHOW_MAPGEN_ANIMATION, spawner};

use super::{MapBuilder, Map, apply_room_to_map, apply_horizontal_corridor,
            apply_vertical_corridor, remove_useless_walls, Rect, TileType,
            Position};


pub struct SimpleMapBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    rooms: Vec<Rect>,
    history: Vec<Map>
}

impl MapBuilder for SimpleMapBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self){
        self.rooms_and_corridors(10, 4, 8);
    }
    
    fn spawn_entities(&mut self, world: &mut World, res: &mut Resources) {
        for room in self.rooms.iter().skip(1) {
            spawner::fill_room(world, res, room, self.depth);
        }
    }

    fn get_map_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_ANIMATION {
            let mut snapshot = self.map.clone();
            for revealed in snapshot.revealed_tiles.iter_mut() {
                *revealed = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl SimpleMapBuilder {
    pub fn new(new_depth: i32) -> SimpleMapBuilder {
        SimpleMapBuilder{
            map: Map::new(new_depth),
            starting_position: Position{x: 0, y: 0},
            depth: new_depth,
            rooms: Vec::new(),
            history: Vec::new()
        }
    }

    fn rooms_and_corridors(&mut self, max_rooms: i32, min_size: i32, max_size: i32) {
        let mut rng = RandomNumberGenerator::new();

        self.take_snapshot();
        for _ in 0..max_rooms {
            let w: i32 = rng.range(min_size, max_size);
            let h: i32 = rng.range(min_size, max_size);
            let x: i32 = rng.range(1, self.map.width - w - 1);
            let y: i32 = rng.range(1, self.map.height - h - 1);

            let new_room = Rect::new(x, y, w, h);
            let mut place_room = true;

            for other_room in self.rooms.iter() {
                if new_room.intersect(&other_room) {
                    place_room = false;
                }
            }

            if place_room {
                apply_room_to_map(&mut self.map, &new_room);
                self.rooms.push(new_room);
            }
            self.take_snapshot();
        }


        for i in 1..self.rooms.len() {
            let (x1, y1) = self.rooms[i].center();
            let (x2, y2) = self.rooms[i - 1].center();

            apply_horizontal_corridor(&mut self.map, x1, x2, y1);
            apply_vertical_corridor(&mut self.map, x2, y1, y2);
            apply_vertical_corridor(&mut self.map, x1, y1, y2);
            apply_horizontal_corridor(&mut self.map, x1, x2, y2);

            self.take_snapshot();
        }
        

        let stairs_down_pos = self.rooms[self.rooms.len() - 1].center();
        let stairs_idx = self.map.xy_idx(stairs_down_pos.0, stairs_down_pos.1);
        self.map.tiles[stairs_idx] = TileType::StairsDown;

        remove_useless_walls(&mut self.map);

        let start_pos = self.rooms[0].center();
        self.starting_position = Position{x: start_pos.0, y: start_pos.1};
        self.take_snapshot();
    }
}
