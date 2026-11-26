//! Kaelen, the dungeon hunter: movement, dagger, and the four skills.
//! Numbers are from the person's spec; HP and damage are counted in half-hearts.

use macroquad::prelude::*;

use crate::level::{circle_hits_wall, MapData, TILE};
use crate::pixels::Facing;

pub const WALK_SPEED: f32 = 180.0;
pub const DASH_SPEED: f32 = 480.0;
pub const DASH_TIME: f32 = 0.18;
pub const DASH_COOLDOWN: f32 = 1.5;
pub const FLARE_COOLDOWN: f32 = 8.0;
pub const SHIELD_COOLDOWN: f32 = 12.0;
pub const BLINK_COOLDOWN: f32 = 10.0;
pub const FLARE_RADIUS: f32 = 90.0;
pub const FLARE_FREEZE: f32 = 2.0;
pub const BLINK_DISTANCE: f32 = 128.0;
pub const ATTACK_TIME: f32 = 0.25;
pub const ATTACK_ACTIVE_TIME: f32 = 0.19;
pub const ATTACK_COOLDOWN: f32 = 0.27;
pub const ATTACK_REACH: f32 = 36.0;
pub const INPUT_BUFFER: f32 = 0.14;
pub const INVUL_TIME: f32 = 1.0;
/// Solid wall collision radius around Kaelen's lower body.
pub const RADIUS: f32 = 13.0;
/// Vertical offset from `pos` (centre of Kaelen's 64x64 sprite, body y=8..55) to the 64x64 frame bottom.
pub const SPRITE_FEET_OFFSET: f32 = 32.0;
pub const HURTBOX_W: f32 = 20.0;
pub const HURTBOX_H: f32 = 42.0;
pub const FEET_W: f32 = 18.0;
pub const FEET_H: f32 = 12.0;

/// Keys the player can press, filled by the keyboard or by the scripted self-test.
#[derive(Clone, Copy)]
pub struct Keys {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub up_press: bool,
    pub down_press: bool,
    pub left_press: bool,
    pub right_press: bool,
    pub four_way: bool,
    pub attack: bool,
    pub interact: bool,
    pub dash: bool,
    pub flare: bool,
    pub shield: bool,
    pub blink: bool,
}

impl Default for Keys {
    fn default() -> Self {
        Keys {
            up: false,
            down: false,
            left: false,
            right: false,
            up_press: false,
            down_press: false,
            left_press: false,
            right_press: false,
            four_way: false,
            attack: false,
            interact: false,
            dash: false,
            flare: false,
            shield: false,
            blink: false,
        }
    }
}

impl Keys {
    pub fn dir(&self) -> Vec2 {
        let mut d = vec2(
            if self.right { 1.0 } else { 0.0 } - if self.left { 1.0 } else { 0.0 },
            if self.down { 1.0 } else { 0.0 } - if self.up { 1.0 } else { 0.0 },
        );
        if d.length_squared() > 0.0 {
            d = d.normalize();
        }
        d
    }
}

/// Chronological stack of currently held directions (top = most recently pressed key).
/// Ensures pressing a new WASD key immediately turns Kaelen with zero lag or opposite-key cancel.
#[derive(Clone, Copy, Default)]
pub struct DirStack {
    stack: [Option<Facing>; 4],
    len: usize,
    prev_held: [bool; 4],
}

impl DirStack {
    fn idx(f: Facing) -> usize {
        match f {
            Facing::Up => 0,
            Facing::Down => 1,
            Facing::Left => 2,
            Facing::Right => 3,
        }
    }

    fn remove(&mut self, f: Facing) {
        if let Some(pos) = self.stack[..self.len].iter().position(|x| *x == Some(f)) {
            for i in pos..self.len - 1 {
                self.stack[i] = self.stack[i + 1];
            }
            self.stack[self.len - 1] = None;
            self.len -= 1;
        }
    }

    fn push_top(&mut self, f: Facing) {
        self.remove(f);
        if self.len < 4 {
            self.stack[self.len] = Some(f);
            self.len += 1;
        }
    }

