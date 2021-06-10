use serde;
use serde::{Serialize, Deserialize};
use hecs::*;
use rltk;
use rltk::{Rltk, Algorithm2D, BaseMap, Point};
use crate::{State, Palette};


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
    pub fn new(new_depth: i32) -> Map {
        Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            tile_content: vec![Vec::new(); MAPCOUNT],
            depth: new_depth
        }
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

