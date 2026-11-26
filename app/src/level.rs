//! Map data parsed from `dungeon_hunter_assets/maps/*.json`, tile collision, and tileset rendering.

use macroquad::prelude::*;
use serde_json::Value;

use crate::assets::Assets;

/// Tile size in pixels (64x64).
pub const TILE: f32 = 64.0;
pub const MAP_W: i32 = 15;
pub const MAP_H: i32 = 11;

pub const FLOOR: u8 = 0;
pub const WALL: u8 = 1;
pub const FLOOR_ALT: u8 = 2;
pub const PIT: u8 = 3;
pub const LAVA: u8 = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyVariant {
    Gold,
    Bronze,
    Silver,
    Crystal,
}

impl KeyVariant {
    pub fn from_tag(s: &str) -> KeyVariant {
        match s {
            "key_bronze" => KeyVariant::Bronze,
            "key_silver" => KeyVariant::Silver,
            "key_crystal" => KeyVariant::Crystal,
            _ => KeyVariant::Gold,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct KeySpawn {
    pub x: i32,
    pub y: i32,
    pub variant: KeyVariant,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyKind {
    Slime,
    Bat,
    Sentinel,
    Mimic,
    Warden,
}

impl EnemyKind {
    pub fn name(self) -> &'static str {
        match self {
            EnemyKind::Slime => "moss_slime",
            EnemyKind::Bat => "echo_bat",
            EnemyKind::Sentinel => "bone_sentinel",
            EnemyKind::Mimic => "shadow_mimic",
            EnemyKind::Warden => "obsidian_warden",
        }
    }

    pub fn from_tag(s: &str) -> EnemyKind {
        match s {
            "moss_slime" => EnemyKind::Slime,
            "echo_bat" => EnemyKind::Bat,
            "bone_sentinel" => EnemyKind::Sentinel,
            "shadow_mimic" => EnemyKind::Mimic,
            _ => EnemyKind::Warden,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EnemySpawn {
    pub kind: EnemyKind,
    pub x: i32,
    pub y: i32,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct MapData {
    pub id: String,
    pub name: String,
    pub theme_tile: String,
    pub challenge_type: String,
    pub reward_skill: String,
    pub width: i32,
    pub height: i32,
    /// 0 floor, 1 wall, 2 floor alt, 3 pit, 4 lava
    pub tiles: Vec<Vec<u8>>,
    pub player_start: (i32, i32),
    pub gate: (i32, i32),
    pub keys_required: usize,
    pub keys: Vec<KeySpawn>,
    pub enemies: Vec<EnemySpawn>,
    pub spikes: Vec<(i32, i32)>,
    pub crumbling: Vec<(i32, i32)>,
    pub blocks: Vec<(i32, i32)>,
    pub plates: Vec<(i32, i32)>,
    pub pillars: Vec<(i32, i32)>,
    pub torches: Vec<(i32, i32)>,
    pub dark: bool,
    pub hint: String,
}

fn xy_obj_or_pair(v: &Value) -> (i32, i32) {
    if v.is_object() {
        (
            v["x"].as_i64().unwrap_or(1) as i32,
            v["y"].as_i64().unwrap_or(1) as i32,
        )
    } else {
        (
            v.get(0).and_then(|n| n.as_i64()).unwrap_or(1) as i32,
            v.get(1).and_then(|n| n.as_i64()).unwrap_or(1) as i32,
        )
    }
}

fn default_hint_for(id_num: i64, challenge: &str) -> String {
    match id_num {
        1 => "Ambil kunci emas dan hindari ranjau duri untuk membuka Gerbang. Hadiah: Dash Step.".to_string(),
        2 => "Dorong 2 balok batu ke pelat tekanan & kumpulkan 2 kunci perunggu. Hadiah: Flare Pulse.".to_string(),
        3 => "Aktifkan ke-4 pilar rune dengan E & kumpulkan 2 kunci perak. Hadiah: Stone Shield.".to_string(),
        4 => "Nyalakan obor dengan E untuk menembus kabut gelap & cari 3 kunci kristal (awas Mimic!).".to_string(),
        5 => "Nyalakan 3 pilar rune dengan E untuk menghancurkan perisai, lalu kalahkan Obsidian Warden!".to_string(),
        _ => challenge.to_string(),
    }
}

impl MapData {
    /// Parse a map from the `dungeon_hunter_assets/maps/*.json` schema.
    pub fn from_json(v: &Value) -> Result<MapData, String> {
        let width = v["width"].as_i64().unwrap_or(MAP_W as i64) as i32;
        let height = v["height"].as_i64().unwrap_or(MAP_H as i64) as i32;

        let tiles: Vec<Vec<u8>> = if let Some(terrain) = v["terrain"].as_array() {
            terrain
                .iter()
                .map(|row| {
                    row.as_array()
                        .map(|r| {
                            r.iter()
                                .map(|cell| match cell.as_str().unwrap_or("wall") {
                                    "floor" => FLOOR,
                                    "pit" => PIT,
                                    "lava" => LAVA,
                                    _ => WALL,
                                })
                                .collect()
                        })
                        .unwrap_or_default()
                })
                .collect()
        } else if let Some(grid) = v["tiles"].as_array() {
            grid.iter()
                .map(|row| row.as_array().map(|r| r.iter().map(|c| c.as_u64().unwrap_or(1) as u8).collect()).unwrap_or_default())
                .collect()
        } else {
            return Err("map needs terrain or tiles".to_string());
        };

        if tiles.len() != height as usize || tiles.iter().any(|r| r.len() != width as usize) {
            return Err(format!("tile grid is not {}x{}", width, height));
        }

        let id_num = v["id"].as_i64().unwrap_or(0);
        let id = if let Some(s) = v["id"].as_str() {
            s.to_string()
        } else {
            format!("level_{}", id_num)
        };
        let name = v["name"].as_str().unwrap_or("Dungeon").to_string();
        let theme_tile = v["theme_tile"].as_str().unwrap_or("floor_moss").to_string();
        let challenge_type = v["challenge_type"].as_str().unwrap_or("").to_string();
        let reward_skill = v["reward_skill"].as_str().unwrap_or("").to_string();

        let player_start = if !v["spawn"].is_null() {
            xy_obj_or_pair(&v["spawn"])
        } else {
            xy_obj_or_pair(&v["player_start"])
        };
        let gate = xy_obj_or_pair(&v["gate"]);

        let mut spikes = Vec::new();
        let mut crumbling = Vec::new();
        if let Some(traps) = v["traps"].as_array() {
            for t in traps {
                let pos = xy_obj_or_pair(t);
                match t["type"].as_str().unwrap_or("") {
                    "floor_spikes" => spikes.push(pos),
                    "crumbling_tile" => crumbling.push(pos),
                    _ => {}
                }
            }
        }

        let mut plates = Vec::new();
        let mut blocks = Vec::new();
        let mut pillars = Vec::new();
        let mut torches = Vec::new();
        if let Some(puzzles) = v["puzzles"].as_array() {
            for pz in puzzles {
                let pos = xy_obj_or_pair(pz);
                match pz["type"].as_str().unwrap_or("") {
                    "pressure_plate" => plates.push(pos),
                    "push_block" => blocks.push(pos),
                    "rune_pillar" => pillars.push(pos),
                    "torch_brazier" => torches.push(pos),
                    _ => {}
                }
            }
        }

        let mut keys = Vec::new();
        let mut enemies = Vec::new();
        if let Some(ents) = v["entities"].as_array() {
            for ent in ents {
                let (x, y) = xy_obj_or_pair(ent);
                match ent["type"].as_str().unwrap_or("") {
                    "key" => {
                        let variant = KeyVariant::from_tag(ent["variant"].as_str().unwrap_or("key_gold"));
                        keys.push(KeySpawn { x, y, variant });
                    }
                    "enemy" => {
                        let kind = EnemyKind::from_tag(ent["enemy_kind"].as_str().unwrap_or("moss_slime"));
                        enemies.push(EnemySpawn { kind, x, y });
                    }
                    _ => {}
                }
            }
        }

        let keys_required = v["gate"]["keys_required"]
            .as_u64()
            .map(|n| n as usize)
            .unwrap_or_else(|| keys.len());
        let dark = v["darkness_fog"].as_bool().or_else(|| v["dark"].as_bool()).unwrap_or(false);
        let hint = v["hint"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| default_hint_for(id_num, &challenge_type));

        Ok(MapData {
            id,
            name,
            theme_tile,
            challenge_type,
            reward_skill,
            width,
            height,
            tiles,
            player_start,
            gate,
            keys_required,
            keys,
            enemies,
            spikes,
            crumbling,
            blocks,
            plates,
            pillars,
            torches,
            dark,
            hint,
        })
    }

    pub fn tile(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return WALL;
        }
        self.tiles[y as usize][x as usize]
    }
}

pub fn rand_range(a: f32, b: f32) -> f32 {
    use macroquad::rand::gen_range;
    gen_range(a, b)
}

#[allow(dead_code)]
pub fn tile_blocks(t: u8) -> bool {
    t == WALL
}

#[allow(dead_code)]
pub fn is_solid(map: &MapData, x: i32, y: i32) -> bool {
    tile_blocks(map.tile(x, y))
}

/// Is a circle at world position `p` with `r` overlapping a solid wall tile?
pub fn circle_hits_wall(map: &MapData, p: Vec2, r: f32) -> bool {
    circle_hits_tile_pred(map, p, r, |t| t == WALL)
}

/// Used by ground enemies so they do not step into open pits or molten lava.
pub fn circle_hits_wall_or_hazard(map: &MapData, p: Vec2, r: f32) -> bool {
    circle_hits_tile_pred(map, p, r, |t| t == WALL || t == PIT || t == LAVA)
}

fn circle_hits_tile_pred<F: Fn(u8) -> bool>(map: &MapData, p: Vec2, r: f32, pred: F) -> bool {
    let x0 = ((p.x - r) / TILE).floor() as i32;
    let x1 = ((p.x + r) / TILE).floor() as i32;
    let y0 = ((p.y - r) / TILE).floor() as i32;
    let y1 = ((p.y + r) / TILE).floor() as i32;
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            if pred(map.tile(tx, ty)) {
                let rx = tx as f32 * TILE;
                let ry = ty as f32 * TILE;
                let cx = p.x.clamp(rx, rx + TILE);
                let cy = p.y.clamp(ry, ry + TILE);
                if (p - vec2(cx, cy)).length_squared() < r * r {
                    return true;
                }
            }
        }
    }
    false
}

pub fn tile_center(x: i32, y: i32) -> Vec2 {
    vec2(x as f32 * TILE + TILE * 0.5, y as f32 * TILE + TILE * 0.5)
}

pub fn world_to_tile(p: Vec2) -> (i32, i32) {
    ((p.x / TILE).floor() as i32, (p.y / TILE).floor() as i32)
}

/// Draw the dungeon terrain tiles directly from the authentic 64x64 tile PNGs in `Assets`.
pub fn draw_map(a: &Assets, map: &MapData, cam: &Rect, pits: &[(i32, i32)], time: f32) {
    let floor_tex = a.floor_sheet(&map.theme_tile);
    let lava_frame = ((time * 3.0) as usize) % 2;
    let x0 = ((cam.x / TILE).floor() as i32 - 1).max(0);
    let y0 = ((cam.y / TILE).floor() as i32 - 1).max(0);
    let x1 = (((cam.x + cam.w) / TILE).ceil() as i32 + 1).min(map.width);
    let y1 = (((cam.y + cam.h) / TILE).ceil() as i32 + 1).min(map.height);
    for ty in y0..y1 {
        for tx in x0..x1 {
            let rect = Rect::new(tx as f32 * TILE, ty as f32 * TILE, TILE, TILE);
            let t = map.tile(tx, ty);
            if t == WALL {
                a.wall.draw_in(0, rect);
            } else if t == PIT || pits.contains(&(tx, ty)) {
                a.pit.draw_in(0, rect);
            } else if t == LAVA {
                a.lava.draw_in(lava_frame, rect);
            } else {
                floor_tex.draw_in(0, rect);
            }
        }
    }
}
