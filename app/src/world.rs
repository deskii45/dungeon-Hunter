//! One playable dungeon using `dungeon_hunter_assets/` sprites, tiles, and map JSONs.

use macroquad::prelude::*;

use crate::assets::Assets;
use crate::enemy::{Enemy, HitOutcome, State};
use crate::level::{draw_map, tile_center, world_to_tile, EnemyKind, KeyVariant, MapData, LAVA, PIT, TILE};
use crate::pixels::Facing;
use crate::player::{Player, FLARE_FREEZE, FLARE_RADIUS};
use crate::puzzle::*;
use crate::ui::{col, label};

pub struct Settings {
    pub difficulty: usize, // 0 easy 1 medium 2 hard
    pub max_hp: i32,
    pub enemy_speed: f32,
    pub enemy_damage: i32,
    pub trap_damage: i32,
    pub slam_damage: i32,
}

impl Settings {
    pub fn for_difficulty(d: usize) -> Settings {
        match d {
            0 => Settings { difficulty: 0, max_hp: 10, enemy_speed: 0.8, enemy_damage: 1, trap_damage: 1, slam_damage: 2 },
            1 => Settings { difficulty: 1, max_hp: 6, enemy_speed: 1.0, enemy_damage: 2, trap_damage: 2, slam_damage: 2 },
            _ => Settings { difficulty: 2, max_hp: 4, enemy_speed: 1.25, enemy_damage: 2, trap_damage: 4, slam_damage: 4 },
        }
    }

    pub fn name(&self) -> &'static str {
        ["Easy", "Medium", "Hard"][self.difficulty.min(2)]
    }
}

#[derive(Default, Clone, Copy)]
pub struct PlayEvents {
    pub attacked: bool,
    pub dashed: bool,
    pub flared: bool,
    pub shielded: bool,
    pub blinked: bool,
    pub key_picked: bool,
    pub gate_opened: bool,
    pub player_hurt: bool,
    pub enemy_hit: bool,
    pub enemy_killed: bool,
    pub boss_slam: bool,
    pub boss_shield_break: bool,
    pub spike_activated: bool,
    pub tile_crumbled: bool,
    pub block_pushed: bool,
    pub plate_activated: bool,
    pub rune_activated: bool,
    pub torch_lit: bool,
}

pub struct FloatText {
    pub pos: Vec2,
    pub text: String,
    pub life: f32,
    pub color: Color,
}

pub struct Play {
    pub index: usize,
    pub map: MapData,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub spikes: Vec<Spike>,
    pub crumble: Vec<Crumble>,
    pub blocks: Vec<PushBlock>,
    pub plates: Vec<(i32, i32)>,
    pub pillars: Vec<Pillar>,
    pub torches: Vec<Torch>,
    /// every key on the map: (x, y, variant, still_there)
    pub keys_left: Vec<(i32, i32, KeyVariant, bool)>,
    pub keys_held: i32,
    /// 0 locked, 1 opening, 2 open portal
    pub gate_state: u8,
    pub gate_t: f32,
    pub push_cd: f32,
    pub shield_break_t: f32,
    pub cam: Vec2,
    pub time: f32,
    pub floats: Vec<FloatText>,
    pub shake: f32,
    pub last_safe: (i32, i32),
    pub safe_t: f32,
    pub hint: String,
}

impl Play {
    pub fn new(index: usize, map: MapData, settings: &Settings, skills: [bool; 4]) -> Play {
        let start = tile_center(map.player_start.0, map.player_start.1);
        let player = Player::new(start, settings.max_hp, skills);
        let enemies = map
            .enemies
            .iter()
            .map(|e| Enemy::spawn(e.kind, e.x, e.y, settings.enemy_speed))
            .collect();
        let spikes = map
            .spikes
            .iter()
            .enumerate()
            .map(|(i, (x, y))| Spike::new(*x, *y, (i as f32 * 0.37) % 3.0))
            .collect();
        let crumble = map.crumbling.iter().map(|(x, y)| Crumble::new(*x, *y)).collect();
        let blocks = map.blocks.iter().map(|(x, y)| PushBlock { x: *x, y: *y }).collect();
        let pillars = map
            .pillars
            .iter()
            .map(|(x, y)| Pillar { x: *x, y: *y, lit: false, anim: 0.0 })
            .collect();
        let torches = map.torches.iter().map(|(x, y)| Torch { x: *x, y: *y, lit: false }).collect();
        let keys_left = map.keys.iter().map(|k| (k.x, k.y, k.variant, true)).collect();
        let hint = map.hint.clone();
        Play {
            index,
            player,
            enemies,
            spikes,
            crumble,
            blocks,
            plates: map.plates.clone(),
            pillars,
            torches,
            keys_left,
            keys_held: 0,
            gate_state: 0,
            gate_t: 0.0,
            push_cd: 0.0,
            shield_break_t: 0.0,
            cam: start,
            time: 0.0,
            floats: Vec::new(),
            shake: 0.0,
            last_safe: map.player_start,
            safe_t: 0.0,
            hint,
            map,
        }
    }

    pub fn primary_key_variant(&self) -> KeyVariant {
        self.keys_left.first().map(|k| k.2).unwrap_or(KeyVariant::Gold)
    }

    pub fn pillars_lit(&self) -> usize {
        self.pillars.iter().filter(|p| p.lit).count()
    }

    pub fn all_pillars_lit(&self) -> bool {
        self.pillars.is_empty() || self.pillars_lit() == self.pillars.len()
    }

    pub fn boss_shielded(&self) -> bool {
        !self.all_pillars_lit()
    }

