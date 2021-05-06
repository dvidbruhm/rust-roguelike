use std::collections::HashMap;

use hecs::*;
use resources::*;
use rltk::RandomNumberGenerator;
use crate::components::{AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, DealsDamage, EquipmentSlot, Equippable, Item, MeleeDefenseBonus, MeleePowerBonus, Monster, Name, Player, Position, ProvidesHealing, Ranged, Renderable, SerializeMe, Viewshed};
use crate::{Palette, RenderOrder};
use crate::rect::Rect;
use crate::weighted_table::WeightedTable;

const MAX_SPAWNS: i32 = 8;


pub fn player(world: &mut World, pos: (i32, i32)) -> Entity {
    world.spawn((
        SerializeMe {},
        Position {x: pos.0, y: pos.1},
        Renderable {
            glyph: rltk::to_cp437('ô'),
            fg: Palette::COLOR_0,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Player,
            ..Default::default()
        },
        Player {},
        Viewshed {
            visible_tiles: Vec::new(),
            range: 20,
            dirty: true
        },
        Name {name: "Blabinou".to_string()},
        CombatStats {max_hp: 30, hp: 30, defense: 2, power: 5, regen_rate: 1}
    ))
}

pub fn room_table(depth: i32) -> WeightedTable {
    WeightedTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + depth)
        .add("Confusion Scroll", 2 + depth)
        .add("Magic Missile Scroll", 4)
        .add("Dagger", 2)
        .add("Shield", 2)
        .add("Longsword", depth - 1)
        .add("Tower Shield", depth - 1)
}

pub fn fill_room(world: &mut World, res: &mut Resources, room: &Rect, depth: i32) {
    let spawn_table = &room_table(depth);
    let mut spawn_points: HashMap<(i32, i32), String> = HashMap::new();
    {
        let mut rng = &mut res.get_mut::<RandomNumberGenerator>().unwrap();
        let nb_spawns = rng.range(-2, MAX_SPAWNS + depth);

        for _i in 0..nb_spawns {
            let mut added = false;
            while !added {
                let x = rng.range(room.x1, room.x1 + room.width());
                let y = rng.range(room.y1, room.y1 + room.height());
                if !spawn_points.contains_key(&(x, y)) {
                    spawn_points.insert((x, y), spawn_table.roll(&mut rng).unwrap());
                    added = true;
                }
            }
        }
    }

    for spawn in spawn_points.iter() {
        let (x, y) = *spawn.0;
        match spawn.1.as_ref() {
            "Goblin" => goblin(world, x, y),
            "Orc" => orc(world, x, y),
            "Health Potion" => health_potion(world, x, y),
            "Fireball Scroll" => fireball_scroll(world, x, y),
            "Confusion Scroll" => confusion_scroll(world, x, y),
            "Magic Missile Scroll" => magic_missile_scroll(world, x, y),
            "Dagger" => dagger(world, x, y),
            "Shield" => shield(world, x, y),
            "Longsword" => longsword(world, x, y),
            "Tower Shield" => tower_shield(world, x, y),
            _ => {}
        }
    }
}

fn orc(world: &mut World, x: i32, y:i32) {
    monster(world, x, y, rltk::to_cp437('o'), "Orc".to_string());
}

fn goblin(world: &mut World, x: i32, y:i32) {
    monster(world, x, y, rltk::to_cp437('g'), "Goblin".to_string());
}

fn monster(world: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: String) {
    world.spawn((
        Position {x, y},
        Renderable {
            glyph,
            fg: Palette::COLOR_1,
            bg: Palette::MAIN_BG,
            order: RenderOrder::NPC,
            ..Default::default()
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true
        },
        Monster {},
        Name {name},
        BlocksTile {},
        CombatStats {max_hp: 8, hp: 8, defense: 1, power: 4, regen_rate: 0}
    ));
}

fn health_potion(world: &mut World, x: i32, y:i32) {
    world.spawn((
        Position {x, y},
        Renderable {
            glyph: rltk::to_cp437('p'),
            fg: Palette::COLOR_4,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Health potion".to_string()},
        Item {},
        ProvidesHealing { heal: 8 },
        Consumable {}
    ));
}

fn magic_missile_scroll(world: &mut World, x: i32, y:i32) {
    world.spawn((
        Position {x, y},
        Renderable {
            glyph: rltk::to_cp437('('),
            fg: Palette::COLOR_4,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Magic missile scroll".to_string()},
        Item {},
        Consumable {},
        DealsDamage {damage: 8},
        Ranged {range:6}
    ));
}

fn fireball_scroll(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position {x, y},
        Renderable {
            glyph: rltk::to_cp437('*'),
            fg: Palette::COLOR_4,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Fireball scroll".to_string()},
        Item {},
        Consumable {},
        DealsDamage {damage: 20},
        Ranged {range: 6},
        AreaOfEffect {radius: 3}
    ));
}

fn confusion_scroll(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position {x, y},
        Renderable {
            glyph: rltk::to_cp437('&'),
            fg: Palette::COLOR_4,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Confusion scroll".to_string()},
        Item {},
        Consumable {},
        Ranged {range: 6},
        Confusion {turns: 4}
    ));
}

fn dagger(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position {x, y},
        Renderable {
            glyph: rltk::to_cp437('│'),
            fg: Palette::COLOR_3,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Dagger".to_string()},
        Item {},
        Equippable {slot: EquipmentSlot::RightHand},
        MeleePowerBonus {power: 4}
    ));
}

fn longsword(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position {x, y},
        Renderable {
            glyph: rltk::to_cp437('│'),
            fg: Palette::COLOR_3,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Dagger".to_string()},
        Item {},
        Equippable {slot: EquipmentSlot::RightHand},
        MeleePowerBonus {power: 8}
    ));
}

fn shield(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position {x, y},
        Renderable {
            glyph: rltk::to_cp437('°'),
            fg: Palette::COLOR_4,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Shield".to_string()},
        Item {},
        Equippable {slot: EquipmentSlot::LeftHand},
        MeleeDefenseBonus {defense: 4}
    ));
}

fn tower_shield(world: &mut World, x: i32, y: i32) {
    world.spawn((
        Position {x, y},
        Renderable {
            glyph: rltk::to_cp437('°'),
            fg: Palette::COLOR_4,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Shield".to_string()},
        Item {},
        Equippable {slot: EquipmentSlot::LeftHand},
        MeleeDefenseBonus {defense: 8}
    ));
}
