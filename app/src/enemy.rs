//! The five enemy types calibrated to `dungeon_hunter_assets/sprites/enemies/`.

use macroquad::prelude::*;

use crate::level::{circle_hits_wall, circle_hits_wall_or_hazard, tile_center, EnemyKind, MapData, TILE};

pub struct Enemy {
    pub kind: EnemyKind,
    pub pos: Vec2,
    pub home: Vec2,
    pub hp: i32,
    pub max_hp: i32,
    pub speed: f32,
    pub facing: f32,
    pub state: State,
    pub timer: f32,
    pub frozen: f32,
    pub hurt_flash: f32,
    pub alive: bool,
    pub lunge: Vec2,
    pub knock: Vec2,
    pub last_hit_swing: u32,
    pub anim: f32,
    pub awake: bool,
    pub damaged: bool,
    pub swooping: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HitOutcome {
    AlreadyHit,
    Shielded,
    Landed,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Idle,
    Walk,
    Windup,
    Lunge,
    Rest,
}

impl Enemy {
    pub fn spawn(kind: EnemyKind, x: i32, y: i32, speed_mult: f32) -> Enemy {
        let (hp, speed) = match kind {
            EnemyKind::Slime => (2, 70.0),
            EnemyKind::Bat => (2, 120.0),
            EnemyKind::Sentinel => (3, 90.0),
            EnemyKind::Mimic => (3, 110.0),
            EnemyKind::Warden => (8, 80.0),
        };
        let pos = tile_center(x, y);
        Enemy {
            kind,
            pos,
            home: pos,
            hp,
            max_hp: hp,
            speed: speed * speed_mult,
            facing: 1.0,
            state: State::Walk,
            timer: 0.0,
            frozen: 0.0,
            hurt_flash: 0.0,
            alive: true,
            lunge: Vec2::ZERO,
            knock: Vec2::ZERO,
            last_hit_swing: 0,
            anim: crate::level::rand_range(0.0, 3.0),
            awake: kind == EnemyKind::Warden,
            damaged: false,
            swooping: false,
        }
    }

    pub fn radius(&self) -> f32 {
        match self.kind {
            EnemyKind::Warden => 26.0,
            EnemyKind::Sentinel => 13.0,
            EnemyKind::Bat => 13.0,
            EnemyKind::Slime => 15.0,
            EnemyKind::Mimic => 17.0,
        }
    }

    /// Bottom-centre Y passed to `SpriteSheet::draw_ex` so `self.pos` aligns with each PNG sprite's visual body centre.
    pub fn sprite_feet_y(&self) -> f32 {
        match self.kind {
            EnemyKind::Slime => self.pos.y + 24.0,
            EnemyKind::Bat => self.pos.y + 34.0,
            EnemyKind::Sentinel => self.pos.y + 31.0,
            EnemyKind::Mimic => self.pos.y + 24.0,
            EnemyKind::Warden => self.pos.y + 46.0,
        }
    }

    /// Depth-sort foot Y coordinate (matches the built-in ground shadow row in each PNG).
    pub fn sort_y(&self) -> f32 {
        match self.kind {
            EnemyKind::Slime => self.pos.y + 18.0,
            EnemyKind::Bat => self.pos.y + 22.0,
            EnemyKind::Sentinel => self.pos.y + 24.0,
            EnemyKind::Mimic => self.pos.y + 18.0,
            EnemyKind::Warden => self.pos.y + 40.0,
        }
    }

    /// Hurtbox fitted to the non-transparent pixel silhouette of each enemy PNG.
    pub fn hurtbox(&self) -> Rect {
        match self.kind {
            EnemyKind::Slime => Rect::new(self.pos.x - 18.0, self.pos.y - 14.0, 36.0, 28.0),
            EnemyKind::Bat => Rect::new(self.pos.x - 18.0, self.pos.y - 11.0, 36.0, 22.0),
            EnemyKind::Sentinel => Rect::new(self.pos.x - 11.0, self.pos.y - 21.0, 22.0, 44.0),
            EnemyKind::Mimic => {
                if !self.awake {
                    Rect::new(self.pos.x - 19.0, self.pos.y - 14.0, 38.0, 28.0)
                } else {
                    Rect::new(self.pos.x - 19.0, self.pos.y - 24.0, 38.0, 38.0)
                }
            }
            EnemyKind::Warden => Rect::new(self.pos.x - 28.0, self.pos.y - 36.0, 56.0, 76.0),
        }
    }

    /// Tightened inner core for direct body-contact damage.
    pub fn contact_box(&self) -> Option<Rect> {
        if self.kind == EnemyKind::Mimic && !self.awake {
            return None;
        }
        let h = self.hurtbox();
        Some(Rect::new(h.x + 3.0, h.y + 3.0, (h.w - 6.0).max(8.0), (h.h - 6.0).max(8.0)))
    }