    /// All required keys collected, level puzzles solved, and any boss defeated.
    pub fn requirement_met(&self) -> bool {
        let keys_ok = (self.keys_held as usize) >= self.map.keys_required && self.keys_left.iter().all(|k| !k.3);
        let plates_ok = self.plates.is_empty() || self.plates.iter().all(|(x, y)| self.block_on(*x, *y));
        let pillars_ok = self.all_pillars_lit();
        let boss_ok = !self.enemies.iter().any(|e| e.kind == EnemyKind::Warden && e.alive);
        keys_ok && plates_ok && pillars_ok && boss_ok
    }

    fn block_on(&self, x: i32, y: i32) -> bool {
        self.blocks.iter().any(|b| b.x == x && b.y == y)
    }

    pub fn block_can_enter(&self, x: i32, y: i32) -> bool {
        use crate::level::{FLOOR, FLOOR_ALT};
        let t = self.map.tile(x, y);
        if t != FLOOR && t != FLOOR_ALT {
            return false;
        }
        if self.blocks.iter().any(|b| b.x == x && b.y == y) {
            return false;
        }
        if self.pillars.iter().any(|p| p.x == x && p.y == y) {
            return false;
        }
        if (self.map.gate.0 - x).abs() <= 0 && (self.map.gate.1 - y).abs() <= 1 {
            return false;
        }
        if self.crumble.iter().any(|c| c.x == x && c.y == y && !c.solid()) {
            return false;
        }
        if self.enemies.iter().any(|e| e.alive && world_to_tile(e.pos) == (x, y)) {
            return false;
        }
        true
    }

    pub fn float(&mut self, pos: Vec2, text: &str, color: Color) {
        self.floats.push(FloatText { pos, text: text.to_string(), life: 1.1, color });
    }

