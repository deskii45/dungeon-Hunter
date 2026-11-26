# 🏰 Dungeon Hunter

> *A 2D pixel-art top-down puzzle-adventure game built from scratch in Rust.*

![Rust](https://img.shields.io/badge/Rust-Edition%202021-orange?logo=rust)
![Macroquad](https://img.shields.io/badge/Macroquad-0.4.16-blue)
![Platform](https://img.shields.io/badge/Platform-Windows-lightgrey?logo=windows)
![License](https://img.shields.io/badge/License-MIT-green)

---

## 📖 Latar Belakang / Background

**Dungeon Hunter** adalah game petualangan puzzle 2D dengan gaya pixel-art bertema dungeon gelap. Pemain berperan sebagai seorang penjelajah yang harus menyusuri **5 level dungeon** yang semakin berbahaya, menyelesaikan teka-teki, menghindari jebakan mematikan, dan mengalahkan musuh-musuh unik di setiap lantai.

Game ini **tidak menggunakan engine siap pakai** seperti Unity, Godot, atau Unreal Engine. Seluruh kode ditulis **dari nol (from scratch)** menggunakan bahasa pemrograman **Rust** dengan library grafis ringan **Macroquad**, sehingga menghasilkan file executable yang sangat kecil, cepat, dan responsif.

### 🎯 Fitur Utama

- **5 Level Dungeon** dengan tema unik:
  - 🌿 **Level 1 - Gua Lumut** *(Moss Cave)*: Pengenalan mekanik dasar
  - 🦇 **Level 2 - Lorong Gema** *(Echo Corridor)*: Musuh terbang & navigasi gelap
  - 🔮 **Level 3 - Ruang Rune** *(Rune Chamber)*: Puzzle rune & pilar sihir
  - 👤 **Level 4 - Labirin Bayangan** *(Shadow Labyrinth)*: Musuh mimik & jebakan tersembunyi
  - ⚔️ **Level 5 - Kubah Obsidian** *(Obsidian Vault)*: Boss fight melawan Obsidian Warden

- **5 Jenis Musuh** dengan AI dan pola serangan berbeda:
  - 🟢 Moss Slime — Bergerak lambat, membelah diri
  - 🦇 Echo Bat — Terbang, mengejar pemain
  - 💀 Bone Sentinel — Penjaga berpatroli, serangan kuat
  - 👤 Shadow Mimic — Menyamar sebagai objek, menyerang mendadak
  - 🛡️ Obsidian Warden — Boss akhir dengan mekanik shield

- **6 Jenis Jebakan & Puzzle**:
  - Floor Spikes, Pressure Plates, Crumbling Tiles
  - Rune Pillars, Torch Braziers, Dungeon Gates

- **4 Skill Aktif**: Dagger Slash, Dash, Flare Pulse, Phase Blink, Stone Shield

- **29 Audio WAV** — BGM dan SFX polyphonic (footsteps, combat, UI, ambient)

- **Kontrol Responsif** — Low-latency input menggunakan Win32 `GetAsyncKeyState`

---

## 🛠️ Teknologi / Tech Stack

| Komponen | Teknologi |
|----------|-----------|
| Bahasa Pemrograman | **Rust** (Edition 2021) |
| Framework Grafis | **Macroquad** 0.4.16 (OpenGL) |
| Audio Engine | Custom **Win32 WaveOut** PCM mixer (`winmm.dll`) |
| Input System | **Win32 GetAsyncKeyState** (zero-delay) |
| Map Format | **JSON** (parsed with `serde_json`) |
| Map Grid | 15×11 tiles, 64px per tile |
| Build System | **Cargo** + `build.bat` |

---

## 📁 Struktur Folder / Project Structure

```
dungeon-hunter/
├── README.md                    ← Anda di sini
├── app/
│   ├── build.bat                ← Script build & run
│   ├── Cargo.toml               ← Rust dependencies
│   ├── Cargo.lock
│   └── src/
│       ├── main.rs              ← Entry point & game loop
│       ├── assets.rs            ← Asset loader (sprites, tilesets, UI)
│       ├── enemy.rs             ← AI musuh & pola serangan
│       ├── fx.rs                ← Efek visual (partikel, flash)
│       ├── level.rs             ← Level loader & tile logic
│       ├── maps.rs              ← Map data parser (JSON → grid)
│       ├── pixels.rs            ← Pixel rendering utilities
│       ├── player.rs            ← Player movement & collision
│       ├── puzzle.rs            ← Puzzle/trap mechanics
│       ├── sfx.rs               ← Win32 WaveOut audio mixer
│       ├── states.rs            ← Game states (menu, play, pause, etc.)
│       ├── ui.rs                ← HUD, buttons, menu UI
│       └── world.rs             ← World simulation & game settings
│
└── dungeon_hunter_assets/
    ├── audio/
    │   ├── bgm/                 ← Background music (5 tracks)
    │   └── sfx/                 ← Sound effects (24 files)
    │       ├── enemies/
    │       ├── player/
    │       ├── skills/
    │       ├── ui/
    │       └── world/
    ├── backgrounds/             ← Level background images
    ├── maps/                    ← 5 level maps (JSON)
    ├── sprites/
    │   ├── player/              ← Hero spritesheet & frames
    │   ├── enemies/             ← 5 enemy types (sheets & frames)
    │   ├── items/               ← Keys, skill icons
    │   └── traps_and_puzzles/   ← 6 trap/puzzle types
    ├── tilesets/                ← Dungeon tileset
    └── ui/                      ← Menu buttons, HUD elements
```

---

## 🚀 Cara Menjalankan / How to Run

### Prasyarat (Prerequisites)

1. **Rust toolchain** — Install dari [https://rustup.rs/](https://rustup.rs/)
2. **Windows 10/11** — Game menggunakan Win32 API untuk audio & input
3. **Git** (opsional, untuk clone repository)

### Langkah-langkah

#### 1. Clone Repository

```bash
git clone https://github.com/deskii45/dungeon-Hunter.git
cd dungeon-Hunter
```

#### 2. Build & Run dengan build.bat (Cara Termudah)

```bash
cd app
build.bat run
```

Ini akan:
- Compile game menggunakan `cargo build --offline`
- Langsung menjalankan `target\debug\dungeon_hunter.exe`

#### 3. Atau Build Manual dengan Cargo

```bash
cd app
cargo build --release
```

Lalu jalankan executable yang dihasilkan:

```bash
.\target\release\dungeon_hunter.exe
```

### 📍 Lokasi File Executable

| Build Mode | Lokasi File |
|------------|-------------|
| Debug | `app/target/debug/dungeon_hunter.exe` |
| Release | `app/target/release/dungeon_hunter.exe` |

> **⚠️ Penting:** Jalankan game dari root folder project (`dungeon-hunter/`), agar path ke folder `dungeon_hunter_assets/` dapat ditemukan oleh game.

---

## 🎮 Kontrol / Controls

| Tombol | Aksi |
|--------|------|
| `W` | Gerak Atas |
| `A` | Gerak Kiri |
| `S` | Gerak Bawah |
| `D` | Gerak Kanan |
| `Space` | Serang / Aksi |
| `E` | Gunakan Skill |
| `Esc` | Pause / Menu |

---

## 🖼️ Opsi Command Line

```bash
dungeon_hunter.exe                       # Jalankan game normal
dungeon_hunter.exe --debug               # Mode debug (timing info di konsol)
dungeon_hunter.exe --screenshot out.png  # Render 120 frame, simpan screenshot, keluar
dungeon_hunter.exe --self-test out.json  # Jalankan scripted test, tulis hasil, keluar
```

---

## 🧪 Testing

Jalankan unit test bawaan:

```bash
cd app
cargo test --offline
```

Test yang tersedia:
- `world::tests` — World simulation & game logic
- `sfx::tests` — Audio system validation

---

## 👥 Credits

- **Game Design & Programming**: Custom engine built in Rust
- **Art Assets**: Pixel art sprites (64×64 tiles)
- **Audio**: Original WAV tracks — 5 BGM + 24 SFX
- **Framework**: [Macroquad](https://macroquad.rs/) by Fedor Logachev

---

## 📄 License

This project is open source. Feel free to explore, modify, and learn from the code.