    /// Active weapon / strike hitbox during `State::Lunge`.
    pub fn strike_box(&self) -> Option<Rect> {
        if self.state != State::Lunge {
            return None;
        }
        let d = if self.lunge.length_squared() > 0.001 {
            self.lunge.normalize()
        } else {
            vec2(self.facing, 0.0)
        };
        match self.kind {
            EnemyKind::Sentinel => {
                if d.x.abs() >= d.y.abs() {
                    let sx = if d.x >= 0.0 { self.pos.x + 2.0 } else { self.pos.x - 36.0 };
                    Some(Rect::new(sx, self.pos.y - 9.0, 34.0, 18.0))
                } else {
                    let sy = if d.y >= 0.0 { self.pos.y + 2.0 } else { self.pos.y - 36.0 };
                    Some(Rect::new(self.pos.x - 10.0, sy, 20.0, 34.0))
                }
            }
            EnemyKind::Mimic => {
                let c = self.pos + d * 14.0;
                Some(Rect::new(c.x - 17.0, c.y - 15.0, 34.0, 30.0))
            }
            EnemyKind::Slime => Some(Rect::new(self.pos.x - 17.0, self.pos.y - 12.0, 34.0, 24.0)),
            EnemyKind::Warden => {
                let c = self.pos + d * 20.0;
                Some(Rect::new(c.x - 40.0, c.y - 36.0, 80.0, 72.0))
            }
            EnemyKind::Bat => None,
        }
    }

    pub fn hit(&mut self, dmg: i32, shielded: bool, swing_id: u32, from: Vec2) -> HitOutcome {
        if swing_id != 0 && self.last_hit_swing == swing_id {
            return HitOutcome::AlreadyHit;
        }
        self.last_hit_swing = swing_id;
        if shielded {
            return HitOutcome::Shielded;
        }
        self.hp -= dmg;
        self.hurt_flash = 0.15;
        self.awake = true;
        self.damaged = true;
        let away = (self.pos - from).normalize_or_zero();
        let kb = if self.kind == EnemyKind::Warden { 45.0 } else { 135.0 };
        self.knock = away * kb;
        if self.hp <= 0 {
            self.hp = 0;
            self.alive = false;
        }
        HitOutcome::Landed
    }