    pub fn update(&mut self, keys: &crate::player::Keys, dt: f32, settings: &Settings) -> PlayEvents {
        let mut ev = PlayEvents::default();
        self.time += dt;
        self.safe_t += dt;
        self.push_cd = (self.push_cd - dt).max(0.0);
        self.shield_break_t = (self.shield_break_t - dt).max(0.0);

        let feet = self.player.feet_pos();
        if self.safe_t > 0.25 {
            self.safe_t = 0.0;
            let (tx, ty) = world_to_tile(feet);
            let t = self.map.tile(tx, ty);
            let crumb = self.crumble.iter().any(|c| c.x == tx && c.y == ty);
            let spike = self.spikes.iter().any(|s| s.x == tx && s.y == ty);
            if t != PIT && t != LAVA && !crumb && !spike {
                self.last_safe = (tx, ty);
            }
        }

        // Push-block detection along Kaelen's active input stack direction
        let pdir = self
            .player
            .dir_stack
            .primary()
            .map(|f| f.dir())
            .unwrap_or_else(|| keys.dir());
        let mut pushing: Option<(usize, i32, i32)> = None;
        if self.push_cd <= 0.0 && pdir.length_squared() > 0.0 {
            let (pdx, pdy) = if pdir.x.abs() >= pdir.y.abs() {
                (pdir.x.signum() as i32, 0)
            } else {
                (0, pdir.y.signum() as i32)
            };
            let cc = self.player.collider_center() + vec2(pdx as f32, pdy as f32) * 4.0;
            let pr = self.player.radius();
            for (i, b) in self.blocks.iter().enumerate() {
                let br = Rect::new(b.x as f32 * TILE + 4.0, b.y as f32 * TILE + 4.0, TILE - 8.0, TILE - 8.0);
                let cx = cc.x.clamp(br.x, br.x + br.w);
                let cy = cc.y.clamp(br.y, br.y + br.h);
                if (cc - vec2(cx, cy)).length_squared() <= (pr + 2.0) * (pr + 2.0) {
                    let nx = b.x + pdx;
                    let ny = b.y + pdy;
                    if self.block_can_enter(nx, ny) {
                        pushing = Some((i, nx, ny));
                    }
                    break;
                }
            }
        }

        // Solid dynamic obstacles (pillars + non-pushed blocks)
        let pushed_idx = pushing.map(|(i, _, _)| i);
        let mut obstacles: Vec<Rect> = Vec::with_capacity(self.blocks.len() + self.pillars.len());
        for (i, b) in self.blocks.iter().enumerate() {
            if Some(i) != pushed_idx {
                obstacles.push(Rect::new(b.x as f32 * TILE + 4.0, b.y as f32 * TILE + 4.0, TILE - 8.0, TILE - 8.0));
            }
        }
        for p in self.pillars.iter() {
            obstacles.push(Rect::new(p.x as f32 * TILE + 18.0, p.y as f32 * TILE + 22.0, 28.0, 32.0));
        }

        let res = self.player.update(&self.map, &obstacles, keys, dt);
        ev.attacked = res.attacked;
        ev.dashed = res.dashed;
        ev.flared = res.flared;
        ev.shielded = res.shielded;
        ev.blinked = res.blinked;
        if res.dashed || res.blinked {
            self.shake = self.shake.max(3.0);
        }
        if res.flared {
            for e in self.enemies.iter_mut() {
                if e.alive && (e.pos - self.player.pos).length() < FLARE_RADIUS + 14.0 {
                    e.frozen = FLARE_FREEZE;
                }
            }
            self.shake = 6.0;
            self.float(self.player.pos - vec2(0.0, 30.0), "Flare Pulse!", col((255, 230, 150)));
        }
        if res.shielded {
            self.float(self.player.pos - vec2(0.0, 30.0), "Stone Shield", col((150, 220, 255)));
        }

        for s in self.spikes.iter_mut() {
            let was_up = s.is_up();
            s.tick(dt);
            if !was_up && s.is_up() {
                ev.spike_activated = true;
            }
        }

        let (ptx, pty) = world_to_tile(self.player.feet_pos());
        for c in self.crumble.iter_mut() {
            if c.solid() && c.x == ptx && c.y == pty {
                c.trigger();
            }
            if c.tick(dt) {
                ev.tile_crumbled = true;
            }
        }

        // Complete 1-tile push-block slide with a brief step cooldown so blocks never overshoot
        if !self.player.in_dash() {
            if let Some((i, nx, ny)) = pushing {
                let b = &self.blocks[i];
                let bc = tile_center(b.x, b.y);
                let push_vec = vec2((nx - b.x) as f32, (ny - b.y) as f32);
                let offset = self.player.collider_center() - bc;
                let along = offset.x * push_vec.x + offset.y * push_vec.y;
                if along >= -TILE * 0.24 {
                    self.blocks[i].x = nx;
                    self.blocks[i].y = ny;
                    self.push_cd = 0.16;
                    let on_plate = self.plates.contains(&(nx, ny));
                    let msg = if on_plate { "Pelat aktif!" } else { "Balok bergeser" };
                    let clr = if on_plate { col((130, 225, 255)) } else { col((200, 190, 170)) };
                    self.float(tile_center(nx, ny), msg, clr);
                    ev.block_pushed = true;
                    if on_plate {
                        ev.plate_activated = true;
                    }
                }
            }
        }

        // Interact with E: torch braziers & rune pillars
        if keys.interact {
            for i in 0..self.torches.len() {
                let (x, y, lit) = (self.torches[i].x, self.torches[i].y, self.torches[i].lit);
                if !lit && (self.player.pos - tile_center(x, y)).length() < TILE * 1.15 {
                    self.torches[i].lit = true;
                    ev.torch_lit = true;
                    self.float(tile_center(x, y), "Obor menyala", col((255, 210, 120)));
                }
            }
            let was_shielded = self.boss_shielded();
            let total_pillars = self.pillars.len();
            for i in 0..total_pillars {
                let (x, y, lit) = (self.pillars[i].x, self.pillars[i].y, self.pillars[i].lit);
                if !lit && (self.player.pos - tile_center(x, y)).length() < TILE * 1.25 {
                    self.pillars[i].lit = true;
                    ev.rune_activated = true;
                    let n = self.pillars.iter().filter(|q| q.lit).count();
                    self.float(tile_center(x, y), &format!("Rune {}/{} menyala", n, total_pillars), col((150, 240, 255)));
                }
            }
            if was_shielded && !self.boss_shielded() && self.enemies.iter().any(|e| e.kind == EnemyKind::Warden && e.alive) {
                self.shield_break_t = 0.7;
                self.shake = 7.0;
                ev.boss_shield_break = true;
                if let Some(w) = self.enemies.iter().find(|e| e.kind == EnemyKind::Warden && e.alive) {
                    self.float(w.pos - vec2(0.0, 44.0), "Perisai Warden hancur!", col((255, 190, 110)));
                }
            }
        }

        // Key pickup
        let phurt = self.player.hurtbox();
        let mut picked = 0;
        let mut picks: Vec<(i32, i32)> = Vec::new();
        for k in self.keys_left.iter_mut() {
            if k.3 {
                let kc = tile_center(k.0, k.1);
                let kr = Rect::new(kc.x - 18.0, kc.y - 18.0, 36.0, 36.0);
                if phurt.overlaps(&kr) {
                    k.3 = false;
                    picked += 1;
                    picks.push((k.0, k.1));
                }
            }
        }
        for (x, y) in picks {
            self.float(tile_center(x, y), "+1 Kunci", col((240, 200, 90)));
        }
        if picked > 0 {
            ev.key_picked = true;
        }
        self.keys_held += picked;

        // Gate state transitions (`gate_locked` -> `gate_opening` -> `gate_portal`)
        if self.gate_state == 0 && self.requirement_met() {
            self.gate_state = 1;
            self.gate_t = 0.68;
            ev.gate_opened = true;
            let g = tile_center(self.map.gate.0, self.map.gate.1);
            self.float(g, "Gerbang terbuka!", col((150, 220, 255)));
        }
        if self.gate_state == 1 {
            self.gate_t -= dt;
            if self.gate_t <= 0.0 {
                self.gate_state = 2;
            }
        }

        // Enemies: contact + directional strike hitboxes
        let player_hurtbox = self.player.hurtbox();
        for i in 0..self.enemies.len() {
            if !self.enemies[i].alive {
                continue;
            }
            let was_lunge = self.enemies[i].state == State::Lunge;
            let shielded_now = self.boss_shielded();
            self.enemies[i].update(&self.map, self.player.pos, dt, shielded_now);
            let epos = self.enemies[i].pos;
            let kind = self.enemies[i].kind;
            let lunging = self.enemies[i].state == State::Lunge;
            if kind == EnemyKind::Warden && !was_lunge && lunging {
                ev.boss_slam = true;
            }
            let contact = self.enemies[i]
                .contact_box()
                .map(|cb| cb.overlaps(&player_hurtbox))
                .unwrap_or(false);
            let in_strike = self.enemies[i]
                .strike_box()
                .map(|sb| sb.overlaps(&player_hurtbox))
                .unwrap_or(false);
            if contact || in_strike {
                let dmg = if kind == EnemyKind::Warden && in_strike && lunging {
                    settings.slam_damage
                } else {
                    settings.enemy_damage
                };
                if self.player.hurt(dmg, epos) {
                    ev.player_hurt = true;
                    self.shake = 7.0;
                    self.float(
                        self.player.pos - vec2(0.0, 26.0),
                        &format!("-{:.1} Hati", dmg as f32 / 2.0),
                        col((255, 130, 130)),
                    );
                }
            }
        }

        // Dagger slash vs enemy hurtboxes (at most 1 hit per swing_id)
        if self.player.attack_active() {
            let hitbox = self.player.attack_box();
            let swing_id = self.player.swing_id;
            let ppos = self.player.pos;
            let all_lit = self.all_pillars_lit();
            for i in 0..self.enemies.len() {
                if !self.enemies[i].alive {
                    continue;
                }
                let ehurt = self.enemies[i].hurtbox();
                if hitbox.overlaps(&ehurt) {
                    let epos = self.enemies[i].pos;
                    let name = self.enemies[i].kind.name();
                    let eshielded = self.enemies[i].shielded(all_lit);
                    match self.enemies[i].hit(1, eshielded, swing_id, ppos) {
                        HitOutcome::AlreadyHit => {}
                        HitOutcome::Shielded => {
                            self.float(epos - vec2(0.0, 28.0), "Kebal - nyalakan rune", col((150, 220, 255)));
                        }
                        HitOutcome::Landed => {
                            ev.enemy_hit = true;
                            self.float(epos - vec2(0.0, 24.0), "-0.5", col((255, 245, 190)));
                            if !self.enemies[i].alive {
                                ev.enemy_killed = true;
                                self.float(epos - vec2(0.0, 44.0), name, col((220, 220, 220)));
                                self.shake = 5.0;
                            }
                        }
                    }
                }
            }
        }

        // Floor spikes vs Kaelen's boots (`feet_box`)
        let pfeet = self.player.feet_box();
        let feet_center = self.player.feet_pos();
        let mut spike_hit = false;
        for s in self.spikes.iter() {
            if let Some(sr) = s.damage_rect(TILE) {
                if sr.overlaps(&pfeet) {
                    let spike_origin = vec2(sr.x + sr.w * 0.5, sr.y + sr.h * 0.5);
                    let push_from = if (feet_center - spike_origin).length_squared() > 1.0 {
                        self.player.pos - (feet_center - spike_origin)
                    } else {
                        spike_origin
                    };
                    if self.player.hurt(settings.trap_damage, push_from) {
                        spike_hit = true;
                    }
                }
            }
        }
        if spike_hit {
            ev.player_hurt = true;
            self.shake = 7.0;
            self.float(
                self.player.pos - vec2(0.0, 26.0),
                &format!("-{:.1} Hati", settings.trap_damage as f32 / 2.0),
                col((255, 130, 130)),
            );
        }

        // Falling into a pit, collapsed crumble tile, or molten lava
        let (tx, ty) = world_to_tile(self.player.feet_pos());
        let tile_here = self.map.tile(tx, ty);
        let on_gone = self.crumble.iter().any(|c| c.x == tx && c.y == ty && !c.solid());
        if (tile_here == PIT || tile_here == LAVA || on_gone) && !self.player.in_dash() {
            if self.player.invul <= 0.0 {
                let dmg = settings.trap_damage;
                if self.player.hurt(dmg, tile_center(tx, ty)) {
                    ev.player_hurt = true;
                    self.float(self.player.pos - vec2(0.0, 26.0), &format!("-{:.1} Hati", dmg as f32 / 2.0), col((255, 130, 130)));
                }
                self.shake = 8.0;
            }
            self.player.pos = tile_center(self.last_safe.0, self.last_safe.1);
            self.player.knock = Vec2::ZERO;
        }

        // 1:1 pixel-locked camera for zero-latency directional response
        self.cam = self.player.pos;

        for f in self.floats.iter_mut() {
            f.life -= dt;
            f.pos.y -= dt * 26.0;
        }
        self.floats.retain(|f| f.life > 0.0);
        self.shake = (self.shake - dt * 20.0).max(0.0);
        ev
    }

