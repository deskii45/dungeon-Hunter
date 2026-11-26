//! Menus and screens using `main_menu_bg.png`, `victory_bg.png`, and `btn_normal`/`btn_hover`.

use macroquad::prelude::*;

use crate::assets::Assets;
use crate::fx::{draw_menu_bg, draw_victory_bg};
use crate::ui::*;

const GOLD_COL: Color = ACCENT;

#[derive(Clone, Copy)]
pub struct Menu {
    pub sel: usize,
    pub mouse: Vec2,
    pub clicked: bool,
}

pub enum ScreenAction {
    None,
    Difficulty,
    Start(usize),
    Options,
    Back,
    Quit,
    Resume,
    RestartLevel,
    NextLevel,
    MainMenu,
}

pub fn main_menu(m: &mut Menu, a: &Assets, t: f32) -> ScreenAction {
    let (sw, sh) = (screen_width(), screen_height());
    draw_menu_bg(a, t);
    let mut act = ScreenAction::None;

    let bob = (t * 1.6).sin() * 3.0;
    let title_y = (sh * 0.12).max(52.0);
    label("DUNGEON", sw * 0.5, title_y + bob, 72.0, col((240, 210, 120)));
    label("HUNTER", sw * 0.5, title_y + 66.0 + bob, 56.0, col((214, 120, 80)));
    label("Petualangan pixel di lima ruang bawah tanah", sw * 0.5, title_y + 98.0 + bob, 20.0, col(TEXT));

    let bw = 340.0;
    let bh = 62.0;
    let items = ["Start Game", "Options", "Quit Game"];
    let x = sw * 0.5 - bw * 0.5;
    let y0 = sh * 0.45;
    for (i, it) in items.iter().enumerate() {
        let r = Rect::new(x, y0 + i as f32 * (bh + 14.0), bw, bh);
        let hover = r.contains(m.mouse) || m.sel == i;
        if button(a, r, it, hover, m.mouse, m.clicked) {
            m.sel = i;
            act = match i {
                0 => ScreenAction::Difficulty,
                1 => ScreenAction::Options,
                _ => ScreenAction::Quit,
            };
        }
    }
    label("WASD / Panah: pilih   Enter: OK   Esc: keluar", sw * 0.5, sh - 24.0, 18.0, col(TEXT_DIM));
    act
}

pub fn difficulty_menu(m: &mut Menu, a: &Assets, t: f32) -> ScreenAction {
    let (sw, sh) = (screen_width(), screen_height());
    draw_menu_bg(a, t);
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.58));
    label("PILIH KESULITAN", sw * 0.5, sh * 0.16, 44.0, GOLD_COL);
    let items = ["Easy", "Medium", "Hard"];
    let subs = [
        "5 Hati  -  musuh 80%  -  jebakan ringan  -  checkpoint tiap level",
        "3 Hati  -  musuh 100%  -  jebakan normal",
        "2 Hati  -  musuh 125%  -  jebakan & slam lebih sakit",
    ];
    let bw = 520.0;
    let bh = 56.0;
    let x = sw * 0.5 - bw * 0.5;
    let y0 = sh * 0.27;
    let mut act = ScreenAction::None;
    for i in 0..3 {
        let r = Rect::new(x, y0 + i as f32 * (bh + 38.0), bw, bh);
        let hover = r.contains(m.mouse) || m.sel == i;
        if button(a, r, items[i], hover, m.mouse, m.clicked) {
            m.sel = i;
            act = ScreenAction::Start(i);
        }
        label(subs[i], sw * 0.5, r.y + bh + 20.0, 17.0, col(TEXT_DIM));
    }
    let back = Rect::new(sw * 0.5 - 140.0, sh - 92.0, 280.0, 52.0);
    if button(a, back, "Kembali", back.contains(m.mouse) || m.sel == 3, m.mouse, m.clicked) {
        m.sel = 3;
        act = ScreenAction::Back;
    }
    act
}

pub struct OptionsState {
    pub volume: f32,
    pub mute: bool,
    /// 0 = WASD + Arrows, 1 = WASD only, 2 = Arrows only
    pub ctrl_mode: u8,
    /// true = crisp 4-way last-key-priority turns (default); false = 8-way diagonal
    pub four_way: bool,
    pub fullscreen: bool,
}

impl OptionsState {
    pub fn activate(&mut self, row: usize, dir: i32) {
        match row {
            0 => {
                if dir < 0 {
                    self.volume = ((self.volume - 0.1) * 10.0).round().max(0.0) / 10.0;
                } else if dir > 0 {
                    self.volume = ((self.volume + 0.1) * 10.0).round().min(10.0) / 10.0;
                } else {
                    let next = ((self.volume + 0.1) * 10.0).round() as i32;
                    self.volume = (if next > 10 { 0 } else { next }) as f32 / 10.0;
                }
            }
            1 => self.mute = !self.mute,
            2 => {
                self.ctrl_mode = if dir < 0 {
                    (self.ctrl_mode + 2) % 3
                } else {
                    (self.ctrl_mode + 1) % 3
                };
            }
            3 => self.four_way = !self.four_way,
            4 => {
                self.fullscreen = !self.fullscreen;
                set_fullscreen(self.fullscreen);
            }
            _ => {}
        }
    }