    pub fn update(&mut self, map: &MapData, player: Vec2, dt: f32, warden_shielded: bool) {
        let fps = match self.kind {
            EnemyKind::Slime => if self.state == State::Lunge { 8.0 } else { 6.0 },
            EnemyKind::Bat => if self.swooping { 12.0 } else { 10.0 },
            EnemyKind::Sentinel => if self.state == State::Lunge { 10.0 } else { 8.0 },
            EnemyKind::Mimic => if self.awake { 10.0 } else { 3.0 },
            EnemyKind::Warden => if self.state == State::Lunge { 8.0 } else { 6.0 },
        };
        self.anim += dt * fps;
        self.hurt_flash = (self.hurt_flash - dt).max(0.0);
        if self.knock.length_squared() > 1.0 {
            self.step(map, self.knock * dt);
            self.knock *= (1.0 - dt * 10.0).max(0.0);
        }
        if self.frozen > 0.0 {
            self.frozen -= dt;
            return;
        }
        self.timer -= dt;
        let to_player = player - self.pos;
        let dist = to_player.length();
        if self.state != State::Lunge && to_player.x.abs() > 1.0 {
            self.facing = if to_player.x > 0.0 { 1.0 } else { -1.0 };
        }

        match self.kind {
            EnemyKind::Slime => {
                if dist < 160.0 {
                    self.awake = true;
                }
                if self.state == State::Lunge {
                    self.step(map, self.lunge * (self.speed * 2.1) * dt);
                    if self.timer <= 0.0 {
                        self.state = State::Walk;
                        self.timer = 0.55;
                    }
                } else if self.timer <= 0.0 {
                    let d = if self.awake { to_player.normalize_or_zero() } else { (self.home - self.pos).normalize_or_zero() };
                    self.lunge = if d.length_squared() > 0.0 { d } else { vec2(self.facing, 0.0) };
                    if self.lunge.x.abs() > 0.05 {
                        self.facing = self.lunge.x.signum();
                    }
                    self.state = State::Lunge;
                    self.timer = 0.32;
                }
            }
            EnemyKind::Bat => {
                if dist < 220.0 || self.awake {
                    self.awake = true;
                    self.swooping = dist < 120.0;
                    let wob = vec2((self.anim * 1.8).sin(), (self.anim * 2.4).cos()) * 0.25;
                    let d = if dist > 10.0 {
                        (to_player.normalize_or_zero() + wob).normalize_or_zero()
                    } else {
                        to_player.normalize_or_zero()
                    };
                    let mult = if self.swooping { 1.15 } else { 1.0 };
                    self.step(map, d * self.speed * mult * dt);
                } else {
                    self.swooping = false;
                    let a = self.anim * 0.35;
                    let target = self.home + vec2(a.cos(), a.sin()) * (TILE * 0.9);
                    self.step(map, (target - self.pos).normalize_or_zero() * self.speed * 0.4 * dt);
                }
            }
            EnemyKind::Sentinel => {
                if self.state == State::Lunge {
                    self.step(map, self.lunge * (self.speed * 2.4) * dt);
                    if self.timer <= 0.0 {
                        self.state = State::Rest;
                        self.timer = 1.45;
                    }
                } else if self.state == State::Windup {
                    if self.timer <= 0.0 {
                        self.state = State::Lunge;
                        self.timer = 0.20;
                    }
                } else if self.state == State::Rest {
                    if self.timer <= 0.0 {
                        self.state = State::Walk;
                    }
                } else if self.timer <= 0.0 && dist < 130.0 {
                    self.state = State::Windup;
                    self.timer = 0.35;
                    self.lunge = to_player.normalize_or_zero();
                    if self.lunge.x.abs() > 0.05 {
                        self.facing = self.lunge.x.signum();
                    }
                } else if dist > 115.0 {
                    self.step(map, to_player.normalize_or_zero() * self.speed * 0.6 * dt);
                } else if dist < 44.0 {
                    self.step(map, -to_player.normalize_or_zero() * self.speed * 0.35 * dt);
                }
            }
            EnemyKind::Mimic => {
                if !self.awake {
                    if dist < 96.0 {
                        self.awake = true;
                        self.timer = 0.28;
                    }
                } else if self.state == State::Lunge {
                    self.step(map, self.lunge * (self.speed * 2.4) * dt);
                    if self.timer <= 0.0 {
                        self.state = State::Rest;
                        self.timer = 0.5;
                    }
                } else if self.state == State::Rest {
                    if self.timer <= 0.0 {
                        self.state = State::Walk;
                    }
                } else {
                    let d = to_player.normalize_or_zero();
                    if dist < 86.0 && self.timer <= 0.0 {
                        self.state = State::Lunge;
                        self.lunge = d;
                        if d.x.abs() > 0.05 {
                            self.facing = d.x.signum();
                        }
                        self.timer = 0.22;
                    } else {
                        self.step(map, d * self.speed * dt);
                    }
                }
            }
            EnemyKind::Warden => {
                let _ = warden_shielded;
                if self.state == State::Lunge {
                    self.step(map, self.lunge * (self.speed * 2.0) * dt);
                    if self.timer <= 0.0 {
                        self.state = State::Rest;
                        self.timer = 2.0;
                    }
                } else if self.state == State::Windup {
                    if self.timer <= 0.0 {
                        self.state = State::Lunge;
                        self.timer = 0.25;
                    }
                } else if self.state == State::Rest {
                    if self.timer <= 0.0 {
                        self.state = State::Walk;
                    }
                } else if dist < 110.0 && self.timer <= 0.0 {
                    self.state = State::Windup;
                    self.timer = 0.5;
                    self.lunge = to_player.normalize_or_zero();
                    if self.lunge.x.abs() > 0.05 {
                        self.facing = self.lunge.x.signum();
                    }
                } else if dist > 42.0 {
                    self.step(map, to_player.normalize_or_zero() * self.speed * dt);
                }
            }
        }
    }

    fn step(&mut self, map: &MapData, d: Vec2) {
        if d.length_squared() < 0.0001 {
            return;
        }
        let r = self.radius();
        let flying = self.kind == EnemyKind::Bat;
        let blocked = |p: Vec2| -> bool {
            if flying {
                circle_hits_wall(map, p, r)
            } else {
                circle_hits_wall_or_hazard(map, p, r)
            }
        };
        let p = self.pos + vec2(d.x, 0.0);
        if !blocked(p) {
            self.pos.x = p.x;
        }
        let p = self.pos + vec2(0.0, d.y);
        if !blocked(p) {
            self.pos.y = p.y;
        }
        let (w, h) = (map.width as f32 * TILE, map.height as f32 * TILE);
        self.pos.x = self.pos.x.clamp(r + 2.0, w - r - 2.0);
        self.pos.y = self.pos.y.clamp(r + 2.0, h - r - 2.0);
    }

    pub fn anim_frame(&self) -> usize {
        self.anim as usize
    }

    pub fn shielded(&self, pillars_lit: bool) -> bool {
        self.kind == EnemyKind::Warden && !pillars_lit
    }
}