    pub fn camera_rect(&self) -> Rect {
        self.camera_rect_for(screen_width(), screen_height(), crate::ui::HUD_BAND)
    }

    pub fn camera_rect_for(&self, sw: f32, sh: f32, band: f32) -> Rect {
        let view_h = (sh - band).max(120.0);
        let (mw, mh) = (self.map.width as f32 * TILE, self.map.height as f32 * TILE);
        let x = if mw < sw { (mw - sw) * 0.5 } else { (self.cam.x - sw * 0.5).clamp(0.0, mw - sw) };
        let mut screen_y = self.cam.y - view_h * 0.5;
        if mh <= view_h {
            screen_y = (mh - view_h) * 0.5 - 24.0;
        } else {
            screen_y = screen_y.clamp(0.0, mh - view_h);
        }
        let sx = crate::level::rand_range(-1.0, 1.0) * self.shake;
        let sy = crate::level::rand_range(-1.0, 1.0) * self.shake;
        Rect::new(x + sx, screen_y - band + sy, sw, view_h + band)
    }

    #[allow(dead_code)]
    pub fn band(&self) -> f32 {
        crate::ui::HUD_BAND
    }

    pub fn draw_in(&self, a: &Assets, view: Rect, show_hitboxes: bool) {
        let pits: Vec<(i32, i32)> = self.crumble.iter().filter(|c| !c.solid()).map(|c| (c.x, c.y)).collect();
        draw_map(a, &self.map, &view, &pits, self.time);
        self.draw_after_map(a, view, show_hitboxes);
    }

