# Paket Aset Audio 16-Bit Retro — Dungeon Hunter

Ekstrak folder `audio/` ini ke dalam folder `./dungeon_hunter_assets/` sehingga posisinya menjadi `./dungeon_hunter_assets/audio/`. Seluruh file menggunakan format **WAV 16-bit PCM 44.1 kHz** agar langsung kompatibel dengan `macroquad::audio` maupun `bevy::audio` tanpa perlu codec tambahan.

## Daftar Lengkap File Audio (29 File)

### 1. Musik Latar (`bgm/`)
- `lobby_theme.wav` *(loop)* — Musik latar gua atmosferik untuk Main Menu / Lobby.
- `dungeon_explore.wav` *(loop)* — Musik eksplorasi 16-bit untuk gameplay Level 1 sampai Level 4.
- `boss_obsidian_warden.wav` *(loop)* — Musik pertarungan intens melawan Mini-Boss di Level 5.
- `victory_fanfare.wav` *(one-shot)* — Jingle kemenangan saat menamatkan Level 5.
- `game_over_jingle.wav` *(one-shot)* — Jingle saat HP Kaelen habis.

### 2. Efek Suara Karakter (`sfx/player/`)
- `footstep_stone_1.wav` & `footstep_stone_2.wav` — Langkah kaki Kaelen bergantian di lantai batu.
- `dagger_slash.wav` — Tebasan belati.
- `player_hurt.wav` — Saat Kaelen terkena hit musuh atau jebakan.
- `player_death.wav` — Saat Kaelen gugur.

### 3. Efek Suara Skill Reward (`sfx/skills/`)
- `skill_dash.wav` — *Dash Step* (`Shift`)
- `skill_flare_pulse.wav` — *Flare Pulse* (`Q`)
- `skill_stone_shield.wav` — *Stone Shield* (`R`)
- `skill_phase_blink.wav` — *Phase Blink* (`F`)

### 4. Efek Suara Musuh & Boss (`sfx/enemies/`)
- `enemy_hit.wav` — Musuh terkena serangan Kaelen.
- `enemy_death.wav` — Musuh dikalahkan.
- `boss_slam.wav` — Hantaman gelombang batu *Obsidian Warden*.
- `boss_shield_break.wav` — Perisai boss pecah setelah 3 pilar rune aktif.

### 5. Efek Suara Item, Jebakan, Puzzle & Gerbang (`sfx/world/`)
- `key_pickup.wav` — Mengambil kunci.
- `spike_activate.wav` — Jebakan duri lantai muncul.
- `tile_crumble.wav` — Ubin lantai retak dan runtuh.
- `block_push.wav` — Mendorong balok batu puzzle.
- `pressure_plate.wav` — Pelat tekanan terinjak.
- `rune_activate.wav` — Pilar rune menyala.
- `torch_ignite.wav` — Obor dinyalakan.
- `gate_open.wav` — Gerbang gua terbuka & portal aktif.

### 6. Efek Suara Menu & UI (`sfx/ui/`)
- `ui_hover.wav` — Mengarahkan pilihan menu.
- `ui_click.wav` — Menekan tombol konfirmasi.
- `ui_back.wav` — Kembali atau Pause.

## Tips Integrasi di Rust (`macroquad`)
Pastikan fitur `audio` aktif di `Cargo.toml`:
```toml
macroquad = { version = "0.4", features = ["audio"] }
```
Lalu muat suara dengan `load_sound("./dungeon_hunter_assets/audio/bgm/lobby_theme.wav").await` dan mainkan BGM dengan `play_sound(&sound, PlaySoundParams { looped: true, volume })`.
