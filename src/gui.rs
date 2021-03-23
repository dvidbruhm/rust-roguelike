use rltk::{Rltk, Point, VirtualKeyCode};
use hecs::*;
use resources::*;
use crate::components::{CombatStats, Name, Position, InBackpack};
use crate::gamelog::{GameLog};
use crate::map::{Map};
use crate::{Palette};

pub fn draw_gui(world: &World, res: &Resources, ctx: &mut Rltk) {
    ctx.print_color(0, 10, Palette::MAIN_FG, Palette::MAIN_BG, "─".repeat(80));

    let player_id: &Entity = &res.get::<Entity>().unwrap();
    let player_stats = world.get::<CombatStats>(*player_id).unwrap();
    let hp_gui = format!("{} / {} HP", player_stats.hp, player_stats.max_hp);

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
        let mut padding_x = arrow_pos.x + 1;
        let mut left_x = mouse_pos.0 + 4;
        let mut y = mouse_pos.1;
        if mouse_pos.0 > map.width / 2 {
            sign = -1;
            arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            padding_x = arrow_pos.x;
            left_x = mouse_pos.0 - width;
        }

        for s in tooltip.iter() {
            ctx.print_color(left_x, y, Palette::MAIN_FG, Palette::MAIN_BG, s);
            let padding = (width - s.len() as i32) - 1;
            for i in 0..padding {
                ctx.print_color(padding_x + sign * i, y, Palette::MAIN_FG, Palette::MAIN_BG, " ");
            }
            y += 1;
        }
        ctx.print_color(arrow_pos.x, arrow_pos.y, Palette::MAIN_FG, Palette::MAIN_BG, "->");
    }
}

#[derive(PartialEq)]
pub enum ItemMenuResult {Cancel, NoResponse, Selected, Dropped}

pub fn show_inventory(world: &mut World, res: &mut Resources, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_id = res.get::<Entity>().unwrap();
    let mut query = world.query::<(&InBackpack, &Name)>();
    static mut DROPPING: bool = false;

    let inventory = query.iter().filter(|item| item.1.0.owner == *player_id);
    let count = inventory.count();
    let mut y = 25 - (count / 2);
    ctx.draw_box(15, y - 2, 31, count + 3, Palette::MAIN_FG, Palette::MAIN_BG);
    let mut title = "Inventory: use";
    unsafe {
        if DROPPING {
            title = "Inventory: drop";
        }
    }
    ctx.print_color(18, y - 2, Palette::MAIN_FG, Palette::MAIN_BG, title);

    let mut useable: Vec<Entity> = Vec::new();
    for (j, (id, (_pack, name))) in &mut world.query::<(&InBackpack, &Name)>().iter().filter(|item| item.1.0.owner == *player_id).enumerate() {
        ctx.set(17, y, Palette::MAIN_FG, Palette::MAIN_BG, rltk::to_cp437('('));
        ctx.set(18, y, Palette::COLOR_0, Palette::MAIN_BG, 97 + j as rltk::FontCharType);
        ctx.set(19, y, Palette::MAIN_FG, Palette::MAIN_BG, rltk::to_cp437(')'));

        ctx.print_color(21, y, Palette::MAIN_FG, Palette::MAIN_BG, &name.name.to_string());
        useable.push(id);
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::I => { (ItemMenuResult::Cancel, None) }
                VirtualKeyCode::D => { unsafe {DROPPING = !DROPPING;} (ItemMenuResult::NoResponse, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        unsafe {
                            if !DROPPING { return (ItemMenuResult::Selected, Some(useable[selection as usize])) }
                            else { return (ItemMenuResult::Dropped, Some(useable[selection as usize])) }
                        }
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}
