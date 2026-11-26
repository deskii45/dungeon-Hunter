# Paket Aset UI Popup Tutorial Kontrol (Level 1–5) — Dungeon Hunter

Ekstrak isi ZIP ini ke dalam folder `./dungeon_hunter_assets/` sehingga membentuk folder `./dungeon_hunter_assets/ui/tutorial/`.

## Isi Paket (22 File)
1. **5 Panel Popup Tutorial Siap Pakai (`640x420` px, PNG transparan)**:
   - `popup_level_1.png` — Kontrol dasar (`WASD/Panah`, `SPACE/J` Serang, `E` Interaksi/Gerbang, `ESC` Pause).
   - `popup_level_2.png` — Skill baru `SHIFT` (*Dash Step*) & puzzle dorong balok batu.
   - `popup_level_3.png` — Skill baru `Q` (*Flare Pulse*) & cara membuka/mengaktifkan 4 Pilar Rune dengan `E`.
   - `popup_level_4.png` — Skill baru `R` (*Stone Shield*), menyalakan Obor dengan `E`, & waspada *Shadow Mimic*.
   - `popup_level_5.png` — Skill baru `F` (*Phase Blink*), pecahkan perisai Boss via 3 Pilar Rune (`E`), & kalahkan *Obsidian Warden*.
2. **Aset Modular (Opsional untuk UI Dinamis)**:
   - `panel_frame_640x400.png` — Bingkai panel kosong bergaya pixel-art batu & lis emas.
   - `keycaps/` — 15 ikon tombol keyboard pixel-art (`W`, `A`, `S`, `D`, `WASD`, `PANAH`, `SPACE`, `J`, `E`, `SHIFT`, `Q`, `R`, `F`, `ESC`, `H`).
3. **Metadata**:
   - `tutorial_manifest.json` — Daftar indeks file popup dan keycaps.

## Cara Pakai di Game (Rust / Macroquad)
- Tampilkan `popup_level_{n}.png` di tengah layar (dengan lapisan hitam semi-transparan di belakangnya) secara otomatis saat pemain pertama kali memasuki Level `n`.
- Tutup popup saat pemain menekan `Enter` atau `Space`.
- Izinkan pemain membuka kembali popup level aktif kapan saja dengan menekan tombol `H`.
