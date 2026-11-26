//! Loads the five dungeon maps directly from `dungeon_hunter_assets/maps/*.json`.

use serde_json::Value;
use std::path::Path;

use crate::assets::find_assets_dir;
use crate::level::MapData;

const LEVEL_FILES: [&str; 5] = [
    "maps/level_1_gua_lumut.json",
    "maps/level_2_lorong_gema.json",
    "maps/level_3_ruang_rune.json",
    "maps/level_4_labirin_bayangan.json",
    "maps/level_5_kubah_obsidian.json",
];

fn parse_map_file(path: &Path) -> Result<MapData, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{:?}: {}", path, e))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{:?}: {}", path, e))?;
    MapData::from_json(&v)
}

/// Load all 5 maps listed in `manifest.json` (falling back to the 5 canonical filenames).
pub fn load_all() -> Vec<MapData> {
    let root = find_assets_dir();
    let manifest_path = root.join("manifest.json");
    if let Ok(text) = std::fs::read_to_string(&manifest_path) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(arr) = v["maps"].as_array() {
                let mut out = Vec::with_capacity(arr.len());
                for item in arr {
                    if let Some(rel) = item["json"].as_str() {
                        match parse_map_file(&root.join(rel)) {
                            Ok(m) => out.push(m),
                            Err(e) => eprintln!("dungeon_hunter: map load error: {}", e),
                        }
                    }
                }
                if !out.is_empty() {
                    return out;
                }
            }
        }
    }

    LEVEL_FILES
        .iter()
        .map(|rel| parse_map_file(&root.join(rel)).expect("level JSON loads from dungeon_hunter_assets/maps"))
        .collect()
}

#[allow(dead_code)]
pub fn level_1_gua_lumut() -> MapData {
    let root = find_assets_dir();
    parse_map_file(&root.join(LEVEL_FILES[0])).expect("level 1 JSON loads")
}

#[allow(dead_code)]
pub fn level_2_ruang_batu() -> MapData {
    let root = find_assets_dir();
    parse_map_file(&root.join(LEVEL_FILES[1])).expect("level 2 JSON loads")
}
