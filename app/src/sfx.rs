//! Audio engine loading all 29 16-bit PCM WAV files from `dungeon_hunter_assets/audio/`
//! and mixing BGM + polyphonic SFX with zero external crate dependencies.

use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::assets::find_assets_dir;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BgmKind {
    None,
    Lobby,
    Explore,
    Boss,
    Victory,
    GameOver,
}

#[derive(Clone)]
pub struct Clip {
    samples: Arc<Vec<i16>>,
    rec_vol: f32,
}

fn parse_wav(path: &Path, rec_vol: f32) -> Option<Clip> {
    let data = std::fs::read(path).ok()?;
    if data.len() < 44 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return None;
    }
    let mut pos = 12usize;
    let mut channels = 1u16;
    let mut bits = 16u16;
    let mut pcm_slice: Option<&[u8]> = None;

    while pos + 8 <= data.len() {
        let id = &data[pos..pos + 4];
        let size = u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]) as usize;
        pos += 8;
        if pos + size > data.len() {
            break;
        }
        if id == b"fmt " && size >= 16 {
            channels = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
            bits = u16::from_le_bytes([data[pos + 14], data[pos + 15]]);
        } else if id == b"data" {
            pcm_slice = Some(&data[pos..pos + size]);
            break;
        }
        pos += size + (size & 1);
    }

    let raw = pcm_slice?;
    if bits != 16 {
        return None;
    }
    let mut out = Vec::with_capacity(raw.len() / (2 * channels.max(1) as usize));
    if channels == 1 {
        for chunk in raw.chunks_exact(2) {
            out.push(i16::from_le_bytes([chunk[0], chunk[1]]));
        }
    } else {
        let step = 2 * channels as usize;
        for chunk in raw.chunks_exact(step) {
            let mut sum = 0i32;
            for c in 0..channels as usize {
                sum += i16::from_le_bytes([chunk[c * 2], chunk[c * 2 + 1]]) as i32;
            }
            out.push((sum / channels as i32) as i16);
        }
    }
    Some(Clip { samples: Arc::new(out), rec_vol })
}

struct Voice {
    samples: Arc<Vec<i16>>,
    pos: usize,
    vol: f32,
}

struct BgmState {
    kind: BgmKind,
    samples: Option<Arc<Vec<i16>>>,
    pos: usize,
    looped: bool,
    rec_vol: f32,
}

struct MixerState {
    master_vol: f32,
    muted: bool,
    bgm: BgmState,
    voices: Vec<Voice>,
}

impl Default for MixerState {
    fn default() -> Self {
        MixerState {
            master_vol: 0.7,
            muted: false,
            bgm: BgmState { kind: BgmKind::None, samples: None, pos: 0, looped: false, rec_vol: 0.5 },
            voices: Vec::with_capacity(16),
        }
    }
}

pub struct Sfx {
    mixer: Arc<Mutex<MixerState>>,
    pub bgm_lobby: Option<Clip>,
    pub bgm_explore: Option<Clip>,
    pub bgm_boss: Option<Clip>,
    pub bgm_victory: Option<Clip>,
    pub bgm_gameover: Option<Clip>,

    pub footstep_1: Option<Clip>,
    pub footstep_2: Option<Clip>,
    pub slash: Option<Clip>,
    pub hurt: Option<Clip>,
    pub player_death: Option<Clip>,

    pub skill_dash: Option<Clip>,
    pub skill_flare: Option<Clip>,
    pub skill_shield: Option<Clip>,
    pub skill_blink: Option<Clip>,

    pub enemy_hit: Option<Clip>,
    pub enemy_death: Option<Clip>,
    pub boss_slam: Option<Clip>,
    pub boss_shield_break: Option<Clip>,

    pub pickup: Option<Clip>,
    pub spike_activate: Option<Clip>,
    pub tile_crumble: Option<Clip>,
    pub block_push: Option<Clip>,
    pub pressure_plate: Option<Clip>,
    pub rune_activate: Option<Clip>,
    pub torch_ignite: Option<Clip>,
    pub gate: Option<Clip>,