    pub fn control_label(&self) -> &'static str {
        match self.ctrl_mode {
            0 => "Tombol: WASD + Panah",
            1 => "Tombol: WASD",
            _ => "Tombol: Panah",
        }
    }

    pub fn move_mode_label(&self) -> &'static str {
        if self.four_way {
            "Gerak: 4 Arah Klasik"
        } else {
            "Gerak: 8 Arah Responsif"
        }
    }
}

pub fn options_menu(m: &mut Menu, a: &Assets, o: &mut OptionsState, t: f32) -> ScreenAction {
    let (sw, sh) = (screen_width(), screen_height());
    draw_menu_bg(a, t);
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.64));
    label("PENGATURAN", sw * 0.5, sh * 0.13, 42.0, GOLD_COL);
    let bw = 560.0;
    let bh = 46.0;
    let gap = 12.0;
    let x = sw * 0.5 - bw * 0.5;
    let y0 = sh * 0.21;
    let mut act = ScreenAction::None;

    label_left("Volume", x + 10.0, y0 + 31.0, 24.0, if m.sel == 0 { GOLD_COL } else { col(TEXT) });
    let minus = Rect::new(x + 145.0, y0 + 3.0, 56.0, bh - 6.0);
    let plus = Rect::new(x + 475.0, y0 + 3.0, 56.0, bh - 6.0);
    if button(a, minus, "-", minus.contains(m.mouse) || m.sel == 0, m.mouse, m.clicked) {
        m.sel = 0;
        o.activate(0, -1);
    }
    if button(a, plus, "+", plus.contains(m.mouse) || m.sel == 0, m.mouse, m.clicked) {
        m.sel = 0;
        o.activate(0, 1);
    }
    let bar = Rect::new(x + 211.0, y0 + 11.0, 254.0, 24.0);
    draw_rectangle(bar.x, bar.y, bar.w, bar.h, col_a((20, 16, 26), 0.9));
    draw_rectangle(bar.x, bar.y, bar.w * o.volume, bar.h, GOLD_COL);
    draw_rectangle_lines(bar.x, bar.y, bar.w, bar.h, 2.0, if m.sel == 0 { GOLD_COL } else { col(PANEL_EDGE) });
    label(&format!("{:.0}%", o.volume * 100.0), bar.x + bar.w * 0.5, bar.y + 19.0, 16.0, col((20, 16, 26)));

    let rows_y = y0 + bh + gap;
    let r = Rect::new(x, rows_y, bw, bh);
    let mute_text = format!("Suara: {}", if o.mute { "Mati" } else { "Nyala" });
    if button(a, r, &mute_text, r.contains(m.mouse) || m.sel == 1, m.mouse, m.clicked) {
        m.sel = 1;
        o.activate(1, 0);
    }
    let r = Rect::new(x, rows_y + bh + gap, bw, bh);
    let ctrl = o.control_label();
    if button(a, r, ctrl, r.contains(m.mouse) || m.sel == 2, m.mouse, m.clicked) {
        m.sel = 2;
        o.activate(2, 0);
    }
    let r = Rect::new(x, rows_y + 2.0 * (bh + gap), bw, bh);
    let mv = o.move_mode_label();
    if button(a, r, mv, r.contains(m.mouse) || m.sel == 3, m.mouse, m.clicked) {
        m.sel = 3;
        o.activate(3, 0);
    }
    let r = Rect::new(x, rows_y + 3.0 * (bh + gap), bw, bh);
    let fs = format!("Layar Penuh: {}", if o.fullscreen { "Ya" } else { "Tidak" });
    if button(a, r, &fs, r.contains(m.mouse) || m.sel == 4, m.mouse, m.clicked) {
        m.sel = 4;
        o.activate(4, 0);
    }

    let back = Rect::new(sw * 0.5 - 140.0, sh - 88.0, 280.0, 48.0);
    if button(a, back, "Kembali", back.contains(m.mouse) || m.sel == 5, m.mouse, m.clicked) {
        m.sel = 5;
        act = ScreenAction::Back;
    }
    label("Space/J/Klik: belati  E: interaksi  Shift: dash  Q: flare  R: perisai  F: blink", sw * 0.5, sh - 22.0, 17.0, col(TEXT_DIM));
    act
}