    pub fn update(&mut self, keys: &Keys) {
        let dirs = [
            (Facing::Up, keys.up, keys.up_press),
            (Facing::Down, keys.down, keys.down_press),
            (Facing::Left, keys.left, keys.left_press),
            (Facing::Right, keys.right, keys.right_press),
        ];
        // First prune released keys
        for &(f, held, _) in &dirs {
            if !held {
                self.remove(f);
            }
        }
        // Push newly pressed keys onto the top of the stack (strictly gated on `!prev_held`
        // so delayed OS repeat/keydown messages can NEVER re-promote an older held key).
        for &(f, held, press) in &dirs {
            let i = Self::idx(f);
            let active = held || press;
            if active && !self.prev_held[i] {
                self.push_top(f);
            } else if held && !self.stack[..self.len].contains(&Some(f)) {
                self.push_top(f);
            }
            self.prev_held[i] = held;
        }
    }

    /// Most recently pressed active direction.
    pub fn primary(&self) -> Option<Facing> {
        if self.len > 0 {
            self.stack[self.len - 1]
        } else {
            None
        }
    }

    /// Most recent perpendicular held direction underneath `primary` (for corridor pre-turning).
    pub fn secondary_perpendicular(&self, primary: Facing) -> Option<Facing> {
        let horiz = matches!(primary, Facing::Left | Facing::Right);
        for i in (0..self.len.saturating_sub(1)).rev() {
            if let Some(f) = self.stack[i] {
                let f_horiz = matches!(f, Facing::Left | Facing::Right);
                if f_horiz != horiz {
                    return Some(f);
                }
            }
        }
        None
    }

    /// SOCD-resolved 8-way vector (if opposite keys are held, the newest key wins instead of 0).
    pub fn eight_way_dir(&self) -> Vec2 {
        let mut x = 0.0f32;
        let mut y = 0.0f32;
        let mut got_x = false;
        let mut got_y = false;
        for i in (0..self.len).rev() {
            if let Some(f) = self.stack[i] {
                match f {
                    Facing::Left if !got_x => {
                        x = -1.0;
                        got_x = true;
                    }
                    Facing::Right if !got_x => {
                        x = 1.0;
                        got_x = true;
                    }
                    Facing::Up if !got_y => {
                        y = -1.0;
                        got_y = true;
                    }
                    Facing::Down if !got_y => {
                        y = 1.0;
                        got_y = true;
                    }
                    _ => {}
                }
            }
        }
        let d = vec2(x, y);
        if d.length_squared() > 0.0 { d.normalize() } else { Vec2::ZERO }
    }
}

#[allow(dead_code)]
pub struct Player {
    pub pos: Vec2,
    pub facing: Facing,
    pub hp: i32,
    pub max_hp: i32,
    pub moving: bool,
    pub stride_alt: bool,
    pub anim: f32,
    pub idle_anim: f32,
    pub attack_t: f32,
    pub attack_cd: f32,
    pub swing_id: u32,
    pub invul: f32,
    pub knock: Vec2,
    pub dash_t: f32,
    pub dash_cd: f32,
    pub dash_dir: Vec2,
    pub flare_cd: f32,
    pub shield_cd: f32,
    pub shield_on: bool,
    pub blink_cd: f32,
    pub skills: [bool; 4],
    pub walked: f32,
    pub trail: Vec<(Vec2, f32)>,
    pub safe_tile: (i32, i32),
    pub dir_stack: DirStack,
    attack_buf: f32,
    dash_buf: f32,
    flare_buf: f32,
    shield_buf: f32,
    blink_buf: f32,
}

pub struct SkillResult {
    pub attacked: bool,
    pub dashed: bool,
    pub flared: bool,
    pub shielded: bool,
    pub blinked: bool,
}

