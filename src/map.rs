use std::cmp as cmp;
use legion::*;
use rltk;
use rltk::{RGB, RandomNumberGenerator, Rltk, Algorithm2D, BaseMap, Point};
use crate::{State};
use crate::rect::{Rect};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TileType {
    Wall, Floor
}

pub struct Map {
    pub tiles : Vec<TileType>,
    pub rooms : Vec<Rect>,
    pub width : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
    pub blocked : Vec<bool>,
    pub tile_content : Vec<Vec<Entity>>
}

impl Map {
    pub fn get_tile(&self, x: i32, y: i32) -> TileType {
        self.tiles[self.xy_idx(x, y)]
    }
    
    pub fn set_tile(&mut self, x: i32, y: i32, value: TileType) {
        let idx = self.xy_idx(x, y);
        self.tiles[idx] = value;
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }
    
    pub fn idx_xy(&self, idx: usize) -> (i32, i32) {
        (idx as i32 % self.width, idx as i32 / self.width)
    }

    pub fn set_blocked(&mut self) {
        for (i, t) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *t == TileType::Wall;
        }
    }
    
    pub fn clear_tile_content(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height { return false; }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 .. room.y2 {
            for x in room.x1 .. room.x2 {
                self.set_tile(x, y, TileType::Floor);
            }
        }
    }

    fn apply_vertical_corridor(&mut self, x:i32, y1: i32, y2:i32) {
        for y in cmp::min(y1, y2)..=cmp::max(y1, y2) {
            self.set_tile(x, y, TileType::Floor);
        }
    }

    fn apply_horizontal_corridor(&mut self, x1: i32, x2:i32, y: i32) {
        for x in cmp::min(x1, x2)..=cmp::max(x1, x2) {
            self.set_tile(x, y, TileType::Floor);
        }
    }

    pub fn new_map_rooms_corridors(max_rooms: i32, min_size: i32, max_size: i32) -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; 80 * 50],
            rooms: Vec::new(),
            width: 80,
            height: 50,
            revealed_tiles: vec![false; 80 * 50],
            visible_tiles: vec![false; 80 * 50],
            blocked: vec![false; 80 * 50],
            tile_content: vec![Vec::new(); 80 * 50]
        };
        let mut rng = RandomNumberGenerator::new();

        for _ in 0..max_rooms {
            let w : i32 = rng.range(min_size, max_size);
            let h : i32 = rng.range(min_size, max_size);
            let x : i32 = rng.range(1, map.width - w - 1);
            let y : i32 = rng.range(1, map.height - h - 1);

            let new_room = Rect::new(x, y, w, h);
            let mut place_room = true;

            for other_room in map.rooms.iter() {
                if new_room.intersect(&other_room) {
                    place_room = false;
                }
            }

            if place_room {
                map.apply_room_to_map(&new_room);
                map.rooms.push(new_room);
            }
        }

        for i in 1..map.rooms.len() {
            let (x1, y1) = map.rooms[i].center();
            let (x2, y2) = map.rooms[i - 1].center();

            map.apply_horizontal_corridor(x1, x2, y1);
            map.apply_vertical_corridor(x2, y1, y2);
            map.apply_vertical_corridor(x1, y1, y2);
            map.apply_horizontal_corridor(x1, x2, y2);
        }

        map
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let (x1, y1) = self.idx_xy(idx1);
        let (x2, y2) = self.idx_xy(idx2);
        let p1 = Point::new(x1, y1);
        let p2 = Point::new(x2, y2);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let (x, y) = self.idx_xy(idx);
        let w = self.width as usize;

        if self.is_exit_valid(x - 1, y) { exits.push((idx - 1, 1.0)) };
        if self.is_exit_valid(x + 1, y) { exits.push((idx + 1, 1.0)) };
        if self.is_exit_valid(x, y - 1) { exits.push((idx - w, 1.0)) };
        if self.is_exit_valid(x, y + 1) { exits.push((idx + w, 1.0)) };

        if self.is_exit_valid(x - 1, y - 1) { exits.push((idx - w - 1, 1.45)) };
        if self.is_exit_valid(x + 1, y - 1) { exits.push((idx - w + 1, 1.45)) };
        if self.is_exit_valid(x - 1, y + 1) { exits.push((idx + w - 1, 1.45)) };
        if self.is_exit_valid(x + 1, y + 1) { exits.push((idx + w + 1, 1.45)) };

        exits
    }
}

pub fn draw_map(gs: &State, ctx : &mut Rltk) {
    let map = gs.ecs.resources.get::<Map>().unwrap();
    let mut y = 0;
    let mut x = 0;

    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;
            let mut bg;
            match tile {
                TileType::Floor => {
                    fg = RGB::from_f32(0., 0., 0.);
                    bg = RGB::from_f32(0.2, 0.1, 0.);
                    glyph = rltk::to_cp437(' ');
                }
                TileType::Wall => {
                    fg = RGB::from_f32(0.6, 0.3, 0.0);
                    bg = RGB::from_f32(0., 0.1, 0.);
                    glyph = rltk::to_cp437('█');
                }
            }
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale();
                bg = bg.to_greyscale();
            }
            ctx.set(x, y, fg, bg, glyph);
        }
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}

