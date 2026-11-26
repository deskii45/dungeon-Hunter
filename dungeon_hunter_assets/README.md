# Paket Aset 2D Pixel — Dungeon Hunter (Rust)

Paket aset ini siap langsung disalin ke folder `assets/` pada proyek Rust Anda (**Bevy** atau **Macroquad**). Seluruh sprite animasi disediakan dalam **dua format sekaligus**:
1. **Frame PNG terpisah (`frames/*_0.png`, `*_1.png`, ...)** — tinggal ganti indeks gambar berdasarkan timer (*frame-by-frame swapping*).
2. **Horizontal Spritesheet (`sheets/*_sheet.png`)** — siap dipakai sebagai *Texture Atlas* untuk performa render optimal.

## Struktur Folder
- `backgrounds/` — Latar layar utama (`main_menu_bg.png`) dan layar tamat (`victory_bg.png`) ukuran 1376×768.
- `sprites/player/` — Karakter utama **Kaelen** (64×64 px): `kaelen_idle`, `kaelen_walk_down`, `kaelen_walk_up`, `kaelen_walk_side`, `kaelen_attack`, `kaelen_dash`.
- `sprites/enemies/` — 5 tipe musuh beserta animasinya:
  - `moss_slime/` (64×64): `idle`, `hop`
  - `echo_bat/` (64×64): `fly`, `swoop`
  - `bone_sentinel/` (64×64): `walk`, `thrust`
  - `shadow_mimic/` (64×64): `chest_idle`, `ambush`
  - `obsidian_warden/` (96×96 Mini-Boss): `idle`, `slam`, `shield_break`
- `sprites/items/keys/` — 4 varian kunci beranimasi melayang & berkilau (`key_gold`, `key_bronze`, `key_silver`, `key_crystal`, masing-masing 4 frame).
- `sprites/items/skills/` — 5 ikon skill reward (`dash_step.png`, `flare_pulse.png`, `stone_shield.png`, `phase_blink.png`, `relic_of_hunter.png`).
- `sprites/traps_and_puzzles/` — `floor_spikes` (4 frame), `crumbling_tile` (4 frame), `push_block.png`, `pressure_plate` (2 frame), `rune_pillar` (inactive + 4 frame aktif), `torch_brazier` (unlit + 4 frame api), dan `dungeon_gate` (96×96: locked, opening 4 frame, portal 4 frame).
- `tilesets/` — `cave_tileset_64x64.png` dan tile individual (`floor_moss`, `floor_echo`, `floor_rune`, `floor_shadow`, `floor_obsidian`, `wall`, `pit`, `lava_0`, `lava_1`).
- `ui/` — Hati HP (`heart_full`, `heart_half`, `heart_empty`) dan tombol menu (`btn_normal`, `btn_hover`).
- `maps/` — 5 file peta grid JSON siap parse (`level_1_gua_lumut.json` s.d. `level_5_kubah_obsidian.json`) + pratinjau PNG di `maps/previews/`.
- `manifest.json` — Daftar indeks lengkap seluruh aset, jumlah frame, dimensi piksel, dan rekomendasi FPS.

## Contoh Logika Animasi Pindah Gambar di Rust (Frame Swapping)
```rust
struct SpriteAnim {
    frames: Vec<TextureHandle>, // atau Texture2D di Macroquad
    fps: f32,
    timer: f32,
    current_frame: usize,
}

impl SpriteAnim {
    fn update(&mut self, dt: f32) {
        self.timer += dt;
        let frame_duration = 1.0 / self.fps;
        if self.timer >= frame_duration {
            self.timer -= frame_duration;
            self.current_frame = (self.current_frame + 1) % self.frames.len();
        }
    }
}
```
Tips: pastikan filter tekstur diset ke **Nearest** (`FilterMode::Nearest` di Macroquad atau `ImagePlugin::default_nearest()` di Bevy) agar piksel tetap tajam tanpa blur.
