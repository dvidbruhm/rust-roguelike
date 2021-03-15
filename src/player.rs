use std::cmp::{max, min};
use rltk::{Rltk, VirtualKeyCode};
use hecs::*;

use crate::{State, RunState};
use crate::map::{Map};
use crate::components::{Position, Player, Viewshed, CombatStats, WantsToAttack};

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
            _ => { return RunState::AwaitingInput }
        }
    }
    RunState::PlayerTurn
}
