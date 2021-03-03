use std::cmp::{max, min};
use rltk::{Rltk, VirtualKeyCode};
use legion::*;

use crate::State;
use crate::map;
use crate::map::TileType;
use crate::components::{Position, Player};

pub fn try_move_player(dx: i32, dy: i32, gs: &mut State) {
    let mut query = <(&mut Position, &Player)>::query();
    let map = gs.ecs.resources.get::<Vec<TileType>>().unwrap();

    for (pos, _player) in query.iter_mut(&mut gs.ecs.world) {
        let destination_idx = map::xy_idx(pos.x + dx, pos.y + dy);
        if map[destination_idx] != TileType::Wall {
            pos.x = min(79, max(0, pos.x + dx));
            pos.y = min(49, max(0, pos.y + dy));
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, gs),
            VirtualKeyCode::Right => try_move_player(1, 0, gs),
            VirtualKeyCode::Up => try_move_player(0, -1, gs),
            VirtualKeyCode::Down => try_move_player(0, 1, gs),
            _ => {}
        }
    }
}
