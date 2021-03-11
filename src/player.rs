use std::cmp::{max, min};
use rltk::{Rltk, VirtualKeyCode};
use legion::*;

use crate::{State, RunState};
use crate::map::{Map};
use crate::components::{Position, Player, Viewshed, CombatStats, Name};

pub fn try_move_player(dx: i32, dy: i32, gs: &mut State) {
    let mut query = <(&mut Position, &Player, &mut Viewshed)>::query();
    let map = gs.ecs.resources.get::<Map>().unwrap();

    let mut dest_idx = 0;
    for (pos, _player, _vs) in query.iter_mut(&mut gs.ecs.world) {
        dest_idx = map.xy_idx(pos.x + dx, pos.y + dy);
    }

    for potential_target in map.tile_content[dest_idx].iter() {
        if let Ok(mut combat_stats) = gs.ecs.world.entry_mut(*potential_target).unwrap().get_component_mut::<CombatStats>() {
            if let Ok(name) = gs.ecs.world.entry_mut(*potential_target).unwrap().get_component_mut::<Name>() {
                println!("I stab : {}", name.name);
                return;
            }
        }
    }

    if !map.blocked[dest_idx] {
        for (pos, _player, vs) in query.iter_mut(&mut gs.ecs.world) {
            pos.x = min(79, max(0, pos.x + dx));
            pos.y = min(49, max(0, pos.y + dy));

            vs.dirty = true;

            let mut ppos = gs.ecs.resources.get_mut::<rltk::Point>().unwrap();
            ppos.x = pos.x;
            ppos.y = pos.y;
        }
    }

}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => { return RunState::Paused }
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, gs),
            VirtualKeyCode::Right => try_move_player(1, 0, gs),
            VirtualKeyCode::Up => try_move_player(0, -1, gs),
            VirtualKeyCode::Down => try_move_player(0, 1, gs),
            VirtualKeyCode::Y => try_move_player(-1, -1, gs),
            VirtualKeyCode::U => try_move_player(1, -1, gs),
            VirtualKeyCode::N => try_move_player(1, 1, gs),
            VirtualKeyCode::B => try_move_player(-1, 1, gs),
            _ => { return RunState::Paused }
        }
    }
    RunState::Running
}
