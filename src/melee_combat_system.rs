use hecs::*;
use resources::*;
use crate::components::{WantsToAttack, Name, CombatStats, TakeDamage};
use crate::gamelog::{GameLog};

pub fn melee_combat(world: &mut World, res: &mut Resources) {
    let mut log = res.get_mut::<GameLog>().unwrap();

    let mut to_remove_wants_melee: Vec<Entity> = vec![];
    let mut to_add_damage: Vec<(Entity, i32)> = vec![];

    for (id, (wants_attack, name, stats)) in &mut world.query::<(&WantsToAttack, &Name, &CombatStats)>() {
        if stats.hp > 0 {
            let target_stats = &world.get::<CombatStats>(wants_attack.target).unwrap();
            if target_stats.hp > 0 {
                let target_name = &world.get::<Name>(wants_attack.target).unwrap();
                
                let damage = i32::max(0, stats.power - target_stats.defense);
                
                if damage == 0 {
                    log.messages.push(format!("{} is unable to hurt {}", &name.name, &target_name.name));
                }
                else {
                    log.messages.push(format!("{} hits {} for {} hp", &name.name, &target_name.name, damage));
                    to_add_damage.push((wants_attack.target, damage));
                }
            }
        }
        to_remove_wants_melee.push(id);
    }
    
    for (id, damage) in to_add_damage.iter() {
        TakeDamage::add_damage(world, *id, *damage);
    }

    for id in to_remove_wants_melee.iter() {
        let _res = world.remove_one::<WantsToAttack>(*id);
    }
}
