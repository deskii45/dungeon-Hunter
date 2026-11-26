//! Loads all textures, animations, tiles, UI, and backgrounds from `dungeon_hunter_assets/`.

use std::path::{Path, PathBuf};

use crate::level::KeyVariant;
use crate::pixels::SpriteSheet;

/// Locate `dungeon_hunter_assets/` regardless of whether the game is launched from the workspace
/// root, `app/`, `build.bat`, or directly from `target/debug` or `target/release`.
pub fn find_assets_dir() -> PathBuf {
    let candidates = [
        PathBuf::from("dungeon_hunter_assets"),
        PathBuf::from("../dungeon_hunter_assets"),
        PathBuf::from("assets"),
        PathBuf::from("../assets"),
    ];
    for c in &candidates {
        if c.join("manifest.json").is_file() {
            return c.clone();
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut cur = exe.parent();
        while let Some(dir) = cur {
            let p1 = dir.join("dungeon_hunter_assets");
            if p1.join("manifest.json").is_file() {
                return p1;
            }
            let p2 = dir.join("assets");
            if p2.join("manifest.json").is_file() {
                return p2;
            }
            cur = dir.parent();
        }
    }
    PathBuf::from("dungeon_hunter_assets")
}

fn seq(root: &Path, prefix: &str, count: usize) -> SpriteSheet {
    let paths: Vec<PathBuf> = (0..count).map(|i| root.join(format!("{}_{}.png", prefix, i))).collect();
    SpriteSheet::from_png_files(&paths)
}

fn single(root: &Path, rel: &str) -> SpriteSheet {
    SpriteSheet::from_single_png(&root.join(rel))
}

#[allow(dead_code)]
pub struct Assets {
    pub main_menu_bg: SpriteSheet,
    pub victory_bg: SpriteSheet,
    pub btn_normal: SpriteSheet,
    pub btn_hover: SpriteSheet,
    pub hearts: SpriteSheet,
    pub tutorial_popups: [SpriteSheet; 5],
    pub tutorial_frame: SpriteSheet,
    pub key_h: SpriteSheet,

    pub hero_idle: SpriteSheet,
    pub hero_walk_down: SpriteSheet,
    pub hero_walk_up: SpriteSheet,
    pub hero_walk_side: SpriteSheet,
    pub hero_attack: SpriteSheet,
    pub hero_dash: SpriteSheet,

    pub slime_idle: SpriteSheet,
    pub slime_hop: SpriteSheet,
    pub bat_fly: SpriteSheet,
    pub bat_swoop: SpriteSheet,
    pub sentinel_walk: SpriteSheet,
    pub sentinel_thrust: SpriteSheet,
    pub mimic_idle: SpriteSheet,
    pub mimic_ambush: SpriteSheet,
    pub warden_idle: SpriteSheet,
    pub warden_slam: SpriteSheet,
    pub warden_shield_break: SpriteSheet,

    pub key_gold: SpriteSheet,
    pub key_bronze: SpriteSheet,
    pub key_silver: SpriteSheet,
    pub key_crystal: SpriteSheet,
    pub skills: [SpriteSheet; 4],
    pub relic: SpriteSheet,

    pub spikes: SpriteSheet,
    pub crumble: SpriteSheet,
    pub block: SpriteSheet,
    pub plate: SpriteSheet,
    pub pillar_inactive: SpriteSheet,
    pub pillar_active: SpriteSheet,
    pub torch_unlit: SpriteSheet,
    pub torch_lit: SpriteSheet,
    pub gate_locked: SpriteSheet,
    pub gate_opening: SpriteSheet,
    pub gate_portal: SpriteSheet,

    pub floor_moss: SpriteSheet,
    pub floor_echo: SpriteSheet,
    pub floor_rune: SpriteSheet,
    pub floor_shadow: SpriteSheet,
    pub floor_obsidian: SpriteSheet,
    pub wall: SpriteSheet,
    pub pit: SpriteSheet,
    pub lava: SpriteSheet,
}

impl Assets {
    pub fn build() -> Assets {
        let root = find_assets_dir();
        let r = root.as_path();
        Assets {
            main_menu_bg: single(r, "backgrounds/main_menu_bg.png"),
            victory_bg: single(r, "backgrounds/victory_bg.png"),
            btn_normal: single(r, "ui/btn_normal.png"),
            btn_hover: single(r, "ui/btn_hover.png"),
            hearts: SpriteSheet::from_png_files(&[
                r.join("ui/heart_full.png"),
                r.join("ui/heart_half.png"),
                r.join("ui/heart_empty.png"),
            ]),
            tutorial_popups: [
                single(r, "ui/tutorial/popup_level_1.png"),
                single(r, "ui/tutorial/popup_level_2.png"),
                single(r, "ui/tutorial/popup_level_3.png"),
                single(r, "ui/tutorial/popup_level_4.png"),
                single(r, "ui/tutorial/popup_level_5.png"),
            ],
            tutorial_frame: single(r, "ui/tutorial/panel_frame_700x420.png"),
            key_h: single(r, "ui/tutorial/keycaps/key_h.png"),

            hero_idle: seq(r, "sprites/player/frames/kaelen_idle", 4),
            hero_walk_down: seq(r, "sprites/player/frames/kaelen_walk_down", 4),
            hero_walk_up: seq(r, "sprites/player/frames/kaelen_walk_up", 4),
            hero_walk_side: seq(r, "sprites/player/frames/kaelen_walk_side", 4),
            hero_attack: seq(r, "sprites/player/frames/kaelen_attack", 4),
            hero_dash: seq(r, "sprites/player/frames/kaelen_dash", 2),

            slime_idle: seq(r, "sprites/enemies/moss_slime/frames/idle", 4),
            slime_hop: seq(r, "sprites/enemies/moss_slime/frames/hop", 4),
            bat_fly: seq(r, "sprites/enemies/echo_bat/frames/fly", 4),
            bat_swoop: seq(r, "sprites/enemies/echo_bat/frames/swoop", 4),
            sentinel_walk: seq(r, "sprites/enemies/bone_sentinel/frames/walk", 4),
            sentinel_thrust: seq(r, "sprites/enemies/bone_sentinel/frames/thrust", 4),
            mimic_idle: seq(r, "sprites/enemies/shadow_mimic/frames/chest_idle", 2),
            mimic_ambush: seq(r, "sprites/enemies/shadow_mimic/frames/ambush", 4),
            warden_idle: seq(r, "sprites/enemies/obsidian_warden/frames/idle", 4),
            warden_slam: seq(r, "sprites/enemies/obsidian_warden/frames/slam", 4),
            warden_shield_break: seq(r, "sprites/enemies/obsidian_warden/frames/shield_break", 2),

            key_gold: seq(r, "sprites/items/keys/frames/key_gold", 4),
            key_bronze: seq(r, "sprites/items/keys/frames/key_bronze", 4),
            key_silver: seq(r, "sprites/items/keys/frames/key_silver", 4),
            key_crystal: seq(r, "sprites/items/keys/frames/key_crystal", 4),
            skills: [
                single(r, "sprites/items/skills/dash_step.png"),
                single(r, "sprites/items/skills/flare_pulse.png"),
                single(r, "sprites/items/skills/stone_shield.png"),
                single(r, "sprites/items/skills/phase_blink.png"),
            ],
            relic: single(r, "sprites/items/skills/relic_of_hunter.png"),

            spikes: seq(r, "sprites/traps_and_puzzles/floor_spikes/frames/spikes", 4),
            crumble: seq(r, "sprites/traps_and_puzzles/crumbling_tile/frames/crumble", 4),
            block: single(r, "sprites/traps_and_puzzles/push_block.png"),
            plate: seq(r, "sprites/traps_and_puzzles/pressure_plate/frames/plate", 2),
            pillar_inactive: single(r, "sprites/traps_and_puzzles/rune_pillar/pillar_inactive.png"),
            pillar_active: seq(r, "sprites/traps_and_puzzles/rune_pillar/frames/pillar_active", 4),
            torch_unlit: single(r, "sprites/traps_and_puzzles/torch_brazier/brazier_unlit.png"),
            torch_lit: seq(r, "sprites/traps_and_puzzles/torch_brazier/frames/brazier_lit", 4),
            gate_locked: single(r, "sprites/traps_and_puzzles/dungeon_gate/gate_locked.png"),
            gate_opening: seq(r, "sprites/traps_and_puzzles/dungeon_gate/frames/gate_opening", 4),
            gate_portal: seq(r, "sprites/traps_and_puzzles/dungeon_gate/frames/gate_portal", 4),

            floor_moss: single(r, "tilesets/tiles/floor_moss.png"),
            floor_echo: single(r, "tilesets/tiles/floor_echo.png"),
            floor_rune: single(r, "tilesets/tiles/floor_rune.png"),
            floor_shadow: single(r, "tilesets/tiles/floor_shadow.png"),
            floor_obsidian: single(r, "tilesets/tiles/floor_obsidian.png"),
            wall: single(r, "tilesets/tiles/wall.png"),
            pit: single(r, "tilesets/tiles/pit.png"),
            lava: SpriteSheet::from_png_files(&[
                r.join("tilesets/tiles/lava_0.png"),
                r.join("tilesets/tiles/lava_1.png"),
            ]),
        }
    }

    pub fn key_sheet(&self, variant: KeyVariant) -> &SpriteSheet {
        match variant {
            KeyVariant::Gold => &self.key_gold,
            KeyVariant::Bronze => &self.key_bronze,
            KeyVariant::Silver => &self.key_silver,
            KeyVariant::Crystal => &self.key_crystal,
        }
    }

    pub fn floor_sheet(&self, theme: &str) -> &SpriteSheet {
        match theme {
            "floor_moss" => &self.floor_moss,
            "floor_echo" => &self.floor_echo,
            "floor_rune" => &self.floor_rune,
            "floor_shadow" => &self.floor_shadow,
            "floor_obsidian" => &self.floor_obsidian,
            _ => &self.floor_moss,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tutorial_ui_assets_exist_and_have_valid_png_headers() {
        let root = find_assets_dir();
        for lvl in 1..=5 {
            let p = root.join(format!("ui/tutorial/popup_level_{}.png", lvl));
            let bytes = std::fs::read(&p).unwrap_or_else(|e| panic!("missing {:?}: {}", p, e));
            assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"), "invalid PNG signature in {:?}", p);
            let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
            let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
            assert_eq!((w, h), (700, 420), "unexpected dimensions for {:?}", p);
        }

        let frame = root.join("ui/tutorial/panel_frame_700x420.png");
        let fbytes = std::fs::read(&frame).expect("missing panel_frame_700x420.png");
        assert!(fbytes.starts_with(b"\x89PNG\r\n\x1a\n"));

        let keys = [
            "key_w", "key_a", "key_s", "key_d", "key_wasd", "key_arrows", "key_space", "key_j",
            "key_e", "key_shift", "key_q", "key_r", "key_f", "key_esc", "key_h",
        ];
        for k in keys {
            let kp = root.join(format!("ui/tutorial/keycaps/{}.png", k));
            let kbytes = std::fs::read(&kp).unwrap_or_else(|e| panic!("missing {:?}: {}", kp, e));
            assert!(kbytes.starts_with(b"\x89PNG\r\n\x1a\n"), "invalid PNG signature in {:?}", kp);
        }
    }
}