impl Player {
    pub fn new(pos: Vec2, max_hp: i32, skills: [bool; 4]) -> Player {
        let safe = ((pos.x / TILE) as i32, (pos.y / TILE) as i32);
        Player {
            pos,
            facing: Facing::Down,
            hp: max_hp,
            max_hp,
            moving: false,
            stride_alt: false,
            anim: 0.0,
            idle_anim: 0.0,
            attack_t: 0.0,
            attack_cd: 0.0,
            swing_id: 0,
            invul: 0.0,
            knock: Vec2::ZERO,
            dash_t: 0.0,
            dash_cd: 0.0,
            dash_dir: Vec2::ZERO,
            flare_cd: 0.0,
            shield_cd: 0.0,
            shield_on: false,
            blink_cd: 0.0,
            skills,
            walked: 0.0,
            trail: Vec::new(),
            safe_tile: safe,
            dir_stack: DirStack::default(),
            attack_buf: 0.0,
            dash_buf: 0.0,
            flare_buf: 0.0,
            shield_buf: 0.0,
            blink_buf: 0.0,
        }
    }

    pub fn radius(&self) -> f32 {
        RADIUS
    }

    pub fn collider_center(&self) -> Vec2 {
        vec2(self.pos.x, self.pos.y + 8.0)
    }

    pub fn sprite_feet_y(&self) -> f32 {
        self.pos.y + SPRITE_FEET_OFFSET
    }

    /// Slim vertical hurtbox fitted to Kaelen's pixel silhouette (`y = 8..55` inside 64x64 frame).
    pub fn hurtbox(&self) -> Rect {
        Rect::new(self.pos.x - HURTBOX_W * 0.5, self.pos.y - 20.0, HURTBOX_W, HURTBOX_H)
    }

    /// Boot-level hitbox (`y = 43..55` right above the built-in ground shadow) for floor traps.
    pub fn feet_box(&self) -> Rect {
        Rect::new(self.pos.x - FEET_W * 0.5, self.pos.y + 11.0, FEET_W, FEET_H)
    }

    pub fn feet_pos(&self) -> Vec2 {
        vec2(self.pos.x, self.pos.y + 17.0)
    }

    pub fn in_dash(&self) -> bool {
        self.dash_t > 0.0
    }

    /// Move with wall sliding and corner-assist nudging so narrow corridors feel smooth.
    #[allow(dead_code)]
    pub fn move_by(&mut self, map: &MapData, dx: f32, dy: f32) {
        self.move_by_obstacles(map, &[], dx, dy);
    }

    pub fn move_by_obstacles(&mut self, map: &MapData, obstacles: &[Rect], dx: f32, dy: f32) {
        let before = self.pos;
        let blocked = |cand: Vec2| -> bool {
            let cc = vec2(cand.x, cand.y + 8.0);
            circle_hits_wall(map, cc, RADIUS) || obstacles.iter().any(|r| circle_hits_rect(cc, RADIUS, *r))
        };

        let mut moved_x = false;
        let mut moved_y = false;
        if dx != 0.0 {
            let p = self.pos + vec2(dx, 0.0);
            if !blocked(p) {
                self.pos.x = p.x;
                moved_x = true;
            }
        }
        if dy != 0.0 {
            let p = self.pos + vec2(0.0, dy);
            if !blocked(p) {
                self.pos.y = p.y;
                moved_y = true;
            }
        }

        // Wide corner-assist (up to 26 px) whenever a requested axis was blocked by a corner
        if dx != 0.0 && !moved_x && dy.abs() < 0.0001 {
            let probe_x = dx.signum() * 3.0;
            for nudge in [4.0f32, 8.0, 12.0, 16.0, 20.0, 24.0, 26.0] {
                if !blocked(self.pos + vec2(probe_x, -nudge)) {
                    let slide = dx.abs().min(nudge);
                    let np = self.pos + vec2(0.0, -slide);
                    if !blocked(np) {
                        self.pos.y = np.y;
                    }
                    break;
                } else if !blocked(self.pos + vec2(probe_x, nudge)) {
                    let slide = dx.abs().min(nudge);
                    let np = self.pos + vec2(0.0, slide);
                    if !blocked(np) {
                        self.pos.y = np.y;
                    }
                    break;
                }
            }
        }
        if dy != 0.0 && !moved_y && dx.abs() < 0.0001 {
            let probe_y = dy.signum() * 3.0;
            for nudge in [4.0f32, 8.0, 12.0, 16.0, 20.0, 24.0, 26.0] {
                if !blocked(self.pos + vec2(-nudge, probe_y)) {
                    let slide = dy.abs().min(nudge);
                    let np = self.pos + vec2(-slide, 0.0);
                    if !blocked(np) {
                        self.pos.x = np.x;
                    }
                    break;
                } else if !blocked(self.pos + vec2(nudge, probe_y)) {
                    let slide = dy.abs().min(nudge);
                    let np = self.pos + vec2(slide, 0.0);
                    if !blocked(np) {
                        self.pos.x = np.x;
                    }
                    break;
                }
            }
        }

        let (w, h) = (map.width as f32 * TILE, map.height as f32 * TILE);
        self.pos.x = self.pos.x.clamp(RADIUS + 2.0, w - RADIUS - 2.0);
        self.pos.y = self.pos.y.clamp(20.0, h - 20.0);
        self.walked += (self.pos - before).length();
    }

