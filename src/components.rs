use legion::*;
use rltk;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: rltk::RGB,
    pub bg: rltk::RGB,
    pub render: bool
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Player {}

#[derive(Clone, Debug, PartialEq)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Monster {}


#[derive(Clone, Debug, PartialEq)]
pub struct Name {
    pub name: String
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BlocksTile {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32
}

pub struct WantsToAttack {
    pub target: Entity
}

pub struct TakeDamage {
    pub amount: Vec<i32>
}

impl TakeDamage {
    fn add_damage(td: &mut TakeDamage, victim: Entity, amount: i32) {
        //todo
    }
}

pub struct Droplet {}