    pub ui_hover: Option<Clip>,
    pub click: Option<Clip>,
    pub ui_back: Option<Clip>,
}

fn load_clip(audio_root: &Path, rel: &str, rec_vol: f32) -> Option<Clip> {
    let p = audio_root.join(rel);
    let c = parse_wav(&p, rec_vol);
    if c.is_none() {
        eprintln!("dungeon_hunter: warning: failed to load audio {:?}", p);
    }
    c
}

impl Sfx {
    pub fn build(headless: bool) -> Sfx {
        let root = find_assets_dir().join("audio");
        let r = root.as_path();
        let mixer = Arc::new(Mutex::new(MixerState::default()));
        if !headless {
            #[cfg(target_os = "windows")]
            win_audio::spawn_mixer(Arc::clone(&mixer));
        }

        Sfx {
            mixer,
            bgm_lobby: load_clip(r, "bgm/lobby_theme.wav", 0.55),
            bgm_explore: load_clip(r, "bgm/dungeon_explore.wav", 0.50),
            bgm_boss: load_clip(r, "bgm/boss_obsidian_warden.wav", 0.55),
            bgm_victory: load_clip(r, "bgm/victory_fanfare.wav", 0.70),
            bgm_gameover: load_clip(r, "bgm/game_over_jingle.wav", 0.65),

            footstep_1: load_clip(r, "sfx/player/footstep_stone_1.wav", 0.45),
            footstep_2: load_clip(r, "sfx/player/footstep_stone_2.wav", 0.45),
            slash: load_clip(r, "sfx/player/dagger_slash.wav", 0.65),
            hurt: load_clip(r, "sfx/player/player_hurt.wav", 0.80),
            player_death: load_clip(r, "sfx/player/player_death.wav", 0.80),

            skill_dash: load_clip(r, "sfx/skills/skill_dash.wav", 0.70),
            skill_flare: load_clip(r, "sfx/skills/skill_flare_pulse.wav", 0.75),
            skill_shield: load_clip(r, "sfx/skills/skill_stone_shield.wav", 0.75),
            skill_blink: load_clip(r, "sfx/skills/skill_phase_blink.wav", 0.75),

            enemy_hit: load_clip(r, "sfx/enemies/enemy_hit.wav", 0.75),
            enemy_death: load_clip(r, "sfx/enemies/enemy_death.wav", 0.75),
            boss_slam: load_clip(r, "sfx/enemies/boss_slam.wav", 0.85),
            boss_shield_break: load_clip(r, "sfx/enemies/boss_shield_break.wav", 0.85),

            pickup: load_clip(r, "sfx/world/key_pickup.wav", 0.80),
            spike_activate: load_clip(r, "sfx/world/spike_activate.wav", 0.65),
            tile_crumble: load_clip(r, "sfx/world/tile_crumble.wav", 0.70),
            block_push: load_clip(r, "sfx/world/block_push.wav", 0.65),
            pressure_plate: load_clip(r, "sfx/world/pressure_plate.wav", 0.70),
            rune_activate: load_clip(r, "sfx/world/rune_activate.wav", 0.75),
            torch_ignite: load_clip(r, "sfx/world/torch_ignite.wav", 0.70),
            gate: load_clip(r, "sfx/world/gate_open.wav", 0.80),

            ui_hover: load_clip(r, "sfx/ui/ui_hover.wav", 0.50),
            click: load_clip(r, "sfx/ui/ui_click.wav", 0.65),
            ui_back: load_clip(r, "sfx/ui/ui_back.wav", 0.60),
        }
    }

    pub fn sync_settings(&self, volume: f32, mute: bool) {
        if let Ok(mut m) = self.mixer.lock() {
            m.master_vol = volume.clamp(0.0, 1.0);
            m.muted = mute;
        }
    }