    fn draw_after_map(&self, a: &Assets, view: Rect, show_hitboxes: bool) {
        let cam = view;

        // Crumbling floor tiles (`crumble_0..3.png`)
        for c in self.crumble.iter() {
            let r = Rect::new(c.x as f32 * TILE, c.y as f32 * TILE, TILE, TILE);
            a.crumble.draw_in(c.frame(), r);
        }

        // Pressure plates (`plate_0.png` = unlit, `plate_1.png` = glowing cyan)
        for (x, y) in self.plates.iter() {
            let plate_r = Rect::new(*x as f32 * TILE + 10.0, *y as f32 * TILE + 10.0, 44.0, 44.0);
            let lit = self.block_on(*x, *y) || self.player.feet_box().overlaps(&plate_r);
            let r = Rect::new(*x as f32 * TILE, *y as f32 * TILE, TILE, TILE);
            a.plate.draw_in(if lit { 1 } else { 0 }, r);
        }

        // Floor spikes (`spikes_0..3.png`)
        for s in self.spikes.iter() {
            let r = Rect::new(s.x as f32 * TILE, s.y as f32 * TILE, TILE, TILE);
            a.spikes.draw_in(s.frame(), r);
            if s.is_warning() {
                draw_rectangle(r.x + 8.0, r.y + 8.0, TILE - 16.0, TILE - 16.0, Color::new(1.0, 0.35, 0.15, 0.14));
            }
        }

        // Enemy windup telegraphs on the floor
        for e in self.enemies.iter() {
            if !e.alive || e.state != State::Windup {
                continue;
            }
            let d = if e.lunge.length_squared() > 0.001 { e.lunge.normalize() } else { vec2(e.facing, 0.0) };
            match e.kind {
                EnemyKind::Warden => {
                    let c = e.pos + d * 20.0;
                    draw_circle(c.x, c.y, 40.0, Color::new(1.0, 0.35, 0.15, 0.20));
                    draw_circle_lines(c.x, c.y, 40.0, 2.0, Color::new(1.0, 0.55, 0.2, 0.75));
                }
                EnemyKind::Sentinel => {
                    let p0 = e.pos;
                    let p1 = e.pos + d * 36.0;
                    draw_line(p0.x, p0.y, p1.x, p1.y, 3.0, Color::new(1.0, 0.85, 0.45, 0.55));
                }
                _ => {}
            }
        }

        // Dungeon gate (`96x96`: `gate_locked`, `gate_opening_0..3`, `gate_portal_0..3`)
        let gate_c = tile_center(self.map.gate.0, self.map.gate.1);
        let gate_bottom_y = (self.map.gate.1 + 1) as f32 * TILE;
        match self.gate_state {
            0 => a.gate_locked.draw(0, gate_c.x, gate_bottom_y, 1.0),
            1 => {
                let p = (1.0 - (self.gate_t / 0.68).clamp(0.0, 1.0)) * 4.0;
                a.gate_opening.draw((p as usize).min(3), gate_c.x, gate_bottom_y, 1.0);
            }
            _ => {
                let f = ((self.time * 8.0) as usize) % 4;
                a.gate_portal.draw(f, gate_c.x, gate_bottom_y, 1.0);
            }
        }
        if self.gate_state == 0 {
            let keys_rem = self.keys_left.iter().filter(|k| k.3).count();
            let msg = if keys_rem > 0 {
                format!("Terkunci: {} kunci lagi", keys_rem)
            } else if !self.all_pillars_lit() {
                format!("Segel Rune: {}/{}", self.pillars_lit(), self.pillars.len())
            } else if self.enemies.iter().any(|e| e.kind == EnemyKind::Warden && e.alive) {
                "Kalahkan Obsidian Warden!".to_string()
            } else {
                "Terkunci: injak pelat tekanan".to_string()
            };
            label(&msg, gate_c.x, gate_bottom_y + 18.0, 16.0, Color::new(1.0, 0.86, 0.62, 0.92));
        }

        // Depth-sorted actors
        let mut actors: Vec<(f32, usize, f32, f32)> = Vec::new();
        for (i, t) in self.torches.iter().enumerate() {
            actors.push((tile_center(t.x, t.y).y + 20.0, 100 + i, t.x as f32, t.y as f32));
        }
        for (i, p) in self.pillars.iter().enumerate() {
            actors.push((tile_center(p.x, p.y).y + 22.0, 200 + i, p.x as f32, p.y as f32));
        }
        for (i, k) in self.keys_left.iter().enumerate() {
            if k.3 {
                actors.push((tile_center(k.0, k.1).y + 16.0, 300 + i, k.0 as f32, k.1 as f32));
            }
        }
        for (i, b) in self.blocks.iter().enumerate() {
            actors.push((tile_center(b.x, b.y).y + 28.0, 400 + i, b.x as f32, b.y as f32));
        }
        for (i, e) in self.enemies.iter().enumerate() {
            if e.alive {
                actors.push((e.sort_y(), 500 + i, e.pos.x, e.pos.y));
            }
        }
        actors.push((self.player.pos.y + 23.0, 900, self.player.pos.x, self.player.pos.y));
        actors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let shielded = self.boss_shielded();
        for (_, id, x, y) in actors {
            if id >= 900 {
                let p = &self.player;
                for (tp, life) in p.trail.iter() {
                    draw_circle(tp.x, tp.y, 11.0, Color::new(0.4, 0.9, 1.0, life * 1.1));
                }
                let feet_y = p.sprite_feet_y();
                let flip_left = p.facing == Facing::Left;
                if p.in_dash() {
                    let flip = p.dash_dir.x < -0.05 || (p.dash_dir.x.abs() <= 0.05 && flip_left);
                    a.hero_dash.draw_ex(p.frame_index(), p.pos.x, feet_y, 1.0, flip);
                } else if let Some(fr) = p.attack_frame() {
                    a.hero_attack.draw_ex(fr, p.pos.x, feet_y, 1.0, flip_left);
                } else if p.moving {
                    match p.facing {
                        Facing::Down => a.hero_walk_down.draw(p.frame_index(), p.pos.x, feet_y, 1.0),
                        Facing::Up => a.hero_walk_up.draw(p.frame_index(), p.pos.x, feet_y, 1.0),
                        Facing::Right => a.hero_walk_side.draw_ex(p.frame_index(), p.pos.x, feet_y, 1.0, false),
                        Facing::Left => a.hero_walk_side.draw_ex(p.frame_index(), p.pos.x, feet_y, 1.0, true),
                    }
                } else if p.facing == Facing::Up {
                    a.hero_walk_up.draw(0, p.pos.x, feet_y, 1.0);
                } else {
                    a.hero_idle.draw_ex(p.idle_frame(), p.pos.x, feet_y, 1.0, flip_left);
                }

                if p.invul > 0.0 && ((p.invul * 14.0) as i32) % 2 == 0 {
                    draw_circle_lines(p.pos.x, p.pos.y, 24.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.65));
                }
                if p.shield_on {
                    draw_circle(p.pos.x, p.pos.y, 26.0, Color::new(0.4, 0.85, 1.0, 0.14));
                    draw_circle_lines(p.pos.x, p.pos.y, 27.0, 3.0, Color::new(0.4, 0.85, 1.0, 0.9));
                }
            } else if id >= 500 {
                let i = id - 500;
                let e = &self.enemies[i];
                let flip_x = e.facing < 0.0 && e.kind != EnemyKind::Mimic;
                let af = e.anim_frame();
                let fy = e.sprite_feet_y();
                match e.kind {
                    EnemyKind::Slime => {
                        if e.state == State::Lunge {
                            a.slime_hop.draw_ex(af % 4, e.pos.x, fy, 1.0, flip_x);
                        } else {
                            a.slime_idle.draw_ex(af % 4, e.pos.x, fy, 1.0, flip_x);
                        }
                    }
                    EnemyKind::Bat => {
                        if e.swooping {
                            a.bat_swoop.draw_ex(af % 4, e.pos.x, fy, 1.0, flip_x);
                        } else {
                            a.bat_fly.draw_ex(af % 4, e.pos.x, fy, 1.0, flip_x);
                        }
                    }
                    EnemyKind::Sentinel => {
                        if e.state == State::Windup {
                            a.sentinel_thrust.draw_ex(1, e.pos.x, fy, 1.0, flip_x);
                        } else if e.state == State::Lunge {
                            a.sentinel_thrust.draw_ex(2 + (af % 2), e.pos.x, fy, 1.0, flip_x);
                        } else {
                            a.sentinel_walk.draw_ex(af % 4, e.pos.x, fy, 1.0, flip_x);
                        }
                        if e.state == State::Lunge && e.lunge.y.abs() > e.lunge.x.abs() {
                            let d = e.lunge.normalize_or_zero();
                            draw_line(e.pos.x, e.pos.y, e.pos.x + d.x * 28.0, e.pos.y + d.y * 32.0, 3.0, Color::new(0.85, 0.9, 0.96, 0.95));
                        }
                    }
                    EnemyKind::Mimic => {
                        if !e.awake {
                            a.mimic_idle.draw(af % 2, e.pos.x, fy, 1.0);
                        } else {
                            let fr = if e.state == State::Lunge { 2 + (af % 2) } else { af % 4 };
                            a.mimic_ambush.draw(fr, e.pos.x, fy, 1.0);
                        }
                    }
                    EnemyKind::Warden => {
                        if self.shield_break_t > 0.0 {
                            let fr = if self.shield_break_t > 0.35 { 0 } else { 1 };
                            a.warden_shield_break.draw_ex(fr, e.pos.x, fy, 1.0, flip_x);
                        } else if e.state == State::Windup {
                            a.warden_slam.draw_ex(1, e.pos.x, fy, 1.0, flip_x);
                        } else if e.state == State::Lunge {
                            a.warden_slam.draw_ex(2 + (af % 2), e.pos.x, fy, 1.0, flip_x);
                        } else {
                            a.warden_idle.draw_ex(af % 4, e.pos.x, fy, 1.0, flip_x);
                        }
                    }
                }

                let hb = e.hurtbox();
                if e.frozen > 0.0 {
                    draw_rectangle(hb.x - 2.0, hb.y - 2.0, hb.w + 4.0, hb.h + 4.0, Color::new(0.6, 0.85, 1.0, 0.28));
                }
                if e.hurt_flash > 0.0 {
                    draw_rectangle(hb.x, hb.y, hb.w, hb.h, Color::new(1.0, 1.0, 1.0, 0.35));
                }
                if e.damaged && e.hp > 0 && e.hp < e.max_hp {
                    let w = if e.kind == EnemyKind::Warden { 54.0 } else { 36.0 };
                    let hy = hb.y - 10.0;
                    draw_rectangle(e.pos.x - w * 0.5, hy, w, 5.0, Color::new(0.0, 0.0, 0.0, 0.75));
                    draw_rectangle(e.pos.x - w * 0.5, hy, w * e.hp as f32 / e.max_hp as f32, 5.0, col((220, 70, 70)));
                }
                if e.kind == EnemyKind::Warden && shielded {
                    draw_circle(e.pos.x, e.pos.y, 44.0, Color::new(0.35, 0.78, 1.0, 0.12));
                    draw_circle_lines(e.pos.x, e.pos.y, 44.0, 2.5, Color::new(0.45, 0.88, 1.0, 0.78));
                }
            } else if id >= 400 {
                let r = Rect::new(x * TILE, y * TILE, TILE, TILE);
                a.block.draw_in(0, r);
            } else if id >= 300 {
                let i = id - 300;
                let variant = self.keys_left[i].2;
                let f = ((self.time * 6.0 + i as f32) as usize) % 4;
                a.key_sheet(variant).draw_centered(f, x * TILE + TILE * 0.5, y * TILE + TILE * 0.5, 1.0);
            } else if id >= 200 {
                let i = id - 200;
                let p = &self.pillars[i];
                let cx = x * TILE + TILE * 0.5;
                let by = (y + 1.0) * TILE;
                if p.lit {
                    let f = ((self.time * 6.0) as usize) % 4;
                    a.pillar_active.draw(f, cx, by, 1.0);
                    draw_circle(cx, y * TILE + TILE * 0.4, 52.0, Color::new(0.5, 0.9, 1.0, 0.09));
                } else {
                    a.pillar_inactive.draw(0, cx, by, 1.0);
                }
            } else {
                let i = id - 100;
                let t = &self.torches[i];
                let cx = x * TILE + TILE * 0.5;
                let by = (y + 1.0) * TILE;
                if t.lit {
                    let f = ((self.time * 8.0) as usize) % 4;
                    a.torch_lit.draw(f, cx, by, 1.0);
                } else {
                    a.torch_unlit.draw(0, cx, by, 1.0);
                }
            }
        }

