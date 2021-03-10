use rltk::{Rltk, GameState, RltkBuilder};
use legion::*;

mod player;
mod map;
mod components;
mod rain_system;
mod visibility_system;
mod monster_ai_system;
mod rect;

use components::{Position, Renderable, Player, Viewshed, Monster, Name};
use map::{Map};


struct Ecs {
    world: World,
    resources: Resources,
    schedule: Schedule
}

pub struct State {
    ecs: Ecs,
    run_state: RunState
}

impl State {
    fn run_systems(&mut self) {
        self.ecs.schedule.execute(&mut self.ecs.world, &mut self.ecs.resources);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        if self.run_state == RunState::Running {
            self.run_systems();
            self.run_state = RunState::Paused;
        }
        else if self.run_state == RunState::Paused {
            self.run_state = player::player_input(self, ctx);
        }

        map::draw_map(&self, ctx);

        let map = self.ecs.resources.get::<Map>().unwrap();
        let mut query = <(&Position, &Renderable)>::query();


        for (pos, render) in query.iter(&self.ecs.world) {
            let idx = map.xy_idx(pos.x, pos.y);
            if render.render && map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }

        // Display fps
        ctx.print_color(1, 1, rltk::RGB::named(rltk::YELLOW), rltk::RGB::named(rltk::BLACK), &format!("FPS: {}", ctx.fps));
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum RunState { Paused, Running }

fn main() -> rltk::BError {
    let context = RltkBuilder::simple(80, 50).unwrap()
        .with_tile_dimensions(16, 16)
        .with_title("Roguelike")
        .build()?;

    let resources = Resources::default();


    let schedule = Schedule::builder()
        .add_system(rain_system::rain_system())
        .add_system(visibility_system::visibility_system())
        .add_system(monster_ai_system::monster_ai_system())
        .build();

    let mut gs = State {
        ecs: Ecs {world: World::default(), schedule: schedule, resources: resources},
        run_state: RunState::Running
    };

    let map: Map = Map::new_map_rooms_corridors(30, 4, 15);
    let (player_x, player_y) = map.rooms[0].center();

    // Player
    gs.ecs.world.push((
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
            range: 8,
            dirty: true
        },
        Name {name: "Blabinou".to_string()}
    ));

    //Monsters
    for (i, r) in map.rooms.iter().skip(1).enumerate() {
        gs.ecs.world.push((
            Position {x: r.center().0, y: r.center().1},
            Renderable {
                glyph: rltk::to_cp437('ÿ'),
                fg: rltk::RGB::from_f32(1., 0., 0.),
                bg: rltk::RGB::from_f32(0.2, 0.1, 0.0),
                render: true
            },
            Viewshed {
                visible_tiles: Vec::new(),
                range: 6,
                dirty: true
            },
            Monster {},
            Name {name: format!("Wierd y #{}", i)}
        ));
    }

    //Droplets
    /*for i in 0..50 {
        gs.ecs.world.push((
            Position {x: i * 4, y: i * 2},
            Renderable {
                glyph: rltk::to_cp437('•'),
                fg: rltk::RGB::named(rltk::BLUE),
                bg: rltk::RGB::from_f32(0., 0.1, 0.),
                render: true
            },
            Droplet {}
        ));
    }*/

    gs.ecs.resources.insert(map);
    gs.ecs.resources.insert(rltk::Point::new(player_x, player_y));
    rltk::main_loop(context, gs)
}
