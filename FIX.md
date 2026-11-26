# FIX.md — why `craft-anything` failed with `os error 2`, and the control fix

## 1. `os error 2` on every write

```
C:\Users\Dero\OneDrive\Documents\Craft Anything\dungeon-hunter\.craft-anything.tmp-27764-482:
  The system cannot find the file specified. (os error 2)
```

No file was missing. `C:\Users\Dero\OneDrive\Documents` is a OneDrive reparse
point whose cloud provider is not running (`onedrive.exe` absent, `OneDrive
Updater Service` and `OneSyncSvc` stopped), and on that volume:

| operation in the old project tree | result |
| --- | --- |
| create a new file or folder | `ERROR_FILE_NOT_FOUND` → Rust `os error 2` |
| write to an existing file (`WriteFile`, `fs::write`, `WriteAllBytes`) | 0 bytes / `ERROR_BAD_FD` / access denied |
| delete or rename | access denied |
| open for read, `read_dir` | works |

Reproduced from Python, Node, PowerShell/.NET and raw `Win32`, so the app's
temp-file-then-rename strategy is fine — the folder itself cannot take writes.
`C:\Users\Dero\OneDrive` root, `Desktop` and `Attachments` were unaffected.

**Fix applied:** the project now lives at **`C:\Projects\dungeon-hunter`** and
`craft.json.target.path` points there. `app\build.bat` builds in place.

## 2. Why WASD / arrows did not control the game

Three separate defects, all in the input path:

1. **The shipped game started in the menu, not on the map.** `ScreenAction::Start`
   called `start_level(0)` and left `mode` as `Menu`, so the player held a map that
   was never updated or shown. Every key went to the menu handler, and movement
   never ran. Fixed: `Start` enters `Mode::Play`.
2. **Menu presses were sampled twice per frame.** `read_raw()` read the held keys,
   then `handle_global_keys()` re-read `is_key_pressed()` later in the same frame.
   macroquad clears press edges at frame end, so whichever read came second saw
   nothing — the menu looked deaf. Fixed: the frame's presses are sampled once in
   `read_raw()` (carried in `Raw` as `up_press`, `down_press`, `enter_press`,
   `esc_press`) and every consumer reads that same sample.
3. **A keyboard tap on "Quit Game" ended the process,** which is why the window
   vanished while testing controls. Fixed: only the mouse quits; keyboard on that
   row does nothing.

Also fixed along the way:

- the play camera put the map under the HUD band (`camera_rect` now returns a
  view `band` px taller and the camera is shifted by the same amount), so the
  player is no longer half hidden under the hearts;
- the Options rows are operable from the keyboard (`OptionsState::activate`);
- Esc pauses and resumes, and steps back out of Difficulty/Options;
- `--self-test` and `--screenshot` run headless with a fixed 1/60 s step, so a CI
  run finishes in seconds instead of waiting for vsync (measured: self-test 3.5 s
  with all steps passing, screenshot 1.0 s).

## 3. Compile fixes

- `states.rs`: `GOLD` was ambiguous between `macroquad::prelude` and `crate::ui::*`
  and `col(GOLD)` was type-invalid → `const GOLD_COL: Color = ACCENT`.
- `main.rs`: `Image::try_export_png` does not exist in macroquad 0.4 → `export_png`.
- `level.rs`: removed the unused `use rand::Rng;`.

## Verifying

```bat
cd C:\Projects\dungeon-hunter\app
build.bat                       :: cargo build -> BUILD OK
target\debug\dungeon_hunter.exe --self-test st.json    :: all steps pass
target\debug\dungeon_hunter.exe --screenshot shot.png
build.bat run                   :: play: Enter starts, WASD/arrows move, Esc pauses
```

The old OneDrive folder still holds the pre-move copy; its volume refuses writes,
so delete it by hand once OneDrive is running again.
