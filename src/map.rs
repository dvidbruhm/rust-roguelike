use std::cmp as cmp;
use serde;
use serde::{Serialize, Deserialize};
use hecs::*;
use rltk;
use rltk::{RandomNumberGenerator, Rltk, Algorithm2D, BaseMap, Point};
use crate::{State, Palette};
use crate::rect::{Rect};


pub const MAPWIDTH: usize = 80;
pub const MAPHEIGHT: usize = 40;
pub const MAPCOUNT: usize = MAPWIDTH * MAPHEIGHT;
pub const OFFSET_X: usize = 0;
pub const OFFSET_Y: usize = 11;

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum TileType {
    Wall, Floor, StairsDown, StairsUp
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub depth: i32,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content : Vec<Vec<Entity>>
}

impl Map {
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

    pub fn is_wall(&self, x: i32, y: i32) -> bool {
        let idx = self.xy_idx(x, y);
        self.tiles[idx] == TileType::Wall
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

    pub fn transform_mouse_pos(&self, mouse_pos: (i32, i32)) -> (i32, i32) {
        (mouse_pos.0 - OFFSET_X as i32, mouse_pos.1 - OFFSET_Y as i32)
    }

    pub fn mouse_in_bounds(&self, mouse_pos: (i32, i32)) -> bool {
        mouse_pos.0 >= 0  && mouse_pos.0 <= self.width && mouse_pos.1 >= 0 && mouse_pos.1 <= self.height
    }

    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x >= self.width || y < 1 || y >= self.height { return false; }
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

    fn remove_useless_walls(&mut self) {
        let mut to_remove: Vec<(i32, i32)> = Vec::new();

        for i in 0..self.tiles.len() {
            let (x, y) = self.idx_xy(i);

            if x < 1 || x > self.width - 2 || y < 1 || y > self.height - 2 { continue }

            if self.is_wall(x - 1, y - 1) && self.is_wall(x, y - 1) && self.is_wall(x + 1, y - 1) &&
               self.is_wall(x - 1, y    ) && self.is_wall(x, y    ) && self.is_wall(x + 1, y    ) &&
               self.is_wall(x - 1, y + 1) && self.is_wall(x, y + 1) && self.is_wall(x + 1, y + 1) {
                to_remove.push((x, y));
            }
        }

        for (x, y) in to_remove {
            self.set_tile(x, y, TileType::Floor);
        }
    }

    pub fn new_map_rooms_corridors(max_rooms: i32, min_size: i32, max_size: i32, new_depth: i32) -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            rooms: Vec::new(),
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            tile_content: vec![Vec::new(); MAPCOUNT],
            depth: new_depth
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

        let stairs_down_pos = map.rooms[map.rooms.len() - 1].center();
        let stairs_idx = map.xy_idx(stairs_down_pos.0, stairs_down_pos.1);
        map.tiles[stairs_idx] = TileType::StairsDown;

        map.remove_useless_walls();

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
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
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

fn wall_glyph(map: &Map, x: i32, y: i32) -> char {
    if x < 1 || x > map.width - 2 || y < 1 || y > map.height - 2 { return 'x' }
    let mut mask: u8 = 0;

    if map.is_wall(x, y - 1) { mask += 1 }
    if map.is_wall(x, y + 1) { mask += 2 }
    if map.is_wall(x - 1, y) { mask += 4 }
    if map.is_wall(x + 1, y) { mask += 8 }

    match mask {
        0 => { '■' }
        1 => { '│' }
        2 => { '│' }
        3 => { '│' }
        4 => { '─' }
        5 => { '┘' }
        6 => { '┐' }
        7 => { '┤' }
        8 => { '─' }
        9 => { '└' }
        10 => { '┌' }
        11 => { '├' }
        12 => { '─' }
        13 => { '┴' }
        14 => { '┬' }
        15 => { '┼' }
        _ => { 'x' }
    }
}

pub fn draw_map(gs: &State, ctx : &mut Rltk) {
    let map = gs.resources.get::<Map>().unwrap();

    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;
            let mut bg;
            let (x, y) = map.idx_xy(idx);
            match tile {
                TileType::Floor => {
                    fg = Palette::COLOR_2;
                    bg = Palette::MAIN_BG;
                    glyph = rltk::to_cp437(' ');
                }
                TileType::Wall => {
                    fg = Palette::MAIN_FG;
                    bg = Palette::MAIN_BG;
                    glyph = rltk::to_cp437(wall_glyph(&map, x, y));
                }
                TileType::StairsDown => {
                    fg = Palette::MAIN_FG;
                    bg = Palette::MAIN_BG;
                    glyph = rltk::to_cp437('>');
                }
                TileType::StairsUp => {
                    fg = Palette::MAIN_FG;
                    bg = Palette::MAIN_BG;
                    glyph = rltk::to_cp437('<');
                }
            }
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale();
                bg = bg.to_greyscale();
            }
            ctx.set(x as usize + OFFSET_X, y as usize + OFFSET_Y, fg, bg, glyph);
        }
    }
}

