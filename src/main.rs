use rltk::{Rltk, GameState, RltkBuilder};
use legion::*;

mod player;
mod map;
mod components;
mod rain_system;
mod rect;

use components::{Position, Renderable, Player, Droplet};


struct Ecs {
    world: World,
    resources: Resources,
    schedule: Schedule
}

pub struct State {
    ecs: Ecs
}

impl State {
    fn run_systems(&mut self) {
        self.ecs.schedule.execute(&mut self.ecs.world, &mut self.ecs.resources);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        self.run_systems();
        player::player_input(self, ctx);

        let map = self.ecs.resources.get_mut::<Vec<map::TileType>>().unwrap();
        map::draw_map(&map, ctx);

        let mut query = <(&Position, &Renderable)>::query();

        for (pos, render) in query.iter(&self.ecs.world) {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

fn main() -> rltk::BError {
    let context = RltkBuilder::simple(80, 50).unwrap()
        .with_tile_dimensions(16, 16)
        .with_title("Roguelike")
        .build()?;

    let resources = Resources::default();


    let schedule = Schedule::builder()
        .add_system(rain_system::rain_system())
        .build();

    let mut gs = State {
        ecs: Ecs {world: World::default(), schedule: schedule, resources: resources}
    };

    let (map, rooms) = map::new_map_rooms_corridors();
    gs.ecs.resources.insert(map);
    let (player_x, player_y) = rooms[0].center();

    gs.ecs.world.push((
        Position {x: player_x, y: player_y},
        Renderable {
            glyph: rltk::to_cp437('Ω'),
            fg: rltk::RGB::named(rltk::PINK),
            bg: rltk::RGB::named(rltk::BLACK)
        },
        Player {}
    ));

    for i in 0..50 {
        gs.ecs.world.push((
            Position {x: i * 4, y: i * 2},
            Renderable {
                glyph: rltk::to_cp437('/'),
                fg: rltk::RGB::named(rltk::BLUE),
                bg: rltk::RGB::named(rltk::BLACK)
            },
            Droplet {}
        ));
    }


    rltk::main_loop(context, gs)
}
