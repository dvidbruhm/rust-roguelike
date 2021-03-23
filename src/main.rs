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
mod inventory_system;
mod drop_item_system;
mod potion_system;
mod rect;
mod gui;
mod gamelog;
mod spawner;

use components::{Position, Renderable, WantsToDrinkPotion, WantsToDropItem};
use map::{Map};


pub struct Palette;
impl Palette {
    //const TRANS: rltk::RGBA = rltk::RGBA{r:1., g:1., b:1., a:0.};
    const MAIN_BG: rltk::RGB = rltk::RGB{r: 0., g: 0., b: 0.};
    const MAIN_FG: rltk::RGB = rltk::RGB{r: 0., g: 0.8, b: 0.8};
    const COLOR_0: rltk::RGB = rltk::RGB{r: 1., g: 0., b: 1.};
    const COLOR_1: rltk::RGB = rltk::RGB{r: 1., g: 0., b: 0.};
    const COLOR_2: rltk::RGB = rltk::RGB{r: 0., g: 0.2, b: 0.};
    const COLOR_3: rltk::RGB = rltk::RGB{r: 0.7, g: 0.2, b: 0.2};
    const COLOR_4: rltk::RGB = rltk::RGB{r: 0.7, g:0.7, b:0.};
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum RunState { AwaitingInput, PreRun, PlayerTurn, MonsterTurn, ShowInventory }

pub struct State {
    world: World,
    resources: Resources
}

impl State {
    fn run_systems(&mut self) {
        visibility_system::visibility(&mut self.world, &mut self.resources);
        monster_ai_system::monster_ai(&mut self.world, &mut self.resources);
        map_indexing_system::map_indexing(&mut self.world, &mut self.resources);
        melee_combat_system::melee_combat(&mut self.world, &mut self.resources);
        inventory_system::inventory(&mut self.world, &mut self.resources);
        drop_item_system::drop_item(&mut self.world, &mut self.resources);
        damage_system::damage(&mut self.world);
        potion_system::potion(&mut self.world, &mut self.resources);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        map::draw_map(&self, ctx);

        {
            let map = self.resources.get::<Map>().unwrap();

            let mut query = self.world.query::<(&Position, &Renderable)>();
            let mut to_render = query.iter().collect::<Vec<_>>();
            to_render.sort_by_key(|a| -a.1.1.order);

            for (_id, (pos, render)) in to_render {
                let idx = map.xy_idx(pos.x, pos.y);
                if render.render && map.visible_tiles[idx] {
                    ctx.set(pos.x + map::OFFSET_X as i32, pos.y + map::OFFSET_Y as i32, render.fg, render.bg, render.glyph);
                }
            }

            gui::draw_gui(&self.world, &self.resources, ctx);
        }

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
            RunState::ShowInventory => {
                let result = gui::show_inventory(&mut self.world, &mut self.resources, ctx);
                match result.0 {
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Cancel => { new_runstate = RunState::AwaitingInput }
                    gui::ItemMenuResult::Selected => {
                        let item_id = result.1.unwrap();
                        let player_id = self.resources.get::<Entity>().unwrap();
                        self.world.insert_one(*player_id, WantsToDrinkPotion {potion: item_id}).unwrap();
                        new_runstate = RunState::PlayerTurn;
                    }
                    gui::ItemMenuResult::Dropped => {
                        let item_id = result.1.unwrap();
                        let player_id = self.resources.get::<Entity>().unwrap();
                        self.world.insert_one(*player_id, WantsToDropItem {item: item_id}).unwrap();
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
        }

        self.resources.insert::<RunState>(new_runstate).unwrap();

        damage_system::delete_the_dead(&mut self.world, &mut self.resources);

    }
}

fn main() -> rltk::BError {
    let context = RltkBuilder::simple(80, 50).unwrap()
        .with_tile_dimensions(12, 12)
        .with_title("Roguelike")
        .build()?;

    let mut gs = State {
        world: World::new(),
        resources: Resources::default()
    };

    let map: Map = Map::new_map_rooms_corridors(30, 4, 15);
    let player_pos = map.rooms[0].center();
    gs.resources.insert(rltk::RandomNumberGenerator::new());

    // Player
    let player_id = spawner::player(&mut gs.world, player_pos);

    // Monsters
    for r in map.rooms.iter().skip(1) {
        spawner::fill_room(&mut gs.world, &mut gs.resources, r);
    }

    gs.resources.insert(map);
    gs.resources.insert(rltk::Point::new(player_pos.0, player_pos.1));
    gs.resources.insert(player_id);
    gs.resources.insert(RunState::PreRun);
    gs.resources.insert(gamelog::GameLog{messages: vec!["Welcome to the roguelike!".to_string()]});

    rltk::main_loop(context, gs)
}
