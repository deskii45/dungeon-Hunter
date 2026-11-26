#![allow(dead_code)]
//! Puzzles and traps calibrated to `dungeon_hunter_assets/sprites/traps_and_puzzles/`.

use macroquad::prelude::*;

pub const SPIKE_DOWN: f32 = 2.0;
pub const SPIKE_WARN: f32 = 0.5;
pub const SPIKE_UP: f32 = 1.2;
pub const CRUMBLE_DELAY: f32 = 0.9;
pub const BLOCK_SPEED: f32 = 70.0;

/// Spike rhythm: down (frame 0) -> warning (frame 1) -> rising/up/retracting (frames 2 & 3).
pub struct Spike {
    pub x: i32,
    pub y: i32,
    pub t: f32,
}

impl Spike {
    pub fn new(x: i32, y: i32, offset: f32) -> Spike {
        Spike { x, y, t: offset }
    }

    pub fn tick(&mut self, dt: f32) {
        self.t += dt;
        let total = SPIKE_DOWN + SPIKE_WARN + SPIKE_UP;
        if self.t >= total {
            self.t -= total;
        }
    }

    /// Up = damaging. Warning = peeking tips, harmless.
    pub fn is_up(&self) -> bool {
        self.t >= SPIKE_DOWN + SPIKE_WARN
    }

    pub fn is_warning(&self) -> bool {
        self.t >= SPIKE_DOWN && self.t < SPIKE_DOWN + SPIKE_WARN
    }

    /// Active damage region matching the 2x2 spike cluster centered inside the 64x64 tile (`spikes_2`/`spikes_3`).
    pub fn damage_rect(&self, tile: f32) -> Option<Rect> {
        if !self.is_up() {
            return None;
        }
        let p = (self.t - SPIKE_DOWN - SPIKE_WARN) / SPIKE_UP;
        if p >= 0.85 {
            return None;
        }
        Some(Rect::new(
            self.x as f32 * tile + 10.0,
            self.y as f32 * tile + 10.0,
            tile - 20.0,
            tile - 20.0,
        ))
    }

    /// Matches `spikes_0..3.png`: 0 = holes, 1 = peeking tips, 2 = half extended, 3 = fully extended peak.
    pub fn frame(&self) -> usize {
        if self.is_up() {
            let p = (self.t - SPIKE_DOWN - SPIKE_WARN) / SPIKE_UP;
            if p < 0.14 || p >= 0.82 {
                2
            } else {
                3
            }
        } else if self.is_warning() {
            1
        } else {
            0
        }
    }
}

/// A floor tile that cracks across 4 frames (`crumble_0..3.png`) when stepped on and falls away.
pub struct Crumble {
    pub x: i32,
    pub y: i32,
    pub falling: Option<f32>,
    pub gone: bool,
}

impl Crumble {
    pub fn new(x: i32, y: i32) -> Crumble {
        Crumble { x, y, falling: None, gone: false }
    }

    pub fn trigger(&mut self) {
        if self.falling.is_none() && !self.gone {
            self.falling = Some(CRUMBLE_DELAY);
        }
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if let Some(t) = self.falling {
            let t = t - dt;
            if t <= 0.0 {
                self.falling = None;
                self.gone = true;
                return true;
            }
            self.falling = Some(t);
        }
        false
    }

    pub fn solid(&self) -> bool {
        !self.gone
    }

    /// Frame index for `crumble_0..3.png`: 0 intact, 1 cracked, 2 breaking, 3 collapsed hole.
    pub fn frame(&self) -> usize {
        if self.gone {
            3
        } else if let Some(t) = self.falling {
            let p = 1.0 - (t / CRUMBLE_DELAY).clamp(0.0, 1.0);
            if p < 0.5 { 1 } else { 2 }
        } else {
            0
        }
    }
}

pub struct PushBlock {
    pub x: i32,
    pub y: i32,
}

pub struct Pillar {
    pub x: i32,
    pub y: i32,
    pub lit: bool,
    pub anim: f32,
}

pub struct Torch {
    pub x: i32,
    pub y: i32,
    pub lit: bool,
}
