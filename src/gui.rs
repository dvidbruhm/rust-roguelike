use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;
use rltk::{Rltk, Point, VirtualKeyCode};
use hecs::*;
use resources::*;
use crate::components::{CombatStats, Name, Position, InBackpack, Viewshed, Equipped, Equippable};
use crate::gamelog::GameLog;
use crate::map;
use crate::map::Map;
use crate::{Palette, RunState};

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {Cancel, NoResponse, Selected}

pub enum ItemActionSelection {Cancel, NoSelection, Used, Dropped, Unequipped}

#[derive(PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(i8)]
pub enum MainMenuSelection {NewGame, LoadGame, Exit}

pub enum MainMenuResult {NoSelection {selected: MainMenuSelection}, Selection {selected: MainMenuSelection}}

pub enum GameOverResult {NoSelection, QuitToMenu}

pub fn draw_gui(world: &World, res: &Resources, ctx: &mut Rltk) {
    ctx.print_color(0, 10, Palette::MAIN_FG, Palette::MAIN_BG, "─".repeat(80));

    let player_id: &Entity = &res.get::<Entity>().unwrap();
    let player_stats = world.get::<CombatStats>(*player_id).unwrap();
    let hp_gui = format!("{} / {} HP", player_stats.hp, player_stats.max_hp);
    let map = res.get::<Map>().unwrap();

    ctx.print_color(62, 9, Palette::MAIN_FG, Palette::MAIN_BG, format!("Depth: {}", map.depth));
    ctx.print_color(62, 1, Palette::MAIN_FG, Palette::MAIN_BG, hp_gui);

    for y in 0..10 {
        ctx.print_color(60, y, Palette::MAIN_FG, Palette::MAIN_BG, "│");
    }
    ctx.print_color(60, 10, Palette::MAIN_FG, Palette::MAIN_BG, "┴");

    let log = res.get::<GameLog>().unwrap();
    let mut y = 1;
    for m in log.messages.iter().rev() {
        if y < 9 {
            ctx.print_color(2, y, Palette::MAIN_FG, Palette::MAIN_BG, m);
        }
        y += 1;
    }

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, Palette::COLOR_3);
    draw_tooltips(&world, &res, ctx);

    // Display fps
    ctx.print_color(78, 49, Palette::MAIN_FG, Palette::MAIN_BG, &format!("{}", ctx.fps));
}

pub fn draw_tooltips(world: &World, res: &Resources, ctx: &mut Rltk) {
    let map = res.get::<Map>().unwrap();

    let mouse_pos = ctx.mouse_pos();
    let map_mouse_pos = map.transform_mouse_pos(mouse_pos);
    if !map.mouse_in_bounds(map_mouse_pos) { return; }

    let mut tooltip: Vec<String> = Vec::new();

    for (_id, (name, pos)) in world.query::<(&Name, &Position)>().iter() {
        let idx = map.xy_idx(pos.x, pos.y);
        if pos.x == map_mouse_pos.0 && pos.y == map_mouse_pos.1 && map.visible_tiles[idx] {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 { width = s.len() as i32; }
        }
        width += 3;

        let mut sign = 1;
        let mut arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
        let mut left_x = mouse_pos.0 + 4;
        let mut y = mouse_pos.1;
        if mouse_pos.0 > map.width / 2 {
            sign = -1;
            arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            left_x = mouse_pos.0 - width;
        }

        if sign == -1 {ctx.fill_region(rltk::Rect{x1: left_x, x2: left_x - 3 + width, y1: y, y2: y + tooltip.len() as i32 - 1}, rltk::to_cp437(' '), Palette::MAIN_FG, Palette::COLOR_3);}
        else {ctx.fill_region(rltk::Rect{x1: left_x - 1, x2: left_x + width - 4, y1: y, y2: y + tooltip.len() as i32 - 1}, rltk::to_cp437(' '), Palette::MAIN_FG, Palette::COLOR_3);}

        for s in tooltip.iter() {
            ctx.print_color(left_x, y, Palette::MAIN_FG, Palette::COLOR_3, s);
            y += 1;
        }
        ctx.print_color(arrow_pos.x, arrow_pos.y, Palette::MAIN_FG, Palette::COLOR_3, "->");
    }
}

