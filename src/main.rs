use rltk::{Rltk, GameState, RltkBuilder, Point};
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
mod item_use_system;
mod rect;
mod gui;
mod gamelog;
mod spawner;
mod weighted_table;
mod unequip_item_system;
mod particle_system;

pub mod map_builders;

use components::{Position, Renderable, WantsToUseItem, WantsToDropItem, Ranged, InBackpack, Player, Viewshed, Equipped, WantsToUnequipItem};
use map::Map;
use gamelog::GameLog;

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

#[derive(Copy, Clone, PartialEq)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowItemActions {item: Entity},
    ShowTargeting {range: i32, item: Entity},
    MainMenu {menu_selection: gui::MainMenuSelection},
    SaveGame,
    NextLevel,
    GameOver
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum RenderOrder {
    Items,
    NPC,
    Player,
    Particle
}

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
        unequip_item_system::unequip_item(&mut self.world, &mut self.resources);
        item_use_system::item_use(&mut self.world, &mut self.resources);
        damage_system::damage(&mut self.world);
        particle_system::spawn_particles(&mut self.world, &mut self.resources);
    }

    fn entities_to_delete_on_level_change(&mut self) -> Vec<Entity> {
        let mut ids_to_delete: Vec<Entity> = Vec::new();
        let all_entities: Vec<Entity> = self.world.iter().map(|(id, _)| id).collect();

        let player_id = self.resources.get::<Entity>().unwrap();

        for id in all_entities {
            let mut to_delete = true;
            if let Ok(_p) =  self.world.get::<Player>(id) { to_delete = false; }
            
            if let Ok(backpack) = self.world.get::<InBackpack>(id) {
                if backpack.owner == *player_id { to_delete = false; }
            }

            if let Ok(equipped) = self.world.get::<Equipped>(id) {
                if equipped.owner == *player_id { to_delete = false; }
            }

            if to_delete { ids_to_delete.push(id); }
        }

        ids_to_delete
    }

    fn next_level(&mut self) {
        let ids_to_delete = self.entities_to_delete_on_level_change();
        for id in ids_to_delete {
            self.world.despawn(id).unwrap();
        }

        let mut map_builder;
        let current_depth;
        let start_pos;
        {
            let mut map = self.resources.get_mut::<Map>().unwrap();
            current_depth = map.depth;
            map_builder = map_builders::random_builder(current_depth + 1);
            map_builder.build_map();
            *map = map_builder.get_map();
            start_pos = map_builder.get_starting_position();
        }

        map_builder.spawn_entities(&mut self.world, &mut self.resources);

        let player_id = self.resources.get::<Entity>().unwrap();
        let mut player_pos = self.resources.get_mut::<Point>().unwrap();
        *player_pos = Point::new(start_pos.x, start_pos.y);
        let mut player_pos_comp = self.world.get_mut::<Position>(*player_id).unwrap();
        player_pos_comp.x = start_pos.x;
        player_pos_comp.y = start_pos.y;

        let player_vs = self.world.get_mut::<Viewshed>(*player_id);
        if let Ok(mut vs) = player_vs { vs.dirty = true; }

        let mut log = self.resources.get_mut::<GameLog>().unwrap();
        log.messages.push("You descend in the staircase".to_string());
    }

    fn game_over_cleanup(&mut self) {
        // Delete everything
        self.world.clear();

        // Create map
        let mut map_builder;
        let start_pos;
        {
            let mut map_res = self.resources.get_mut::<Map>().unwrap();
            map_builder = map_builders::random_builder(1);
            map_builder.build_map();
            *map_res = map_builder.get_map();
            start_pos = map_builder.get_starting_position();
        }

        map_builder.spawn_entities(&mut self.world, &mut self.resources);

        // Create player
        let player_id = spawner::player(&mut self.world, (start_pos.x, start_pos.y));
        self.resources.insert(Point::new(start_pos.x, start_pos.y));
        self.resources.insert(player_id);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        particle_system::update_particles(&mut self.world, &mut self.resources, ctx);

        let mut new_runstate: RunState = *self.resources.get::<RunState>().unwrap();

        match new_runstate {
            RunState::MainMenu{..} => {}
            RunState::GameOver => {}
            _ => {
                map::draw_map(&self, ctx);

                {
                    let map = self.resources.get::<Map>().unwrap();

                    let mut query = self.world.query::<(&Position, &Renderable)>();
                    let mut to_render = query.iter().collect::<Vec<_>>();
                    to_render.sort_by_key(|a| a.1.1.order);

                    for (_id, (pos, render)) in to_render {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if (idx < map.tiles.len() && render.render && map.visible_tiles[idx]) || render.always_render {
                            ctx.set(pos.x + map::OFFSET_X as i32, pos.y + map::OFFSET_Y as i32, render.fg, render.bg, render.glyph);
                        }
                    }

                    gui::draw_gui(&self.world, &self.resources, ctx);
                }
            }
        }

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
                        new_runstate = RunState::ShowItemActions{ item: result.1.unwrap() }
                    }
                }
            }
            RunState::ShowItemActions{item} => {
                let result = gui::show_item_actions(&mut self.world, &mut self.resources, item, ctx);
                match result {
                    gui::ItemActionSelection::NoSelection => {}
                    gui::ItemActionSelection::Used => {
                        let mut to_add_wants_use_item: Vec<Entity> = Vec::new();
                        {
                            let player_id = self.resources.get::<Entity>().unwrap();
                            let is_item_ranged = self.world.get::<Ranged>(item);
                            match is_item_ranged {
                                Ok(is_item_ranged) => {
                                    new_runstate = RunState::ShowTargeting{range:is_item_ranged.range, item};
                                }
                                Err(_) => {
                                    to_add_wants_use_item.push(*player_id);
                                    new_runstate = RunState::PlayerTurn;
                                }
                            }
                        }

                        for id in to_add_wants_use_item.iter() {
                            self.world.insert_one(*id, WantsToUseItem {item, target: None}).unwrap();
                        }
                    }
                    gui::ItemActionSelection::Dropped => {
                        let player_id = self.resources.get::<Entity>().unwrap();
                        self.world.insert_one(*player_id, WantsToDropItem {item}).unwrap();
                        new_runstate = RunState::PlayerTurn;
                    }
                    gui::ItemActionSelection::Unequipped => {
                        let player_id = self.resources.get::<Entity>().unwrap();
                        self.world.insert_one(*player_id, WantsToUnequipItem{item}).unwrap();
                        new_runstate = RunState::PlayerTurn;
                    }
                    gui::ItemActionSelection::Cancel => { new_runstate = RunState::ShowInventory}
                }
            }
            RunState::ShowTargeting{range, item} => {
                let res = gui::ranged_target(&mut self.world, &mut self.resources, ctx, range);
                match res.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {},
                    gui::ItemMenuResult::Selected => {
                        let player_id = self.resources.get::<Entity>().unwrap();
                        self.world.insert_one(*player_id, WantsToUseItem{item, target: res.1}).unwrap();
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::MainMenu{..} => {
                let result = gui::main_menu(&mut self.world, &mut self.resources, ctx);
                match result {
                    gui::MainMenuResult::NoSelection{selected} => {new_runstate = RunState::MainMenu{menu_selection: selected}}
                    gui::MainMenuResult::Selection{selected} => {
                        match selected {
                            gui::MainMenuSelection::NewGame => {new_runstate = RunState::PreRun}
                            gui::MainMenuSelection::LoadGame => {new_runstate = RunState::PreRun}
                            gui::MainMenuSelection::Exit => {::std::process::exit(0)}
                        }
                    }
                }
            }
            RunState::SaveGame => {
                /*
                let data = serde_json::to_string(&*self.resources.get::<Map>().unwrap()).unwrap();
                println!("{}", data);
    
                let c: Context;
                let mut writer = Vec::with_capacity(128);
                let s = serde_json::Serializer::new(writer);
                hecs::serialize::row::serialize(&self.world, &mut c, s);

                for (id, _s) in self.world.query_mut::<&SerializeMe>() {
                    println!("{:?}", id);
                }
                */
                println!("Saving game... TODO");
                new_runstate = RunState::MainMenu{menu_selection: gui::MainMenuSelection::LoadGame};
            }
            RunState::NextLevel => {
                self.next_level();
                new_runstate = RunState::PreRun;
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => {}
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        new_runstate = RunState::MainMenu {menu_selection: gui::MainMenuSelection::NewGame};
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

    let mut map_builder = map_builders::random_builder(1);
    map_builder.build_map();
    let player_pos = map_builder.get_starting_position();
    gs.resources.insert(rltk::RandomNumberGenerator::new());

    // Player
    let player_id = spawner::player(&mut gs.world, (player_pos.x, player_pos.y));

    // Monsters and items
    map_builder.spawn_entities(&mut gs.world, &mut gs.resources);

    gs.resources.insert(map_builder.get_map());
    gs.resources.insert(rltk::Point::new(player_pos.x, player_pos.y));
    gs.resources.insert(player_id);
    gs.resources.insert(RunState::MainMenu{menu_selection: gui::MainMenuSelection::NewGame});
    gs.resources.insert(gamelog::GameLog{messages: vec!["Welcome to the roguelike!".to_string()]});
    gs.resources.insert(particle_system::ParticleBuilder::new());

    rltk::main_loop(context, gs)
}