    /// Move along `primary` (the newest pressed direction); if a solid wall blocks `primary`
    /// beyond corner-assist range and an older perpendicular key `secondary` is still held,
    /// glide along `secondary` until the `primary` doorway opens.
    fn move_four_way(
        &mut self,
        map: &MapData,
        obstacles: &[Rect],
        primary: Facing,
        secondary: Option<Facing>,
        step: f32,
    ) {
        let pd = primary.dir();
        let before = self.pos;
        self.move_by_obstacles(map, obstacles, pd.x * step, pd.y * step);
        if (self.pos - before).length_squared() < 0.0001 {
            if let Some(sec) = secondary {
                let sd = sec.dir();
                self.move_by_obstacles(map, obstacles, sd.x * step, sd.y * step);
            }
        }
    }

    pub fn update(&mut self, map: &MapData, obstacles: &[Rect], keys: &Keys, dt: f32) -> SkillResult {
        self.attack_cd = (self.attack_cd - dt).max(0.0);
        self.invul = (self.invul - dt).max(0.0);
        self.dash_cd = (self.dash_cd - dt).max(0.0);
        self.flare_cd = (self.flare_cd - dt).max(0.0);
        self.blink_cd = (self.blink_cd - dt).max(0.0);
        self.attack_buf = (self.attack_buf - dt).max(0.0);
        self.dash_buf = (self.dash_buf - dt).max(0.0);
        self.flare_buf = (self.flare_buf - dt).max(0.0);
        self.shield_buf = (self.shield_buf - dt).max(0.0);
        self.blink_buf = (self.blink_buf - dt).max(0.0);

        if keys.attack {
            self.attack_buf = INPUT_BUFFER;
        }
        if keys.dash {
            self.dash_buf = INPUT_BUFFER;
        }
        if keys.flare {
            self.flare_buf = INPUT_BUFFER;
        }
        if keys.shield {
            self.shield_buf = INPUT_BUFFER;
        }
        if keys.blink {
            self.blink_buf = INPUT_BUFFER;
        }

        if self.shield_on {
            self.shield_cd = (self.shield_cd - dt).max(0.0);
            if self.shield_cd <= 0.0 {
                self.shield_on = false;
            }
        }

        let was_moving = self.moving;
        let old_facing = self.facing;

        self.dir_stack.update(keys);
        let primary = self.dir_stack.primary();
        let active_vec = if keys.four_way {
            primary.map(|f| f.dir()).unwrap_or(Vec2::ZERO)
        } else {
            self.dir_stack.eight_way_dir()
        };

        let mut out = SkillResult { attacked: false, dashed: false, flared: false, shielded: false, blinked: false };

        // Instant facing turn: always face the most recently pressed direction key immediately!
        if let Some(p_face) = primary {
            self.facing = p_face;
        }

        // Dash Step (can animation-cancel out of dagger swing for responsive dodging)
        if self.dash_buf > 0.0 && self.skills[0] && self.dash_cd <= 0.0 && self.dash_t <= 0.0 {
            self.dash_buf = 0.0;
            self.dash_dir = if active_vec.length_squared() > 0.0 { active_vec } else { self.facing.dir() };
            self.dash_t = DASH_TIME;
            self.dash_cd = DASH_COOLDOWN;
            self.attack_t = 0.0;
            out.dashed = true;
        }

        if self.dash_t > 0.0 {
            self.dash_t -= dt;
            self.trail.push((self.pos, 0.22));
            let step = DASH_SPEED * dt;
            self.move_by_obstacles(map, obstacles, self.dash_dir.x * step, self.dash_dir.y * step);
            self.moving = true;
            self.anim += dt * 12.0;
        } else if let Some(p_dir) = primary {
            let step = WALK_SPEED * dt;
            let sec = self.dir_stack.secondary_perpendicular(p_dir);
            if keys.four_way {
                self.move_four_way(map, obstacles, p_dir, sec, step);
            } else if let Some(sec_dir) = sec {
                // Responsive 8-way with Priority Doorway Catch & Full-Speed Wall Slide:
                // 1) Check if `primary` (newest pressed key) is blocked; if a doorway is within
                //    corner-assist range, steer directly into the `primary` doorway at full speed!
                // 2) If `primary` is blocked by a flat wall, slide along `secondary` at 100% WALK_SPEED
                //    (instead of slowing to 70.7% diagonal speed).
                // 3) In open space, move smoothly along the normalized 8-way diagonal!
                let pd = p_dir.dir();
                let sd = sec_dir.dir();
                let blocked_primary = {
                    let cand = self.pos + pd * step;
                    let cc = vec2(cand.x, cand.y + 8.0);
                    circle_hits_wall(map, cc, RADIUS) || obstacles.iter().any(|r| circle_hits_rect(cc, RADIUS, *r))
                };
                if blocked_primary {
                    let before = self.pos;
                    // Try priority corner-correction into `primary` doorway first
                    self.move_by_obstacles(map, obstacles, pd.x * step, pd.y * step);
                    if (self.pos - before).length_squared() < 0.0001 {
                        // Flat wall along `primary`: glide at full 100% speed along `secondary`
                        self.move_by_obstacles(map, obstacles, sd.x * step, sd.y * step);
                    }
                } else {
                    let blocked_sec = {
                        let cand = self.pos + sd * step;
                        let cc = vec2(cand.x, cand.y + 8.0);
                        circle_hits_wall(map, cc, RADIUS) || obstacles.iter().any(|r| circle_hits_rect(cc, RADIUS, *r))
                    };
                    if blocked_sec {
                        // Secondary is against a wall: move at full 100% speed along `primary`!
                        self.move_by_obstacles(map, obstacles, pd.x * step, pd.y * step);
                    } else {
                        let d = active_vec;
                        self.move_by_obstacles(map, obstacles, d.x * step, d.y * step);
                    }
                }
            } else {
                let pd = p_dir.dir();
                self.move_by_obstacles(map, obstacles, pd.x * step, pd.y * step);
            }

            // First-Frame Walk Animation Fix: jump straight to an active stride frame (1 or 3)
            // on the exact frame movement starts or changes direction (eliminates 125ms neutral-frame lag).
            if !was_moving || self.facing != old_facing {
                self.stride_alt = !self.stride_alt;
                self.anim = if self.stride_alt { 1.0 } else { 3.0 };
            } else {
                self.anim += dt * 9.5;
            }
            self.moving = true;
        } else {
            self.moving = false;
            self.anim = 0.0;
            self.idle_anim += dt * 6.0;
        }

        if self.knock.length_squared() > 1.0 {
            let k = self.knock * dt;
            self.move_by_obstacles(map, obstacles, k.x, k.y);
            self.knock *= (1.0 - dt * 9.0).max(0.0);
        }
        self.trail.retain_mut(|(_, life)| {
            *life -= dt;
            *life > 0.0
        });

        self.attack_t = (self.attack_t - dt).max(0.0);
        if self.attack_buf > 0.0 && self.attack_cd <= 0.0 && self.dash_t <= 0.0 {
            self.attack_buf = 0.0;
            self.attack_t = ATTACK_TIME;
            self.attack_cd = ATTACK_COOLDOWN;
            self.swing_id = self.swing_id.wrapping_add(1);
            if self.swing_id == 0 {
                self.swing_id = 1;
            }
            out.attacked = true;
        }

        // Flare Pulse
        if self.flare_buf > 0.0 && self.skills[1] && self.flare_cd <= 0.0 {
            self.flare_buf = 0.0;
            self.flare_cd = FLARE_COOLDOWN;
            out.flared = true;
        }
        // Stone Shield
        if self.shield_buf > 0.0 && self.skills[2] && self.shield_cd <= 0.0 && !self.shield_on {
            self.shield_buf = 0.0;
            self.shield_on = true;
            self.shield_cd = SHIELD_COOLDOWN;
            out.shielded = true;
        }
        // Phase Blink
        if self.blink_buf > 0.0 && self.skills[3] && self.blink_cd <= 0.0 {
            self.blink_buf = 0.0;
            self.blink_cd = BLINK_COOLDOWN;
            let d = if active_vec.length_squared() > 0.0 { active_vec } else { self.facing.dir() };
            let mut travelled = 0.0;
            let step = 4.0;
            while travelled < BLINK_DISTANCE {
                let p = self.pos + d * step;
                let cc = vec2(p.x, p.y + 8.0);
                if circle_hits_wall(map, cc, RADIUS) || obstacles.iter().any(|r| circle_hits_rect(cc, RADIUS, *r)) {
                    break;
                }
                self.pos = p;
                travelled += step;
            }
            out.blinked = true;
        }

        out
    }