        if show_hitboxes {
            let ph = self.player.hurtbox();
            let pf = self.player.feet_box();
            draw_rectangle_lines(ph.x, ph.y, ph.w, ph.h, 1.5, Color::new(0.2, 1.0, 0.35, 0.9));
            draw_rectangle_lines(pf.x, pf.y, pf.w, pf.h, 1.5, Color::new(0.25, 0.75, 1.0, 0.95));
            if self.player.attack_active() {
                let ab = self.player.attack_box();
                draw_rectangle_lines(ab.x, ab.y, ab.w, ab.h, 2.0, Color::new(1.0, 0.95, 0.25, 0.95));
            }
            for e in self.enemies.iter() {
                if !e.alive {
                    continue;
                }
                let eh = e.hurtbox();
                draw_rectangle_lines(eh.x, eh.y, eh.w, eh.h, 1.5, Color::new(1.0, 0.55, 0.15, 0.9));
                if let Some(sb) = e.strike_box() {
                    draw_rectangle_lines(sb.x, sb.y, sb.w, sb.h, 2.0, Color::new(1.0, 0.15, 0.2, 0.95));
                }
            }
            for s in self.spikes.iter() {
                if let Some(sr) = s.damage_rect(TILE) {
                    draw_rectangle_lines(sr.x, sr.y, sr.w, sr.h, 1.5, Color::new(1.0, 0.2, 0.3, 0.9));
                }
            }
        }

