use hecs::*;
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
    pub render: bool,
    pub order: i32
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

#[derive(Clone, Copy)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity
}

#[derive(Clone, Copy)]
pub struct WantsToDropItem {
    pub item: Entity
}

pub struct Item {}

pub struct InBackpack {
    pub owner: Entity
}

pub struct Potion {
    pub heal: i32
}

pub struct WantsToDrinkPotion {
    pub potion: Entity
}

pub struct TakeDamage {
    pub amount: Vec<i32>
}

impl TakeDamage {
    pub fn add_damage(world: &mut World, victim: Entity, amount: i32) {
        let mut needs_take_damage = false;

        {
            let take_damage = world.get_mut::<TakeDamage>(victim);
            match take_damage {
                Ok(mut take_dmg) => {
                    take_dmg.amount.push(amount);
                },
                Err(_e) => {
                    needs_take_damage = true;
                }
            }
        }

        if needs_take_damage {
            let _res = world.insert_one(victim, TakeDamage{amount: vec![amount]});
        }
    }
}
