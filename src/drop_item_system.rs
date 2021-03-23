use hecs::*;
use resources::*;
use crate::components::{WantsToDropItem, Position, InBackpack, Name};
use crate::gamelog::{GameLog};

pub fn drop_item(world: &mut World, res: &mut Resources) {
    let mut log = res.get_mut::<GameLog>().unwrap();
    let player_id = res.get_mut::<Entity>().unwrap();
    let mut to_drop: Vec<Entity> = Vec::new();
    let mut to_remove_wants_drop: Vec<Entity> = Vec::new();

    let mut pos: Position = Position{x: 0, y: 0};
    for (id, wants_drop) in &mut world.query::<&WantsToDropItem>().iter() {
        pos = *world.get::<Position>(id).unwrap();
        to_remove_wants_drop.push(id);
        to_drop.push(wants_drop.item);

        let item_name = world.get::<Name>(wants_drop.item).unwrap();
        if id == *player_id {
            log.messages.push(format!("You drop the {}", item_name.name));
        }
    }

    for id in to_remove_wants_drop.iter() {
        world.remove_one::<WantsToDropItem>(*id).unwrap();
    }

    for id in to_drop.iter() {
        world.remove_one::<InBackpack>(*id).unwrap();
        world.insert_one(*id, Position {x: pos.x, y: pos.y}).unwrap();
    }
}
