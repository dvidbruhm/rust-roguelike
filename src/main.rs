use rltk::{Rltk, GameState, RltkBuilder};
use hecs::*;
use resources::Resources;

mod player;
mod map;
mod components;
mod visibility_system;
mod monster_ai_system;
mod map_indexing_system;
mod melee_combat_system;
mod damage_system;
mod rect;

use components::{Position, Renderable, Player, Viewshed, Monster, Name, BlocksTile, CombatStats};
use map::{Map};


pub struct State {
    world: World,
    resources: Resources
}

impl State {
    fn run_systems(&mut self) {
        visibility_system::visibility(&mut self.world, &mut self.resources);
        monster_ai_system::monster_ai(&mut self.world, &mut self.resources);
        map_indexing_system::map_indexing(&mut self.world, &mut self.resources);
        melee_combat_system::melee_combat(&mut self.world);
        damage_system::damage(&mut self.world);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let mut new_runstate: RunState = *self.resources.get::<RunState>().unwrap();

        match new_runstate {
            RunState::PreRun => {
                self.run_systems();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_runstate = player::player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                new_runstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                new_runstate = RunState::AwaitingInput;
            }
        }

        self.resources.insert::<RunState>(new_runstate).unwrap();

        damage_system::delete_the_dead(&mut self.world);

        map::draw_map(&self, ctx);

        let map = self.resources.get::<Map>().unwrap();


        for (_id, (pos, render)) in self.world.query_mut::<(&Position, &Renderable)>() {
            let idx = map.xy_idx(pos.x, pos.y);
            if render.render && map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }

        // Display fps
        ctx.print_color(1, 1, rltk::RGB::named(rltk::YELLOW), rltk::RGB::named(rltk::BLACK), &format!("FPS: {}", ctx.fps));
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum RunState { AwaitingInput, PreRun, PlayerTurn, MonsterTurn }

fn main() -> rltk::BError {
    let context = RltkBuilder::simple(80, 50).unwrap()
        .with_tile_dimensions(16, 16)
        .with_title("Roguelike")
        .build()?;

    let mut gs = State {
        world: World::new(),
        resources: Resources::default()
    };

    let map: Map = Map::new_map_rooms_corridors(30, 4, 15);
    let (player_x, player_y) = map.rooms[0].center();

    // Player
    let player_id = gs.world.spawn((
        Position {x: player_x, y: player_y},
        Renderable {
            glyph: rltk::to_cp437('ô'),
            fg: rltk::RGB::named(rltk::PINK),
            bg: rltk::RGB::from_f32(0., 0.1, 0.),
            render: true
        },
        Player {},
        Viewshed {
            visible_tiles: Vec::new(),
            range: 20,
            dirty: true
        },
        Name {name: "Blabinou".to_string()},
        CombatStats {max_hp: 30, hp: 30, defense: 2, power: 5}
    ));

    //Monsters
    for (i, r) in map.rooms.iter().skip(1).enumerate() {
        gs.world.spawn((
            Position {x: r.center().0, y: r.center().1},
            Renderable {
                glyph: rltk::to_cp437('ÿ'),
                fg: rltk::RGB::from_f32(1., 0., 0.),
                bg: rltk::RGB::from_f32(0.2, 0.1, 0.0),
                render: true
            },
            Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true
            },
            Monster {},
            Name {name: format!("Wierd y #{}", i)},
            BlocksTile {},
            CombatStats {max_hp: 8, hp: 8, defense: 1, power: 4}
        ));
    }

    gs.resources.insert(map);
    gs.resources.insert(rltk::Point::new(player_x, player_y));
    gs.resources.insert(player_id);
    gs.resources.insert(RunState::PreRun);

    rltk::main_loop(context, gs)
}
