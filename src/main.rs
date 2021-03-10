use rltk::{Rltk, GameState, RltkBuilder};
use legion::*;

mod player;
mod map;
mod components;
mod rain_system;
mod visibility_system;
mod rect;

use components::{Position, Renderable, Player, Droplet, Viewshed};
use map::{Map};


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

        map::draw_map(&self, ctx);

        let mut query = <(&Position, &Renderable)>::query();

        for (pos, render) in query.iter(&self.ecs.world) {
            if render.render {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
        ctx.print_color(
            1,
            1,
            rltk::RGB::named(rltk::YELLOW),
            rltk::RGB::named(rltk::BLACK),
            &format!("FPS: {}", ctx.fps),
        );
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
        .add_system(visibility_system::visibility_system())
        .build();

    let mut gs = State {
        ecs: Ecs {world: World::default(), schedule: schedule, resources: resources}
    };

    let map: Map = Map::new_map_rooms_corridors(30, 4, 15);
    let (player_x, player_y) = map.rooms[0].center();
    gs.ecs.resources.insert(map);

    gs.ecs.world.push((
        Position {x: player_x, y: player_y},
        Renderable {
            glyph: rltk::to_cp437('Ω'),
            fg: rltk::RGB::named(rltk::PINK),
            bg: rltk::RGB::from_f32(0., 0.1, 0.),
            render: true
        },
        Player {},
        Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true
        }
    ));

    for i in 0..50 {
        gs.ecs.world.push((
            Position {x: i * 4, y: i * 2},
            Renderable {
                glyph: rltk::to_cp437('/'),
                fg: rltk::RGB::named(rltk::BLUE),
                bg: rltk::RGB::from_f32(0., 0.1, 0.),
                render: true
            },
            Droplet {}
        ));
    }

    rltk::main_loop(context, gs)
}
