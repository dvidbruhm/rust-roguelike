use rltk;

pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: rltk::RGB,
    pub bg: rltk::RGB,
    pub render: bool
}

pub struct Player {}

pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool
}

pub struct Droplet {}