        // Darkness fog (Level 4): light circle around Kaelen + lit torch braziers
        if self.map.dark {
            let lights: Vec<(Vec2, f32)> = {
                let mut v = vec![(self.player.pos, 110.0)];
                for t in self.torches.iter() {
                    if t.lit {
                        v.push((tile_center(t.x, t.y), 185.0));
                    }
                }
                v
            };
            let cols = 20;
            let rows = 13;
            let cw = cam.w / cols as f32;
            let ch = cam.h / rows as f32;
            for cy in 0..rows {
                for cx in 0..cols {
                    let c = vec2(cam.x + (cx as f32 + 0.5) * cw, cam.y + (cy as f32 + 0.5) * ch);
                    let mut dark = 1.0f32;
                    for (lp, lr) in lights.iter() {
                        let d = (c - *lp).length() / *lr;
                        dark = dark.min((d - 0.35).clamp(0.0, 1.0).powf(0.85));
                    }
                    if dark > 0.02 {
                        draw_rectangle(
                            cam.x + cx as f32 * cw - 1.0,
                            cam.y + cy as f32 * ch - 1.0,
                            cw + 2.0,
                            ch + 2.0,
                            Color::new(0.02, 0.02, 0.05, dark * 0.95),
                        );
                    }
                }
            }
            for (lp, lr) in lights.iter() {
                draw_circle(lp.x, lp.y, *lr, Color::new(1.0, 0.8, 0.45, 0.05));
            }
        }

