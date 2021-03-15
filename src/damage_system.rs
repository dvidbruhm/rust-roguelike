use hecs::*;
use crate::components::{TakeDamage, CombatStats, Player};

pub fn damage(world: &mut World) {
    for (_id, (take_dmg, stats)) in &mut world.query::<(&mut TakeDamage, &mut CombatStats)>() {
        if !take_dmg.amount.is_empty() {
            let total_dmg = take_dmg.amount.iter().sum::<i32>();
            stats.hp -= total_dmg;
            take_dmg.amount.drain(0..);
        }
    }
}

pub fn delete_the_dead(world: &mut World) {
    let mut dead: Vec<Entity> = vec![];

    for (id, stats) in &mut world.query::<&CombatStats>() {
        if stats.hp <= 0 {
            match &world.get::<&Player>(id) {
                Err(_) => dead.push(id),
                Ok(_p) => println!("You are dead")
            }
        }
    }

    for id in dead.iter() {
        let _res = world.despawn(*id);
    }
}