    pub fn set_bgm(&self, kind: BgmKind, volume_scale: f32) {
        let (clip, looped): (Option<&Clip>, bool) = match kind {
            BgmKind::None => (None, false),
            BgmKind::Lobby => (self.bgm_lobby.as_ref(), true),
            BgmKind::Explore => (self.bgm_explore.as_ref(), true),
            BgmKind::Boss => (self.bgm_boss.as_ref(), true),
            BgmKind::Victory => (self.bgm_victory.as_ref(), false),
            BgmKind::GameOver => (self.bgm_gameover.as_ref(), false),
        };
        if let Ok(mut m) = self.mixer.lock() {
            if m.bgm.kind != kind {
                m.bgm.kind = kind;
                m.bgm.pos = 0;
                m.bgm.looped = looped;
                if let Some(c) = clip {
                    m.bgm.samples = Some(Arc::clone(&c.samples));
                    m.bgm.rec_vol = c.rec_vol * volume_scale;
                } else {
                    m.bgm.samples = None;
                }
            } else if let Some(c) = clip {
                m.bgm.rec_vol = c.rec_vol * volume_scale;
            }
        }
    }

    pub fn play(&self, clip: &Option<Clip>) {
        let Some(c) = clip else { return };
        if let Ok(mut m) = self.mixer.lock() {
            if m.muted || m.master_vol <= 0.01 {
                return;
            }
            if m.voices.len() >= 16 {
                m.voices.remove(0);
            }
            m.voices.push(Voice {
                samples: Arc::clone(&c.samples),
                pos: 0,
                vol: c.rec_vol,
            });
        }
    }
}

#[cfg(target_os = "windows")]
mod win_audio {
    use super::MixerState;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    const WAVE_MAPPER: u32 = u32::MAX;
    const WHDR_DONE: u32 = 0x0000_0001;
    const CHUNK_SAMPLES: usize = 512;
    const NUM_BUFS: usize = 4;

    #[repr(C)]
    struct WaveFormatEx {
        w_format_tag: u16,
        n_channels: u16,
        n_samples_per_sec: u32,
        n_avg_bytes_per_sec: u32,
        n_block_align: u16,
        w_bits_per_sample: u16,
        cb_size: u16,
    }

    #[repr(C)]
    struct WaveHdr {
        lp_data: *mut u8,
        dw_buffer_length: u32,
        dw_bytes_recorded: u32,
        dw_user: usize,
        dw_flags: u32,
        dw_loops: u32,
        lp_next: usize,
        reserved: usize,
    }

    #[link(name = "winmm")]
    extern "system" {
        fn waveOutOpen(
            phwo: *mut isize,
            u_device_id: u32,
            pwfx: *const WaveFormatEx,
            dw_callback: usize,
            dw_instance: usize,
            fdw_open: u32,
        ) -> u32;
        fn waveOutPrepareHeader(hwo: isize, pwh: *mut WaveHdr, cbwh: u32) -> u32;
        fn waveOutWrite(hwo: isize, pwh: *mut WaveHdr, cbwh: u32) -> u32;
    }

    pub fn spawn_mixer(mixer: Arc<Mutex<MixerState>>) {
        std::thread::Builder::new()
            .name("dh-audio".to_string())
            .spawn(move || unsafe {
                let wfx = WaveFormatEx {
                    w_format_tag: 1,
                    n_channels: 1,
                    n_samples_per_sec: 44100,
                    n_avg_bytes_per_sec: 88200,
                    n_block_align: 2,
                    w_bits_per_sample: 16,
                    cb_size: 0,
                };
                let mut hwo: isize = 0;
                if waveOutOpen(&mut hwo, WAVE_MAPPER, &wfx, 0, 0, 0) != 0 || hwo == 0 {
                    return;
                }

                let mut bufs: Vec<Vec<i16>> = (0..NUM_BUFS).map(|_| vec![0i16; CHUNK_SAMPLES]).collect();
                let mut hdrs: Vec<WaveHdr> = bufs
                    .iter_mut()
                    .map(|b| WaveHdr {
                        lp_data: b.as_mut_ptr() as *mut u8,
                        dw_buffer_length: (CHUNK_SAMPLES * 2) as u32,
                        dw_bytes_recorded: 0,
                        dw_user: 0,
                        dw_flags: 0,
                        dw_loops: 0,
                        lp_next: 0,
                        reserved: 0,
                    })
                    .collect();

                for hdr in hdrs.iter_mut() {
                    if waveOutPrepareHeader(hwo, hdr as *mut WaveHdr, std::mem::size_of::<WaveHdr>() as u32) != 0 {
                        return;
                    }
                    waveOutWrite(hwo, hdr as *mut WaveHdr, std::mem::size_of::<WaveHdr>() as u32);
                }

                loop {
                    let mut submitted = false;
                    for i in 0..NUM_BUFS {
                        if (hdrs[i].dw_flags & WHDR_DONE) != 0 {
                            mix_chunk(&mixer, &mut bufs[i]);
                            waveOutWrite(hwo, &mut hdrs[i] as *mut WaveHdr, std::mem::size_of::<WaveHdr>() as u32);
                            submitted = true;
                        }
                    }
                    if !submitted {
                        std::thread::sleep(Duration::from_millis(4));
                    }
                }
            })
            .ok();
    }

