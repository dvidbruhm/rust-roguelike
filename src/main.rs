use rltk::{Rltk, GameState, RGB, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};
use specs_derive::Component;

struct State {
    ecs: World
}
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        self.run_systems();

        player_input(self, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, ren) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, ren.fg, ren.bg, ren.glyph);
        }

    }
}

impl State {
    fn run_systems(&mut self) {
        let mut lw = LeftWalker{};
        lw.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike")
        .build()?;
    let mut gs = State{
        ecs: World::new()
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();

    for i in 0..10 {
        gs.ecs.create_entity()
            .with(Position {x: i * 4, y: i * 2})
            .with(Renderable {
                glyph: rltk::to_cp437('♥'),
                fg: RGB::named(rltk::GREEN),
                bg: RGB::named(rltk::BLACK),
            })
            .with(LeftMover{})
            .build();
    }
    for i in 0..10 {
        gs.ecs.create_entity()
            .with(Position {x: 80 - i * 4, y: 50 - i * 2})
            .with(Renderable {
                glyph: rltk::to_cp437('µ'),
                fg: RGB::named(rltk::BLUE),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Player {})
            .build();
    }

    rltk::main_loop(context, gs)
}

fn try_move_player(dx: i64, dy: i64, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();


    for (_player, pos) in (&players, &mut positions).join() {
        pos.x = min(79, max(0, pos.x + dx));
        pos.y = min(79, max(0, pos.y + dy));
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

#[derive(Component)]
struct Position {
    x: i64,
    y: i64,
}

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(Component)]
struct LeftMover {}

struct LeftWalker {}

impl <'a> System<'a> for LeftWalker {
    type SystemData = (ReadStorage<'a, LeftMover>,
                       WriteStorage<'a, Position>);

    fn run(&mut self, (lefty, mut pos) : Self::SystemData) {
        for (_lefty, pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            if pos.x < 0 {
                pos.x = 79;
            }
        }
    }
}

#[derive(Component, Debug)]
struct Player {}