    /// The blade arc hitbox right in front of Kaelen's body/hands.
    pub fn attack_box(&self) -> Rect {
        match self.facing {
            Facing::Right => Rect::new(self.pos.x + 2.0, self.pos.y - 15.0, ATTACK_REACH, 32.0),
            Facing::Left => Rect::new(self.pos.x - 2.0 - ATTACK_REACH, self.pos.y - 15.0, ATTACK_REACH, 32.0),
            Facing::Down => Rect::new(self.pos.x - 16.0, self.pos.y + 1.0, 32.0, ATTACK_REACH),
            Facing::Up => Rect::new(self.pos.x - 16.0, self.pos.y - 1.0 - ATTACK_REACH, 32.0, ATTACK_REACH),
        }
    }

    /// True during the active slash portion of the dagger animation.
    pub fn attack_active(&self) -> bool {
        self.attack_t > ATTACK_TIME - ATTACK_ACTIVE_TIME
    }

    #[allow(dead_code)]
    pub fn attacking(&self) -> bool {
        self.attack_t > 0.0
    }

    /// Take damage in half-hearts. Returns true if it landed.
    pub fn hurt(&mut self, amount: i32, from: Vec2) -> bool {
        if self.invul > 0.0 || self.dash_t > 0.0 || self.hp <= 0 {
            return false;
        }
        if self.shield_on {
            self.shield_on = false;
            self.shield_cd = SHIELD_COOLDOWN;
            self.invul = INVUL_TIME;
            return false;
        }
        self.hp = (self.hp - amount).max(0);
        self.invul = INVUL_TIME;
        let away = (self.pos - from).normalize_or_zero();
        self.knock = away * 220.0;
        true
    }

    pub fn frame_index(&self) -> usize {
        if self.dash_t > 0.0 {
            let p = 1.0 - (self.dash_t / DASH_TIME).clamp(0.0, 1.0);
            ((p * 2.0) as usize).min(1)
        } else if self.moving {
            (self.anim as usize) % 4
        } else {
            0
        }
    }

    pub fn idle_frame(&self) -> usize {
        (self.idle_anim as usize) % 4
    }

    pub fn attack_frame(&self) -> Option<usize> {
        if self.attack_t <= 0.0 {
            return None;
        }
        let p = 1.0 - self.attack_t / ATTACK_TIME;
        Some(((p * 4.0) as usize).min(3))
    }

    #[allow(dead_code)]
    pub fn facing_index(&self) -> usize {
        match self.facing {
            Facing::Down => 0,
            Facing::Up => 1,
            Facing::Left => 2,
            Facing::Right => 3,
        }
    }
}

fn circle_hits_rect(p: Vec2, r: f32, rect: Rect) -> bool {
    let cx = p.x.clamp(rect.x, rect.x + rect.w);
    let cy = p.y.clamp(rect.y, rect.y + rect.h);
    (p - vec2(cx, cy)).length_squared() < r * r
}
