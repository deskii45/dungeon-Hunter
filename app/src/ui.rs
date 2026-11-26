#![allow(dead_code)]
//! Menus, pixel-art buttons (`btn_normal.png` / `btn_hover.png`), and the in-game HUD.

use macroquad::prelude::*;

use crate::assets::Assets;
use crate::pixels::{Rgba, SpriteSheet};

pub const BG: Rgba = (16, 12, 24);
pub const PANEL: Rgba = (34, 26, 46);
pub const PANEL_EDGE: Rgba = (96, 78, 120);
pub const GOLD: Rgba = (240, 200, 90);
pub const ACCENT: Color = Color::new(240.0 / 255.0, 200.0 / 255.0, 90.0 / 255.0, 1.0);
pub const TEXT: Rgba = (232, 226, 214);
pub const TEXT_DIM: Rgba = (176, 168, 184);
pub const GREEN: Rgba = (120, 220, 140);
pub const RED: Rgba = (220, 90, 90);

/// Height of the HUD band at the top of the screen.
pub const HUD_BAND: f32 = 100.0;

pub fn col(c: Rgba) -> Color {
    Color::new(c.0 as f32 / 255.0, c.1 as f32 / 255.0, c.2 as f32 / 255.0, 1.0)
}

pub fn col_a(c: Rgba, a: f32) -> Color {
    Color::new(c.0 as f32 / 255.0, c.1 as f32 / 255.0, c.2 as f32 / 255.0, a)
}

/// Text with a crisp 2 px pixel drop-shadow.
pub fn label(text: &str, x: f32, y: f32, size: f32, c: Color) {
    let fs = macroquad::text::measure_text(text, None, size as u16, 1.0);
    draw_text(text, x - fs.width * 0.5 + 2.0, y + 2.0, size, Color::new(0.0, 0.0, 0.0, c.a * 0.8));
    draw_text(text, x - fs.width * 0.5, y, size, c);
}

pub fn label_left(text: &str, x: f32, y: f32, size: f32, c: Color) {
    draw_text(text, x + 2.0, y + 2.0, size, Color::new(0.0, 0.0, 0.0, c.a * 0.8));
    draw_text(text, x, y, size, c);
}

/// Menu button rendered using `ui/btn_normal.png` and `ui/btn_hover.png`.
pub fn button(a: &Assets, rect: Rect, text: &str, hover: bool, mouse: Vec2, clicked: bool) -> bool {
    draw_rectangle(rect.x + 3.0, rect.y + 4.0, rect.w, rect.h, Color::new(0.0, 0.0, 0.0, 0.42));
    if hover {
        a.btn_hover.draw_in(0, rect);
    } else {
        a.btn_normal.draw_in(0, rect);
    }
    let tc = if hover { col(GOLD) } else { col(TEXT) };
    let font_sz = (rect.h * 0.50).clamp(20.0, 30.0);
    label(text, rect.x + rect.w * 0.5, rect.y + rect.h * 0.5 + font_sz * 0.30, font_sz, tc);
    clicked && rect.contains(mouse)
}

pub fn panel(r: Rect) {
    draw_rectangle(r.x + 6.0, r.y + 8.0, r.w, r.h, Color::new(0.0, 0.0, 0.0, 0.55));
    draw_rectangle(r.x, r.y, r.w, r.h, col_a(PANEL, 0.94));
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 3.0, col(PANEL_EDGE));
    draw_rectangle(r.x + 6.0, r.y + 6.0, r.w - 12.0, 2.0, col_a(GOLD, 0.35));
}

/// The in-game HUD: hearts, key count, level name, skill slots.
pub struct HudState<'a> {
    pub hearts: &'a SpriteSheet,
    pub key_icon: &'a SpriteSheet,
    pub key_frame: usize,
    pub skills: &'a [SpriteSheet; 4],
    pub hp: i32,
    pub max_hp: i32,
    pub keys: i32,
    pub keys_total: i32,
    pub level_name: &'a str,
    pub unlocked: [bool; 4],
    pub cooldowns: [f32; 4],
    pub max_cooldowns: [f32; 4],
    pub keys_bound: [&'a str; 4],
}

pub fn draw_hud(h: &HudState, sw: f32) {
    draw_rectangle(0.0, 0.0, sw, HUD_BAND - 4.0, Color::new(0.04, 0.03, 0.06, 0.92));
    draw_rectangle(0.0, HUD_BAND - 6.0, sw, 2.0, col_a(PANEL_EDGE, 0.7));

    // hearts: 2 half-heart units each (`heart_full`, `heart_half`, `heart_empty`)
    let n = ((h.max_hp + 1) / 2).max(1);
    for i in 0..n {
        let units = (h.hp - i * 2).clamp(0, 2);
        let kind = if units >= 2 { 0 } else if units == 1 { 1 } else { 2 };
        let r = Rect::new(16.0 + i as f32 * 36.0, 12.0, 32.0, 32.0);
        h.hearts.draw_in(kind, r);
    }

    // keys collected vs required
    let ky = 52.0;
    h.key_icon.draw_in(h.key_frame, Rect::new(14.0, ky - 2.0, 36.0, 36.0));
    label_left(&format!("{} / {}", h.keys, h.keys_total), 52.0, ky + 24.0, 25.0, col(GOLD));

    // level name, top centre
    label(h.level_name, sw * 0.5, 38.0, 26.0, col(TEXT));

    // skill slots, top right (each PNG in `sprites/items/skills/` is 64x64 with its own ornate gold border)
    let slot = 52.0;
    let gap = 8.0;
    let total = 4.0 * slot + 3.0 * gap;
    let x0 = sw - total - 16.0;
    for i in 0..4 {
        let x = x0 + i as f32 * (slot + gap);
        let y = 12.0;
        let r = Rect::new(x, y, slot, slot);
        draw_rectangle(r.x, r.y, r.w, r.h, col_a((20, 16, 26), 0.85));
        if h.unlocked[i] {
            h.skills[i].draw_in(0, r);
            let cd = h.cooldowns[i];
            if cd > 0.01 {
                let frac = (cd / h.max_cooldowns[i]).clamp(0.0, 1.0);
                draw_rectangle(r.x, r.y, r.w, r.h * frac, Color::new(0.0, 0.0, 0.0, 0.62));
                label(&format!("{:.0}", cd.ceil()), x + slot * 0.5, y + slot * 0.5 + 7.0, 20.0, col(TEXT));
            }
            label(h.keys_bound[i], x + slot * 0.5, y + slot + 16.0, 15.0, col(TEXT_DIM));
        } else {
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, col(PANEL_EDGE));
            label("?", x + slot * 0.5, y + slot * 0.5 + 8.0, 24.0, col(TEXT_DIM));
        }
    }
}

pub fn toast(text: &str, sw: f32, sh: f32, alpha: f32) {
    let a = alpha.clamp(0.0, 1.0);
    let w = (sw - 40.0).min(820.0);
    let r = Rect::new((sw - w) * 0.5, sh - 74.0, w, 44.0);
    draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.0, 0.0, 0.0, 0.65 * a));
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.5, col_a(PANEL_EDGE, 0.6 * a));
    label(text, r.x + r.w * 0.5, r.y + 29.0, 20.0, col_a(TEXT, a));
}
