use std::cmp::{max, min};
use rltk::{Rltk, VirtualKeyCode, Point};
use hecs::*;
use resources::*;

use crate::{State, RunState};
use crate::map::{Map, TileType};
use crate::components::{Position, Player, Viewshed, CombatStats, WantsToAttack, Item, WantsToPickupItem};
use crate::gamelog::GameLog;

pub fn try_move_player(dx: i32, dy: i32, gs: &mut State) {
    let map = gs.resources.get::<Map>().unwrap();
    let mut needs_wants_to_attack: Option<(Entity, WantsToAttack)> = None;

    for (id, (pos, _player, vs)) in &mut gs.world.query::<(&mut Position, &Player, &mut Viewshed)>().iter() {
        let dest_idx = map.xy_idx(pos.x + dx, pos.y + dy);

        for potential_target in map.tile_content[dest_idx].iter() {
            let target_cs = &gs.world.get::<CombatStats>(*potential_target);
            match target_cs {
                Ok(_cs) => {
                    needs_wants_to_attack = Some((id, WantsToAttack {target: *potential_target}));
                    break;
                }
                Err(_e) => {}
            }
        }

        if !map.blocked[dest_idx] {
            pos.x = min(79, max(0, pos.x + dx));
            pos.y = min(49, max(0, pos.y + dy));

            vs.dirty = true;

            let mut ppos = gs.resources.get_mut::<rltk::Point>().unwrap();
            ppos.x = pos.x;
            ppos.y = pos.y;
        }
    }

    if let Some(v) = needs_wants_to_attack {
        let _res = gs.world.insert_one(v.0, v.1);
    }
}

pub fn get_item(world: &mut World, res: &mut Resources){
    let player_id = res.get::<Entity>().unwrap();
    let player_pos = res.get::<Point>().unwrap();
    let mut log = res.get_mut::<GameLog>().unwrap();

    let mut target_item: Option<Entity> = None;

    for (id, (_item, pos)) in &mut world.query::<(&Item, &Position)>() {
        if pos.x == player_pos.x && pos.y == player_pos.y {
            target_item = Some(id);
        }
    }

    match target_item {
        None => {log.messages.push(format!("There is nothing to pick up here"))}
        Some(item) => {
            let _res = world.insert_one(*player_id, WantsToPickupItem {
                collected_by: *player_id,
                item
            });
        }
    }
}

fn try_next_level(_world: &mut World, res: &mut Resources) -> bool {
    let player_pos = res.get::<Point>().unwrap();
    let map = res.get::<Map>().unwrap();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::StairsDown {
        true
    }
    else {
        let mut log = res.get_mut::<GameLog>().unwrap();
        log.messages.push(format!("There is no stairs down here"));
        false
    }
}

fn skip_turn(world: &mut World, res: &mut Resources) -> RunState {
    let player_id = res.get::<Entity>().unwrap();
    let mut stats = world.get_mut::<CombatStats>(*player_id).unwrap();
    stats.hp = i32::min(stats.hp + stats.regen_rate, stats.max_hp);
    RunState::PlayerTurn
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => { return RunState::AwaitingInput }
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, gs),
            VirtualKeyCode::Right => try_move_player(1, 0, gs),
            VirtualKeyCode::Up => try_move_player(0, -1, gs),
            VirtualKeyCode::Down => try_move_player(0, 1, gs),
            VirtualKeyCode::Y => try_move_player(-1, -1, gs),
            VirtualKeyCode::U => try_move_player(1, -1, gs),
            VirtualKeyCode::N => try_move_player(1, 1, gs),
            VirtualKeyCode::B => try_move_player(-1, 1, gs),
            VirtualKeyCode::G => get_item(&mut gs.world, &mut gs.resources),
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::W => return skip_turn(&mut gs.world, &mut gs.resources),
            VirtualKeyCode::Escape => return RunState::SaveGame,
            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.world, &mut gs.resources) { return RunState::NextLevel; }
            }
            _ => { return RunState::AwaitingInput }
        }
    }
    RunState::PlayerTurn
}
