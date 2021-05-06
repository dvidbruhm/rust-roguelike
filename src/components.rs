use serde::{Serialize, Deserialize};
use hecs::*;
use rltk;

use crate::RenderOrder;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    pub always_render: bool,
    pub order: RenderOrder
}

impl Default for Renderable {
    fn default() -> Self {
        Renderable {
            glyph: rltk::to_cp437(' '),
            fg: rltk::RGB{r: 1., g: 1., b: 1.},
            bg: rltk::RGB{r: 0., g: 0., b: 0.},
            render: true,
            always_render: false,
            order: RenderOrder::Player
        }
    }
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
    pub power: i32,
    pub regen_rate: i32,
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

pub struct WantsToUnequipItem {
    pub item: Entity
}

pub struct Item {}

#[derive(PartialEq, Copy, Clone)]
pub enum EquipmentSlot { RightHand, LeftHand }

#[derive(Copy, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot
}

pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot
}

pub struct InBackpack {
    pub owner: Entity
}

pub struct Consumable {}

pub struct MeleePowerBonus {
    pub power: i32
}

pub struct MeleeDefenseBonus {
    pub defense: i32
}

#[derive(Clone, Copy)]
pub struct ProvidesHealing {
    pub heal: i32
}

pub struct Ranged {
    pub range: i32
}

#[derive(Clone, Copy)]
pub struct DealsDamage {
    pub damage: i32
}

#[derive(Clone, Copy)]
pub struct Confusion {
    pub turns: i32
}

pub struct AreaOfEffect {
    pub radius: i32
}

pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<rltk::Point>
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

pub struct SerializeMe {}

pub struct Lifetime {
    pub ms: f32
}

pub struct Velocity {
    pub x: f32,
    pub y: f32
}

pub struct Particle {
    pub float_x: f32,
    pub float_y: f32
}