pub fn pause_menu(m: &mut Menu, a: &Assets, t: f32) -> ScreenAction {
    let (sw, sh) = (screen_width(), screen_height());
    let dim = 0.58 + (t * 0.6).sin() * 0.02;
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, dim));
    let p = Rect::new(sw * 0.5 - 240.0, sh * 0.5 - 190.0, 480.0, 380.0);
    panel(p);
    label("JEDA", sw * 0.5, p.y + 66.0, 44.0, GOLD_COL);
    let items = ["Lanjutkan", "Ulangi Level", "Menu Utama"];
    let mut act = ScreenAction::None;
    for (i, it) in items.iter().enumerate() {
        let r = Rect::new(p.x + 60.0, p.y + 100.0 + i as f32 * 76.0, p.w - 120.0, 58.0);
        if button(a, r, it, r.contains(m.mouse) || m.sel == i, m.mouse, m.clicked) {
            m.sel = i;
            act = match i {
                0 => ScreenAction::Resume,
                1 => ScreenAction::RestartLevel,
                _ => ScreenAction::MainMenu,
            };
        }
    }
    act
}

pub fn game_over(m: &mut Menu, a: &Assets, t: f32, level_name: &str) -> ScreenAction {
    let (sw, sh) = (screen_width(), screen_height());
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.15, 0.0, 0.0, 0.64 + (t * 0.5).sin() * 0.03));
    let p = Rect::new(sw * 0.5 - 260.0, sh * 0.5 - 170.0, 520.0, 330.0);
    panel(p);
    label("KAELEN TEWAS", sw * 0.5, p.y + 76.0, 48.0, col((240, 90, 90)));
    label(&format!("di {}", level_name), sw * 0.5, p.y + 112.0, 20.0, col(TEXT_DIM));
    let items = ["Ulangi Level", "Menu Utama"];
    let mut act = ScreenAction::None;
    for (i, it) in items.iter().enumerate() {
        let r = Rect::new(p.x + 70.0, p.y + 140.0 + i as f32 * 76.0, p.w - 140.0, 58.0);
        if button(a, r, it, r.contains(m.mouse) || m.sel == i, m.mouse, m.clicked) {
            m.sel = i;
            act = if i == 0 { ScreenAction::RestartLevel } else { ScreenAction::MainMenu };
        }
    }
    act
}

pub fn reward(m: &mut Menu, a: &Assets, t: f32, level_index: usize) -> ScreenAction {
    let (sw, sh) = (screen_width(), screen_height());
    draw_menu_bg(a, t);
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.52));
    let p = Rect::new(sw * 0.5 - 330.0, sh * 0.5 - 225.0, 660.0, 450.0);
    panel(p);
    let names = ["Dash Step", "Flare Pulse", "Stone Shield", "Phase Blink"];
    let keys = ["Shift", "Q", "R", "F"];
    let descs = [
        "Lompat cepat melewati jebakan dan celah.",
        "Ledakan cahaya yang membekukan musuh selama 2 detik.",
        "Perisai batu menahan 1 serangan (cooldown 12 detik).",
        "Teleportasi pendek ke arah hadap (cooldown 10 detik).",
    ];
    label("LEVEL SELESAI", sw * 0.5, p.y + 62.0, 40.0, GOLD_COL);
    label("Keterampilan baru terbuka!", sw * 0.5, p.y + 96.0, 22.0, col(TEXT));
    let i = level_index.min(3);
    a.skills[i].draw_centered(0, sw * 0.5, p.y + 185.0, 2.0);
    label(&format!("{}  [{}]", names[i], keys[i]), sw * 0.5, p.y + 272.0, 32.0, col((150, 240, 255)));
    label(descs[i], sw * 0.5, p.y + 304.0, 20.0, col(TEXT_DIM));
    let r = Rect::new(sw * 0.5 - 170.0, p.y + p.h - 92.0, 340.0, 58.0);
    if button(a, r, "Level Berikutnya", r.contains(m.mouse) || m.sel == 0, m.mouse, m.clicked) {
        return ScreenAction::NextLevel;
    }
    ScreenAction::None
}

pub fn victory(m: &mut Menu, a: &Assets, t: f32) -> ScreenAction {
    let (sw, sh) = (screen_width(), screen_height());
    draw_victory_bg(a, t);
    let bob = (t * 2.2).sin() * 4.0;
    label("KEMENANGAN!", sw * 0.5, sh * 0.14, 62.0, col((255, 226, 140)));
    label("Kaelen menaklukkan Kubah Obsidian & meraih Relic of the Hunter!", sw * 0.5, sh * 0.14 + 42.0, 21.0, col(TEXT));
    a.relic.draw_centered(0, sw * 0.5, sh * 0.30 + bob, 1.4);
    let items = ["Main Lagi", "Menu Utama"];
    let mut act = ScreenAction::None;
    for (i, it) in items.iter().enumerate() {
        let r = Rect::new(sw * 0.5 - 350.0 + i as f32 * 370.0, sh - 132.0, 330.0, 60.0);
        if button(a, r, it, r.contains(m.mouse) || m.sel == i, m.mouse, m.clicked) {
            m.sel = i;
            act = if i == 0 { ScreenAction::Start(1) } else { ScreenAction::MainMenu };
        }
    }
    label("Terima kasih sudah bermain!", sw * 0.5, sh - 28.0, 18.0, col(TEXT_DIM));
    act
}
