//! Menu and victory screen backgrounds rendered from `dungeon_hunter_assets/backgrounds/`.

use macroquad::prelude::*;

use crate::assets::Assets;

pub fn draw_menu_bg(a: &Assets, t: f32) {
    let (w, h) = (screen_width(), screen_height());
    a.main_menu_bg.draw_cover(0, w, h);
    let pulse = (t * 2.4).sin() * 0.02 + 0.06;
    draw_rectangle(0.0, 0.0, w, h, Color::new(0.02, 0.02, 0.05, pulse));
}

pub fn draw_victory_bg(a: &Assets, t: f32) {
    let (w, h) = (screen_width(), screen_height());
    a.victory_bg.draw_cover(0, w, h);
    let shimmer = (t * 2.0).sin() * 0.02 + 0.03;
    draw_circle(w * 0.59, h * 0.48, 120.0, Color::new(1.0, 0.88, 0.5, shimmer));
}