pub fn show_inventory(world: &mut World, res: &mut Resources, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_id = res.get::<Entity>().unwrap();

    // Items in backpack
    let mut query = world.query::<(&InBackpack, &Name)>();
    let inventory = query.iter().filter(|item| item.1.0.owner == *player_id);
    let backpack_count = inventory.count();
    let mut y = 25 - (backpack_count / 2);
    ctx.draw_box(10, y - 2, 31, backpack_count + 3, Palette::MAIN_FG, Palette::MAIN_BG);

    let title = "Inventory";
    ctx.print_color(13, y - 2, Palette::MAIN_FG, Palette::MAIN_BG, title);

    let mut useable: Vec<Entity> = Vec::new();
    for (j, (id, (_pack, name))) in world.query::<(&InBackpack, &Name)>().iter().filter(|item| item.1.0.owner == *player_id).enumerate() {
        ctx.set(12, y, Palette::MAIN_FG, Palette::MAIN_BG, rltk::to_cp437('('));
        ctx.set(13, y, Palette::COLOR_0, Palette::MAIN_BG, 97 + j as rltk::FontCharType);
        ctx.set(14, y, Palette::MAIN_FG, Palette::MAIN_BG, rltk::to_cp437(')'));

        ctx.print_color(16, y, Palette::MAIN_FG, Palette::MAIN_BG, &name.name.to_string());
        useable.push(id);
        y += 1;
    }

    // Items equipped
    let mut query = world.query::<(&Equipped, &Name)>();
    let equipped_items = query.iter().filter(|item| item.1.0.owner == *player_id);
    let equipped_count = equipped_items.count();
    
    let mut y = 25 - (equipped_count / 2);
    ctx.draw_box(45, y - 2, 31, equipped_count + 3, Palette::MAIN_FG, Palette::MAIN_BG);

    let title = "Equipment";
    ctx.print_color(48, y - 2, Palette::MAIN_FG, Palette::MAIN_BG, title);

    let mut equipped: Vec<Entity> = Vec::new();
    for (j, (id, (_pack, name))) in world.query::<(&Equipped, &Name)>().iter().filter(|item| item.1.0.owner == *player_id).enumerate() {
        let offset = j + backpack_count;
        ctx.set(47, y, Palette::MAIN_FG, Palette::MAIN_BG, rltk::to_cp437('('));
        ctx.set(48, y, Palette::COLOR_0, Palette::MAIN_BG, 97 + offset as rltk::FontCharType);
        ctx.set(49, y, Palette::MAIN_FG, Palette::MAIN_BG, rltk::to_cp437(')'));

        ctx.print_color(51, y, Palette::MAIN_FG, Palette::MAIN_BG, &name.name.to_string());
        equipped.push(id);
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < backpack_count as i32 {
                        return (ItemMenuResult::Selected, Some(useable[selection as usize]))
                    } else if selection >= backpack_count as i32 && selection < (backpack_count + equipped_count) as i32 {
                        return (ItemMenuResult::Selected, Some(equipped[selection as usize - backpack_count]))
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

pub fn show_item_actions(world: &mut World, _res: &mut Resources, item: Entity, ctx: &mut Rltk) -> ItemActionSelection {
    let item_name = world.get::<Name>(item).unwrap();
    ctx.draw_box(15, 23, 31, 5, Palette::MAIN_FG, Palette::MAIN_BG);
    ctx.print_color(18, 23, Palette::MAIN_FG, Palette::MAIN_BG, item_name.name.to_string());

    let mut in_backpack = false;
    let mut in_equip = false;
    
    if let Ok(_in_backpack) = world.get::<InBackpack>(item) {
        in_backpack = true;
        if let Ok(_equippable) = world.get::<Equippable>(item) {
            ctx.print_color(17, 25, Palette::MAIN_FG, Palette::MAIN_BG, "(a) Equip");
        } else {
            ctx.print_color(17, 25, Palette::MAIN_FG, Palette::MAIN_BG, "(a) Use");
        }
        ctx.print_color(18, 25, Palette::COLOR_0, Palette::MAIN_BG, "a");
        ctx.print_color(17, 26, Palette::MAIN_FG, Palette::MAIN_BG, "(b) Drop");
        ctx.print_color(18, 26, Palette::COLOR_0, Palette::MAIN_BG, "b");
    } else if let Ok(_in_equip) = world.get::<Equipped>(item) {
        in_equip = true;
        ctx.print_color(17, 25, Palette::MAIN_FG, Palette::MAIN_BG, "(a) Unequip");
        ctx.print_color(18, 25, Palette::COLOR_0, Palette::MAIN_BG, "a");
        ctx.print_color(17, 26, Palette::MAIN_FG, Palette::MAIN_BG, "(b) Drop");
        ctx.print_color(18, 26, Palette::COLOR_0, Palette::MAIN_BG, "b");
    } else {
        panic!("Item is not in backpack or equipped? Where is it?");
    }


    match ctx.key {
        None => (ItemActionSelection::NoSelection),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { ItemActionSelection::Cancel }
                VirtualKeyCode::A => { 
                    //TODO: Add unequip action and select here based on if item has Equipped
                    //component?
                    if in_backpack { return ItemActionSelection::Used }
                    if in_equip { return ItemActionSelection::Unequipped }
                    ItemActionSelection::Used
                }
                VirtualKeyCode::B => { ItemActionSelection::Dropped }
                _ => { ItemActionSelection::NoSelection }
            }
        }
    }
}

pub fn ranged_target(world: &mut World, res: &mut Resources, ctx: &mut Rltk, range: i32) -> (ItemMenuResult, Option<Point>) {
    let player_id = res.get::<Entity>().unwrap();
    let player_pos = res.get::<Point>().unwrap();
    ctx.print_color(5, 12, Palette::COLOR_0, Palette::MAIN_BG, "Select a target");

    let mut valid_cells: Vec<Point> = Vec::new();
    match world.get::<Viewshed>(*player_id) {
        Err(_e) => {return (ItemMenuResult::Cancel, None)},
        Ok(player_vs) => {
            for pt in player_vs.visible_tiles.iter() {
                let dist = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *pt);
                if dist as i32 <= range {
                    ctx.set_bg(pt.x + map::OFFSET_X as i32, pt.y + map::OFFSET_Y as i32, Palette::COLOR_4);
                    valid_cells.push(*pt);
                }
            }
        }
    }

    let mouse_pos = ctx.mouse_pos();
    let map_mouse_pos = (mouse_pos.0 - map::OFFSET_X as i32, mouse_pos.1 - map::OFFSET_Y as i32);
    let mut valid_target = false;
    for pt in valid_cells.iter() {
        if pt.x == map_mouse_pos.0 && pt.y == map_mouse_pos.1 { valid_target = true }
    }
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, Palette::COLOR_2);
        if ctx.left_click { return (ItemMenuResult::Selected, Some(Point::new(map_mouse_pos.0, map_mouse_pos.1))) }
    }
    else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, Palette::COLOR_1);
        if ctx.left_click { return (ItemMenuResult::Cancel, None) }
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { return (ItemMenuResult::Cancel, None) },
                _ => (ItemMenuResult::NoResponse, None)
            }
        }
    }
}

