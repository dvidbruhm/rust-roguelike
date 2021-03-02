use rltk::{Rltk, GameState, VirtualKeyCode};
use legion::*;
use rand::Rng;
use std::cmp::{max, min};

struct Position {
    x: i64,
    y: i64,
}

struct Renderable {
    glyph: rltk::FontCharType,
    fg: rltk::RGB,
    bg: rltk::RGB,
}

struct Player {}

struct RandomMover {}

struct State {
    world: World,
    schedule: Schedule
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        self.run_systems();
        player_input(self, ctx);

        let mut query = <(&Position, &Renderable)>::query();

        for (pos, render) in query.iter(&self.world) {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut resources = Resources::default();
        self.schedule.execute(&mut self.world, &mut resources);
    }
}

#[system(for_each)]
fn random_mover(pos: &mut Position, _rm: &RandomMover) {
    pos.x += rand::thread_rng().gen_range(-1..3);
    pos.y += rand::thread_rng().gen_range(-1..3);

    if pos.x < 0 { pos.x = 79 }
    else if pos.x > 79 { pos.x = 0 }
    if pos.y < 0 { pos.y = 49 }
    else if pos.y > 49 { pos.y = 0 }
}

fn try_move_player(dx: i64, dy: i64, world: &mut World) {
    let mut query = <(&mut Position, &Player)>::query();

    for (pos, _player) in query.iter_mut(world) {
        pos.x = min(79, max(0, pos.x + dx));
        pos.y = min(79, max(0, pos.y + dy));
    }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.world),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.world),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.world),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.world),
            _ => {}
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike")
        .build()?;

    let schedule = Schedule::builder()
        .add_system(random_mover_system())
        .build();

    let mut gs = State {
        world: World::default(),
        schedule: schedule
    };

    gs.world.push((
        Position {x: 20, y: 20},
        Renderable {
            glyph: rltk::to_cp437('a'),
            fg: rltk::RGB::named(rltk::YELLOW),
            bg: rltk::RGB::named(rltk::BLACK)
        },
        Player {}
    ));

    for i in 0..10 {
        gs.world.push((
            Position {x: i * 4, y: i * 2},
            Renderable {
                glyph: rltk::to_cp437('b'),
                fg: rltk::RGB::named(rltk::BLUE),
                bg: rltk::RGB::named(rltk::BLACK)
            },
            RandomMover {}
        ));
    }


    rltk::main_loop(context, gs)
}
