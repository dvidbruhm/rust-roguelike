use hecs::*;
use resources::*;
use rltk::{RandomNumberGenerator};
use crate::components::{Position, Renderable, Player, Viewshed, Name, CombatStats, BlocksTile, Monster, Item, Potion};
use crate::{Palette};
use crate::rect::{Rect};

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;


pub fn player(world: &mut World, pos: (i32, i32)) -> Entity {
    world.spawn((
        Position {x: pos.0, y: pos.1},
        Renderable {
            glyph: rltk::to_cp437('ô'),
            fg: Palette::COLOR_0,
            bg: Palette::MAIN_BG,
            render: true,
            order: 0
        },
        Player {},
        Viewshed {
            visible_tiles: Vec::new(),
            range: 20,
            dirty: true
        },
        Name {name: "Blabinou".to_string()},
        CombatStats {max_hp: 30, hp: 30, defense: 2, power: 5}
    ))
}

pub fn random_monster(world: &mut World, res: &mut Resources, x: i32, y: i32) {
    let rng = &mut res.get_mut::<RandomNumberGenerator>().unwrap();
    let roll = rng.range(0, 2);
    match roll {
        0 => { orc(world, x, y); }
        _ => { goblin(world, x, y); }
    }
}

pub fn fill_room(world: &mut World, res: &mut Resources, room: &Rect) {
    let mut monster_spawn_points: Vec<(i32, i32)> = Vec::new();
    let mut item_spawn_points: Vec<(i32, i32)> = Vec::new();
    {
        let rng = &mut res.get_mut::<RandomNumberGenerator>().unwrap();
        let nb_monsters = rng.range(0, MAX_MONSTERS + 1);
        let nb_items = rng.range(0, MAX_ITEMS + 1);

        for _i in 0..nb_monsters {
            let mut added = false;
            while !added {
                let x = rng.range(room.x1, room.x1 + room.width());
                let y = rng.range(room.y1, room.y1 + room.height());
                if !monster_spawn_points.contains(&(x, y)) {
                    monster_spawn_points.push((x, y));
                    added = true;
                }
            }
        }

        for _i in 0..nb_items {
            let mut added = false;
            while !added {
                let x = rng.range(room.x1, room.x1 + room.width());
                let y = rng.range(room.y1, room.y1 + room.height());
                if !item_spawn_points.contains(&(x, y)) {
                    item_spawn_points.push((x, y));
                    added = true;
                }
            }
        }
    }

    for (x, y) in monster_spawn_points.iter() {
        random_monster(world, res, *x, *y);
    }
    for (x, y) in item_spawn_points.iter() {
        health_potion(world, *x, *y);
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
            glyph: glyph,
            fg: Palette::COLOR_1,
            bg: Palette::MAIN_BG,
            render: true,
            order: 1
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true
        },
        Monster {},
        Name {name: name},
        BlocksTile {},
        CombatStats {max_hp: 8, hp: 8, defense: 1, power: 4}
    ));
}

fn health_potion(world: &mut World, x: i32, y:i32) {
    world.spawn((
        Position {x, y},
        Renderable {
            glyph: rltk::to_cp437('p'),
            fg: Palette::COLOR_4,
            bg: Palette::MAIN_BG,
            render: true,
            order: 2
        },
        Name {name: "Health potion".to_string()},
        Item {},
        Potion { heal: 8 }
    ));
}
