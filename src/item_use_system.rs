use hecs::*;
use resources::*;
use crate::{Palette, components::Position, gamelog::GameLog, particle_system::ParticleBuilder};
use crate::components::{WantsToUseItem, CombatStats, ProvidesHealing, Name, Consumable, DealsDamage, TakeDamage, AreaOfEffect, Confusion, Equippable, Equipped, InBackpack};
use crate::map::Map;

pub fn item_use(world: &mut World, res: &mut Resources) {
    let mut log = res.get_mut::<GameLog>().unwrap();
    let player_id = res.get::<Entity>().unwrap();
    let map = res.get::<Map>().unwrap();
    let mut p_builder = res.get_mut::<ParticleBuilder>().unwrap();
    let mut to_remove: Vec<Entity> = Vec::new();
    let mut to_remove_wants_use: Vec<Entity> = Vec::new();
    let mut to_add_take_damage: Vec<(Entity, DealsDamage)> = Vec::new();
    let mut to_heal: Vec<(Entity, ProvidesHealing)> = Vec::new();
    let mut to_add_confusion: Vec<(Entity, Confusion)> = Vec::new();
    let mut to_unequip: Vec<(Entity, Name, Entity)> = Vec::new();
    let mut to_equip: Vec<(Entity, Equippable, Name, Entity)> = Vec::new();

    for (id, use_item) in &mut world.query::<&WantsToUseItem>().iter() {
        let mut used_item = true;

        // Find all targets
        let mut targets: Vec<Entity> = Vec::new();
        match use_item.target {
            None => targets.push(*player_id),
            Some(t) => {
                match world.get::<AreaOfEffect>(use_item.item) {
                    Err(_e) => {
                        // Single target
                        let idx = map.xy_idx(t.x, t.y);
                        for entity in map.tile_content[idx].iter() {
                            targets.push(*entity);
                        }
                    }
                    Ok(aoe) => {
                        // AOE
                        let mut affected_tiles = rltk::field_of_view(t, aoe.radius, &*map);
                        affected_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1);
                        for pt in affected_tiles.iter() {
                            let idx = map.xy_idx(pt.x, pt.y);
                            for entity in map.tile_content[idx].iter() {
                                targets.push(*entity);
                            }
                            p_builder.request(pt.x, pt.y, 0.0, 0.0, Palette::COLOR_3, Palette::MAIN_BG, rltk::to_cp437('o'), 250.0)
                        }
                    }
                }
            }
        }

        // Apply heal if it provides healing
        let item_heals = world.get::<ProvidesHealing>(use_item.item);
        match item_heals {
            Err(_e) => {}
            Ok(healer) => {
                used_item = false;
                for target in targets.iter() {
                    let stats = world.get_mut::<CombatStats>(*target);
                    match stats {
                        Err(_e) => {},
                        Ok(_stats) => {
                            to_heal.push((*target, *healer));
                            if id == *player_id {
                                let name = world.get::<Name>(use_item.item).unwrap();
                                log.messages.push(format!("You use the {}, healing {} hp", name.name, healer.heal));
                            }
                            used_item = true;

                            if let Ok(pos) = world.get::<Position>(*target) {
                                p_builder.request(pos.x, pos.y, 0.0, -3.0, Palette::COLOR_3, Palette::MAIN_BG, rltk::to_cp437('♥'), 1000.0)
                            }
                        }
                    }
                }
            }
        }
        to_remove_wants_use.push(id);

        // Apply damage to target if it deals damage
        let deals_damage = world.get::<DealsDamage>(use_item.item);
        match deals_damage {
            Err(_e) => {}
            Ok(dd) => {
                used_item = false;
                for target in targets.iter() {
                    to_add_take_damage.push((*target, *dd));
                    if id == *player_id {
                        let monster_name = world.get::<Name>(*target).unwrap();
                        let item_name = world.get::<Name>(use_item.item).unwrap();
                        log.messages.push(format!("You use {} on {}, dealing {} hp", item_name.name, monster_name.name, dd.damage));
                    }
                    used_item = true;

                    if let Ok(pos) = world.get::<Position>(*target) {
                        p_builder.request(pos.x, pos.y, 0.0, 0.0, Palette::COLOR_4, Palette::MAIN_BG, rltk::to_cp437('‼'), 250.0)
                    }
                }
            }
        }

        // Apply confusion
        let confusion = world.get::<Confusion>(use_item.item);
        match confusion {
            Err(_e) => {},
            Ok(confusion) => {
                used_item = false;
                for target in targets.iter() {
                    to_add_confusion.push((*target, *confusion));
                    if id == *player_id {
                        let monster_name = world.get::<Name>(*target).unwrap();
                        let item_name = world.get::<Name>(use_item.item).unwrap();
                        log.messages.push(format!("You use {} on {}, confusing them", item_name.name, monster_name.name));
                    }
                    used_item = true;

                    if let Ok(pos) = world.get::<Position>(*target) {
                        p_builder.request(pos.x, pos.y, 0.0, 0.0, Palette::COLOR_3, Palette::MAIN_BG, rltk::to_cp437('?'), 300.0)
                    }
                }
            }
        }

        // Remove item if it's consumable
        let consumable = world.get::<Consumable>(use_item.item);
        match consumable {
            Err(_e) => {}
            Ok(_) => {
                if used_item {
                    to_remove.push(use_item.item);
                }
            }
        }

        // Equip if item is equippable
        let equippable = world.get::<Equippable>(use_item.item);
        match equippable {
            Err(_e) => {}
            Ok(equippable) => {
                let target = targets[0];
                
                // Unequip already equipped item
                for (id, (equipped, name)) in world.query::<(&Equipped, &Name)>().iter() {
                    if equipped.owner == target && equipped.slot == equippable.slot {
                        to_unequip.push((id, name.clone(), target));
                    }
                }

                // Actually equip item
                let item_name = (*world.get::<Name>(use_item.item).unwrap()).clone();
                to_equip.push((use_item.item, *equippable, item_name, target));
            }
        }
    }

    for id in to_remove {
        world.despawn(id).unwrap();
    }

    for id in to_remove_wants_use {
        world.remove_one::<WantsToUseItem>(id).unwrap();
    }

    for (id, deals_damage) in to_add_take_damage {
        TakeDamage::add_damage(world, id, deals_damage.damage);
    }

    for (id, heals) in to_heal {
        let mut stats = world.get_mut::<CombatStats>(id).unwrap();
        stats.hp = i32::min(stats.hp + heals.heal, stats.max_hp);
    }

    for (id, confusion) in to_add_confusion {
        world.insert_one(id, Confusion{turns: confusion.turns}).unwrap();
    }

    for (id, name, target) in to_unequip {
        world.remove_one::<Equipped>(id).unwrap();
        world.insert_one(id, InBackpack{owner: target}).unwrap();
        if target == *player_id {
            log.messages.push(format!("You unequip your {}", name.name));
        }
    }

    for (id, equippable, name, target) in to_equip {
        world.insert_one(id, Equipped{owner: target, slot: equippable.slot}).unwrap();
        world.remove_one::<InBackpack>(id).unwrap();
        if target == *player_id {
            log.messages.push(format!("You equip your {}", name.name));
        }
    }
}