        for f in self.floats.iter() {
            let a = (f.life / 1.1).clamp(0.0, 1.0);
            label(&f.text, f.pos.x, f.pos.y - 20.0, 18.0, Color::new(f.color.r, f.color.g, f.color.b, a));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maps;
    use crate::player::Keys;

    #[test]
    fn all_five_asset_maps_load_cleanly_with_expected_mechanics() {
        let all = maps::load_all();
        assert_eq!(all.len(), 5, "all 5 maps from dungeon_hunter_assets/maps must load");
        assert_eq!(all[0].theme_tile, "floor_moss");
        assert_eq!(all[1].plates.len(), 2);
        assert_eq!(all[1].blocks.len(), 2);
        assert_eq!(all[2].pillars.len(), 4);
        assert!(all[3].dark);
        assert!(all[4].enemies.iter().any(|e| e.kind == EnemyKind::Warden));
    }

    #[test]
    fn input_buffer_fires_attack_and_dash_when_cooldown_ends() {
        let map = maps::level_1_gua_lumut();
        let mut p = Player::new(tile_center(7, 9), 6, [true, false, false, false]);

        let mut k = Keys::default();
        k.attack = true;
        let r1 = p.update(&map, &[], &k, 1.0 / 60.0);
        assert!(r1.attacked);

        k.attack = false;
        while p.attack_cd > 0.08 {
            p.update(&map, &[], &k, 1.0 / 60.0);
        }
        let mut early_tap = Keys::default();
        early_tap.attack = true;
        let r_early = p.update(&map, &[], &early_tap, 1.0 / 60.0);
        assert!(!r_early.attacked);

        let mut fired = false;
        for _ in 0..10 {
            if p.update(&map, &[], &Keys::default(), 1.0 / 60.0).attacked {
                fired = true;
                break;
            }
        }
        assert!(fired, "buffered attack must fire immediately after cooldown");
    }

    #[test]
    fn pressing_w_while_holding_d_immediately_faces_and_moves_up_and_steps_on_frame_one() {
        let map = maps::level_1_gua_lumut();
        // (4, 9) in level_1_gua_lumut has open floor to the right (5, 9) AND above (4, 8)
        let mut p = Player::new(tile_center(4, 9), 6, [false; 4]);

        // Frame 1: Press D from idle -> immediately enters active stride frame (1 or 3, NOT standing frame 0!)
        let mut k = Keys::default();
        k.right = true;
        p.update(&map, &[], &k, 1.0 / 60.0);
        assert!(matches!(p.facing, Facing::Right));
        assert!(p.frame_index() == 1 || p.frame_index() == 3, "first frame of walking must show active stride pose");
        let y_after_d = p.pos.y;

        // Frame 2: Press W while D is still held (and simulate a delayed stale right_press from OS!)
        k.up = true;
        k.up_press = true;
        k.right_press = true; // stale WM_KEYDOWN('D') must NOT overwrite W!
        p.update(&map, &[], &k, 1.0 / 60.0);
        assert!(matches!(p.facing, Facing::Up), "stale right_press must never override newly pressed W");
        assert!(p.pos.y < y_after_d, "character must immediately move Up on frame 1 of pressing W");

        // Frame 3: Opposite key snap-tap (holding D, release W, press A -> immediately reverses Left)
        k.up = false;
        k.up_press = false;
        k.right_press = false;
        k.left = true;
        k.left_press = true;
        let x_before_a = p.pos.x;
        p.update(&map, &[], &k, 1.0 / 60.0);
        assert!(matches!(p.facing, Facing::Left));
        assert!(p.pos.x < x_before_a);
    }

    #[test]
    fn preturn_holding_d_and_w_slides_along_wall_then_turns_up_into_corridor() {
        let map = maps::level_1_gua_lumut();
        // Row y=9 has wall above at x=6,7,8 (row y=8), then open floor above at x=9 (9, 8)
        let mut p = Player::new(tile_center(7, 9), 6, [false; 4]);
        let mut k = Keys::default();
        k.right = true;
        p.update(&map, &[], &k, 1.0 / 60.0);

        k.up = true;
        let start_y = p.pos.y;
        for _ in 0..90 {
            p.update(&map, &[], &k, 1.0 / 60.0);
        }
        assert!(p.pos.y < start_y - 40.0, "pre-holding W while sliding Right must turn Up once x=9 opens");
    }

    #[test]
    fn spike_only_hits_feet_not_head_from_tile_below() {
        let map = maps::level_1_gua_lumut();
        let settings = Settings::for_difficulty(1);
        let mut play = Play::new(0, map, &settings, [false; 4]);

        // Spike at (3, 7) in level_1_gua_lumut.json
        let sx = 3;
        let sy = 7;
        for s in play.spikes.iter_mut() {
            if s.x == sx && s.y == sy {
                s.t = SPIKE_DOWN + SPIKE_WARN + 0.3;
            } else {
                s.t = 0.0;
            }
        }

        // Stand on safe floor (3, 8) just below spike (3, 7): head overlaps top edge, feet on (3, 8)
        play.player.pos = vec2(sx as f32 * TILE + 32.0, (sy + 1) as f32 * TILE + 6.0);
        let hp_before = play.player.hp;
        play.update(&Keys::default(), 1.0 / 60.0, &settings);
        assert_eq!(play.player.hp, hp_before, "head near spike tile must not trigger floor spike");

        // Place feet directly on active spike cluster on (3, 7)
        play.player.pos = vec2(sx as f32 * TILE + 32.0, sy as f32 * TILE + 18.0);
        play.update(&Keys::default(), 1.0 / 60.0, &settings);
        assert!(play.player.hp < hp_before, "feet stepping onto active spikes must deal trap damage");
    }

    #[test]
    fn dagger_hits_enemy_once_per_swing_and_sentinel_lunges_real_distance() {
        let map = maps::level_1_gua_lumut();
        let mut e = Enemy::spawn(EnemyKind::Sentinel, 7, 7, 1.0);
        let start = e.pos;
        let player_pos = start + vec2(80.0, 0.0);

        for _ in 0..45 {
            e.update(&map, player_pos, 1.0 / 60.0, false);
        }
        assert!((e.pos - start).length() > 25.0);

        let hp0 = e.hp;
        assert!(matches!(e.hit(1, false, 7, player_pos), HitOutcome::Landed));
        assert!(matches!(e.hit(1, false, 7, player_pos), HitOutcome::AlreadyHit));
        assert_eq!(e.hp, hp0 - 1);
    }

    #[test]
    fn level_2_push_block_can_enter_pressure_plate() {
        let map = maps::level_2_ruang_batu();
        let settings = Settings::for_difficulty(1);
        let play = Play::new(1, map, &settings, [true, false, false, false]);
        for &(px, py) in &play.plates {
            assert!(play.block_can_enter(px, py), "push block must enter pressure plate ({}, {})", px, py);
        }
    }
}