    fn mix_chunk(mixer: &Arc<Mutex<MixerState>>, out: &mut [i16]) {
        let Ok(mut m) = mixer.lock() else {
            out.fill(0);
            return;
        };
        if m.muted || m.master_vol <= 0.01 {
            out.fill(0);
            m.voices.clear();
            return;
        }
        let master = m.master_vol;
        let bgm_vol = m.bgm.rec_vol * master;
        let bgm_samples = m.bgm.samples.clone();
        let bgm_looped = m.bgm.looped;

        for sample in out.iter_mut() {
            let mut acc = 0.0f32;
            if let Some(ref bgm_buf) = bgm_samples {
                if !bgm_buf.is_empty() && m.bgm.pos < bgm_buf.len() {
                    acc += bgm_buf[m.bgm.pos] as f32 * bgm_vol;
                    m.bgm.pos += 1;
                    if m.bgm.pos >= bgm_buf.len() {
                        if bgm_looped {
                            m.bgm.pos = 0;
                        } else {
                            m.bgm.samples = None;
                        }
                    }
                }
            }
            for v in m.voices.iter_mut() {
                if v.pos < v.samples.len() {
                    acc += v.samples[v.pos] as f32 * (v.vol * master);
                    v.pos += 1;
                }
            }
            *sample = acc.clamp(-32767.0, 32767.0) as i16;
        }
        m.voices.retain(|v| v.pos < v.samples.len());
    }
}

#[cfg(test)]
mod tests {
    use super::Sfx;

    #[test]
    fn all_29_audio_wav_files_load_cleanly() {
        let sfx = Sfx::build(true);
        assert!(sfx.bgm_lobby.is_some());
        assert!(sfx.bgm_explore.is_some());
        assert!(sfx.bgm_boss.is_some());
        assert!(sfx.bgm_victory.is_some());
        assert!(sfx.bgm_gameover.is_some());
        assert!(sfx.footstep_1.is_some());
        assert!(sfx.footstep_2.is_some());
        assert!(sfx.slash.is_some());
        assert!(sfx.hurt.is_some());
        assert!(sfx.player_death.is_some());
        assert!(sfx.skill_dash.is_some());
        assert!(sfx.skill_flare.is_some());
        assert!(sfx.skill_shield.is_some());
        assert!(sfx.skill_blink.is_some());
        assert!(sfx.enemy_hit.is_some());
        assert!(sfx.enemy_death.is_some());
        assert!(sfx.boss_slam.is_some());
        assert!(sfx.boss_shield_break.is_some());
        assert!(sfx.pickup.is_some());
        assert!(sfx.spike_activate.is_some());
        assert!(sfx.tile_crumble.is_some());
        assert!(sfx.block_push.is_some());
        assert!(sfx.pressure_plate.is_some());
        assert!(sfx.rune_activate.is_some());
        assert!(sfx.torch_ignite.is_some());
        assert!(sfx.gate.is_some());
        assert!(sfx.ui_hover.is_some());
        assert!(sfx.click.is_some());
        assert!(sfx.ui_back.is_some());
    }
}
