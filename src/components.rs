use rltk;

pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: rltk::RGB,
    pub bg: rltk::RGB,
}

pub struct Player {}

pub struct Droplet {}
