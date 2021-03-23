use hecs::*;
use resources::*;
use crate::gamelog::{GameLog};
use crate::components::{WantsToDrinkPotion, CombatStats, Potion, Name};

pub fn potion(world: &mut World, res: &mut Resources) {
    let mut log = res.get_mut::<GameLog>().unwrap();
    let player_id = res.get::<Entity>().unwrap();
    let mut to_remove: Vec<Entity> = Vec::new();
    let mut to_remove_wants_drink: Vec<Entity> = Vec::new();

    for (id, (drink, stats)) in &mut world.query::<(&WantsToDrinkPotion, &mut CombatStats)>().iter() {
        let potion = world.get::<Potion>(drink.potion);
        match potion {
            Err(_e) => {}
            Ok(p) => {
                stats.hp = i32::min(stats.hp + p.heal, stats.max_hp);
                if id == *player_id {
                    let name = world.get::<Name>(drink.potion).unwrap();
                    log.messages.push(format!("You drink the {}, healing {} hp", name.name, p.heal));
                }
                to_remove.push(drink.potion);
                to_remove_wants_drink.push(id);
            }
        }
    }

    for id in to_remove.iter() {
        world.despawn(*id).unwrap();
    }

    for id in to_remove_wants_drink.iter() {
        world.remove_one::<WantsToDrinkPotion>(*id).unwrap();
    }
}
