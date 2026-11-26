//! Sprite textures and frame-swapping sheets loaded from `dungeon_hunter_assets/` with `FilterMode::Nearest`.

use macroquad::prelude::*;
use std::path::{Path, PathBuf};

pub type Rgba = (u8, u8, u8);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Facing {
    Down,
    Up,
    Left,
    Right,
}

impl Facing {
    pub fn dir(self) -> Vec2 {
        match self {
            Facing::Down => vec2(0.0, 1.0),
            Facing::Up => vec2(0.0, -1.0),
            Facing::Left => vec2(-1.0, 0.0),
            Facing::Right => vec2(1.0, 0.0),
        }
    }
}

/// Frame-swapping sprite strip loaded from individual PNG frames (or a single PNG texture).
pub struct SpriteSheet {
    pub frames: Vec<Texture2D>,
    pub fw: f32,
    pub fh: f32,
    pub count: usize,
}

fn fallback_tex(w: u16, h: u16) -> Texture2D {
    let bytes = vec![0u8; w as usize * h as usize * 4];
    let tex = Texture2D::from_rgba8(w, h, &bytes);
    tex.set_filter(FilterMode::Nearest);
    tex
}

fn load_tex(path: &Path) -> Option<(Texture2D, f32, f32)> {
    let bytes = std::fs::read(path).ok()?;
    let img = Image::from_file_with_format(&bytes, None).ok()?;
    let (w, h) = (img.width as f32, img.height as f32);
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Nearest);
    Some((tex, w, h))
}

impl SpriteSheet {
    pub fn from_png_files(paths: &[PathBuf]) -> SpriteSheet {
        let mut frames = Vec::with_capacity(paths.len().max(1));
        let mut fw = 64.0;
        let mut fh = 64.0;
        for p in paths {
            if let Some((tex, w, h)) = load_tex(p) {
                fw = w;
                fh = h;
                frames.push(tex);
            } else {
                eprintln!("dungeon_hunter: warning: failed to load asset {:?}", p);
            }
        }
        if frames.is_empty() {
            frames.push(fallback_tex(64, 64));
        }
        let count = frames.len();
        SpriteSheet { frames, fw, fh, count }
    }

    pub fn from_single_png(path: &Path) -> SpriteSheet {
        Self::from_png_files(&[path.to_path_buf()])
    }

    /// Draw one frame with its bottom-centre anchored at `(x, y)`.
    pub fn draw(&self, frame: usize, x: f32, y: f32, scale: f32) {
        self.draw_ex(frame, x, y, scale, false);
    }

    /// Draw one frame with its bottom-centre anchored at `(x, y)`, optionally mirrored horizontally.
    pub fn draw_ex(&self, frame: usize, x: f32, y: f32, scale: f32, flip_x: bool) {
        let f = if self.count == 0 { 0 } else { frame % self.count };
        draw_texture_ex(
            &self.frames[f],
            x - self.fw * scale * 0.5,
            y - self.fh * scale,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(self.fw * scale, self.fh * scale)),
                flip_x,
                ..Default::default()
            },
        );
    }

    /// Draw one frame centred on `(x, y)`.
    pub fn draw_centered(&self, frame: usize, x: f32, y: f32, scale: f32) {
        self.draw_centered_ex(frame, x, y, scale, false);
    }

    pub fn draw_centered_ex(&self, frame: usize, x: f32, y: f32, scale: f32, flip_x: bool) {
        let f = if self.count == 0 { 0 } else { frame % self.count };
        draw_texture_ex(
            &self.frames[f],
            x - self.fw * scale * 0.5,
            y - self.fh * scale * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(self.fw * scale, self.fh * scale)),
                flip_x,
                ..Default::default()
            },
        );
    }

    /// Draw one frame scaled into an exact rectangle `r`.
    pub fn draw_in(&self, frame: usize, r: Rect) {
        let f = if self.count == 0 { 0 } else { frame % self.count };
        draw_texture_ex(
            &self.frames[f],
            r.x,
            r.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(r.w, r.h)),
                ..Default::default()
            },
        );
    }

    /// Draw a background image covering `(sw, sh)` while preserving aspect ratio.
    pub fn draw_cover(&self, frame: usize, sw: f32, sh: f32) {
        let f = if self.count == 0 { 0 } else { frame % self.count };
        let scale = (sw / self.fw).max(sh / self.fh);
        let dw = self.fw * scale;
        let dh = self.fh * scale;
        draw_texture_ex(
            &self.frames[f],
            (sw - dw) * 0.5,
            (sh - dh) * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dw, dh)),
                ..Default::default()
            },
        );
    }
}
