use rltk::{Rltk, GameState, VirtualKeyCode};
use legion::*;
use std::cmp::{max, min};


#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall, Floor
}

struct Ecs {
    world: World,
    resources: Resources,
    schedule: Schedule
}

struct Position {
    x: i32,
    y: i32,
}

struct Renderable {
    glyph: rltk::FontCharType,
    fg: rltk::RGB,
    bg: rltk::RGB,
}

struct Player {}

struct RandomMover {}

struct State {
    ecs: Ecs
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        self.run_systems();
        player_input(self, ctx);

        let map = self.ecs.resources.get_mut::<Vec<TileType>>().unwrap();
        draw_map(&map, ctx);

        let mut query = <(&Position, &Renderable)>::query();

        for (pos, render) in query.iter(&self.ecs.world) {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        self.ecs.schedule.execute(&mut self.ecs.world, &mut self.ecs.resources);
    }
}

#[system(for_each)]
fn random_mover(pos: &mut Position, _rm: &RandomMover) {
    let mut rng = rltk::RandomNumberGenerator::new();
    pos.x += rng.range(-1, 1);
    pos.y += rng.range(-1, 1);

    if pos.x < 0 { pos.x = 79 }
    else if pos.x > 79 { pos.x = 0 }
    if pos.y < 0 { pos.y = 49 }
    else if pos.y > 49 { pos.y = 0 }
}

fn try_move_player(dx: i32, dy: i32, ecs: &mut Ecs) {
    let mut query = <(&mut Position, &Player)>::query();
    let map = ecs.resources.get::<Vec<TileType>>().unwrap();

    for (pos, _player) in query.iter_mut(&mut ecs.world) {
        let destination_idx = xy_idx(pos.x + dx, pos.y + dy);
        if map[destination_idx] != TileType::Wall {
            pos.x = min(79, max(0, pos.x + dx));
            pos.y = min(49, max(0, pos.y + dy));
        }
    }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => {}
        }
    }
}

fn new_map() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; 80 * 50];
    
    let mut rng = rltk::RandomNumberGenerator::new();
    
    for _i in 0..400 {
        let idx = xy_idx(rng.range(1, 79), rng.range(1, 49));
        map[idx] = TileType::Wall;
    }

    map
}

fn draw_map(map: &[TileType], ctx : &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        // Render a tile depending upon the tile type
        match tile {
            TileType::Floor => {
                ctx.set(x, y, rltk::RGB::from_f32(0.5, 0.5, 0.5), rltk::RGB::from_f32(0., 0., 0.), rltk::to_cp437(' '));
            }
            TileType::Wall => {
                ctx.set(x, y, rltk::RGB::from_f32(0.0, 1.0, 0.0), rltk::RGB::from_f32(0., 0., 0.), rltk::to_cp437('#'));
            }
        }

        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike")
        .build()?;

    let resources = Resources::default();


    let schedule = Schedule::builder()
        .add_system(random_mover_system())
        .build();

    let mut gs = State {
        ecs: Ecs {world: World::default(), schedule: schedule, resources: resources}
    };

    gs.ecs.world.push((
        Position {x: 20, y: 20},
        Renderable {
            glyph: rltk::to_cp437('Ω'),
            fg: rltk::RGB::named(rltk::PINK),
            bg: rltk::RGB::named(rltk::BLACK)
        },
        Player {}
    ));

    for i in 0..10 {
        gs.ecs.world.push((
            Position {x: i * 4, y: i * 2},
            Renderable {
                glyph: rltk::to_cp437('b'),
                fg: rltk::RGB::named(rltk::BLUE),
                bg: rltk::RGB::named(rltk::BLACK)
            },
            RandomMover {}
        ));
    }

    gs.ecs.resources.insert(new_map());

    rltk::main_loop(context, gs)
}
