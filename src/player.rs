use std::cmp::{max, min};
use rltk::{Rltk, VirtualKeyCode};
use legion::*;

use crate::{State, RunState};
use crate::map::{Map, TileType};
use crate::components::{Position, Player, Viewshed};

pub fn try_move_player(dx: i32, dy: i32, gs: &mut State) {
    let mut query = <(&mut Position, &Player, &mut Viewshed)>::query();
    let map = gs.ecs.resources.get::<Map>().unwrap();

    for (pos, _player, vs) in query.iter_mut(&mut gs.ecs.world) {
        if map.get_tile(pos.x + dx, pos.y + dy) != TileType::Wall {
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
            _ => { return RunState::Paused }
        }
    }
    RunState::Running
}
