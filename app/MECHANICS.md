# Dungeon Hunter — mechanics sheet

Clean room: the original's files were never opened. Numbers come from the person's spec
("spec"), from the person's follow-up message ("person"), or are estimated ("estimated").

## Units (person)
- Half-heart units; 1 heart = 2 units.
- Tile = 64 px, map grid 15 x 11 tiles per level (person).
- Sprite frames: 64x64 (hero, enemies, key, spike, skills, tiles), 96x96 (warden, gate),
  32x32 hearts (person). Animation 6-12 fps (person: manifest `recommended_fps`).

## Difficulty (person)
| Difficulty | Kaelen HP | Enemy / trap damage | Boss damage |
|---|---|---|---|
| Easy   | 10 units (5 hearts) | 1 | 2 |
| Medium | 6 units (3 hearts)  | 2 | 2 |
| Hard   | 4 units (2 hearts)  | 2 | spikes & slam 4 |
- Enemy speed: 0.8x Easy / 1.0x Medium / 1.25x Hard (person).
- i-frames after any hit: 1.0 s (person).

## Kaelen (person)
- Walk 180 px/s (diagonal normalised).
- Dagger: animation 0.25 s, damage 1, reach about half a tile (30 px, estimated).
- Dash Step (Shift): 480 px/s for 0.18 s, cooldown 1.5 s.
- Flare Pulse (Q): freezes enemies within 90 px (estimated) for 2.0 s, cooldown 8 s.
- Stone Shield (R): blocks 1 hit, cooldown 12 s.
- Phase Blink (F): 128 px teleport, cooldown 10 s.

## Enemies (person)
| Enemy | Speed | HP | Behaviour |
|---|---|---|---|
| moss_slime | 70 px/s | 2 | hops toward player when near, patrols home |
| echo_bat | 120 px/s | 2 | chases inside 220 px |
| bone_sentinel | 90 px/s | 3 | lunges with the spear every 2.0 s |
| shadow_mimic | 110 px/s | 3 | sits still, springs at 96 px |
| obsidian_warden | 80 px/s | 8 | shielded until 3 rune pillars lit, slam every 2.5 s |

## Traps (person)
- Floor spikes: 2.0 s down -> 0.5 s warning -> 1.2 s up (damage only while up).
- Crumbling tiles: collapse 0.9 s after stepped on; falling in costs trap damage and
  puts Kaelen back on the last safe tile.

## Puzzles (spec)
- Pressure plate + push block (level 2): plate lights and opens the gate.
- Rune pillars (levels 3 and 5): E to light, all 3 lit opens the seal / drops the boss shield.
- Darkness (level 4): fog with a light circle around Kaelen; braziers lit with E widen it.
- Dungeon gate: locked until keys/puzzle done, gate_opening then gate_portal, E to enter.

## Controls (spec)
- WASD or arrows to move; Space / J dagger; E interact; Shift dash; Q flare; R shield; F blink; Esc pause.

## HUD (spec)
- Hearts top-left, keys collected, level name, unlocked skill icons.