pub fn main_menu(_world: &mut World, res: &mut Resources, ctx: &mut Rltk) -> MainMenuResult {
    let runstate = res.get::<RunState>().unwrap();

    let get_fg = |sel, menu_item| {
        if sel == menu_item { return Palette::COLOR_1 }
        else { return Palette::MAIN_FG }
    };

    ctx.print_color_centered(15, Palette::COLOR_2, Palette::MAIN_BG, "Roguelike");

    if let RunState::MainMenu{menu_selection: selection} = *runstate {
        ctx.print_color_centered(25, get_fg(selection, MainMenuSelection::NewGame), Palette::MAIN_BG, "Begin new adventure");
        ctx.print_color_centered(30, get_fg(selection, MainMenuSelection::LoadGame), Palette::MAIN_BG, "Load game");
        ctx.print_color_centered(35, get_fg(selection, MainMenuSelection::Exit), Palette::MAIN_BG, "Exit");

        match ctx.key {
            None => {return MainMenuResult::NoSelection{selected: selection}}
            Some(key) => {
                match key{
                    VirtualKeyCode::Escape => {return MainMenuResult::Selection{selected: MainMenuSelection::Exit}}
                    VirtualKeyCode::Up => {
                        let sel: i8 = selection.into();
                        // TODO: use len of menu selections instead of hard coded 3
                        let new_sel = MainMenuSelection::try_from((sel - 1i8).rem_euclid(3)).unwrap();
                        return MainMenuResult::NoSelection{selected: new_sel}
                    }
                    VirtualKeyCode::Down => {
                        let sel: i8 = selection.into();
                        // TODO: use len of menu selections instead of hard coded 3
                        let new_sel = MainMenuSelection::try_from((sel + 1i8).rem_euclid(3)).unwrap();
                        return MainMenuResult::NoSelection{selected: new_sel}
                    }
                    VirtualKeyCode:: Return => {return MainMenuResult::Selection{selected: selection}}
                    _ => {return MainMenuResult::NoSelection{selected: selection}}
                }
            }
        }
    }

    MainMenuResult::NoSelection{selected: MainMenuSelection::NewGame}
}

pub fn game_over(ctx: &mut Rltk) -> GameOverResult {
    ctx.print_color_centered(19, Palette::MAIN_FG, Palette::MAIN_BG, "You are dead.");
    ctx.print_color_centered(23, Palette::MAIN_FG, Palette::MAIN_BG, "Press any key to return to the main menu.");
    match ctx.key {
        None => { return GameOverResult::NoSelection }
        Some(_key) => { return GameOverResult::QuitToMenu }
    }
}
