//! Inline audio playback (replaces the WebView `<audio>` element).
//!
//! One player thread owns the output device; only one clip plays at a time
//! (starting another stops the first, like the Svelte UI). Voice notes are
//! Ogg/Opus, which symphonia can't decode, so those go through libopus.

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::num::NonZero;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::buffer::SamplesBuffer;
use rodio::{Decoder, DeviceSinkBuilder, Player, Source};

#[derive(Debug, Clone, Default)]
pub struct PlayState {
    /// What is loaded (a message id, or "preview").
    pub key: Option<String>,
    pub playing: bool,
    pub position: Duration,
    pub duration: Option<Duration>,
}

enum Cmd {
    Play { key: String, path: PathBuf },
    Toggle,
    Stop,
}

pub struct AudioPlayer {
    tx: Sender<Cmd>,
    pub state: Arc<Mutex<PlayState>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Cmd>();
        let state = Arc::new(Mutex::new(PlayState::default()));
        let st = state.clone();
        std::thread::Builder::new()
            .name("audio".into())
            .spawn(move || {
                // Opened lazily so an app that never plays audio never holds
                // the output device.
                let mut sink = None;
                let mut player: Option<Player> = None;
                loop {
                    let cmd = rx.recv_timeout(Duration::from_millis(100));
                    match cmd {
                        Ok(Cmd::Play { key, path }) => {
                            if let Some(p) = player.take() {
                                p.stop();
                            }
                            if sink.is_none() {
                                match DeviceSinkBuilder::open_default_sink() {
                                    Ok(mut s) => {
                                        s.log_on_drop(false);
                                        sink = Some(s);
                                    }
                                    Err(e) => {
                                        log::warn!("no audio output: {e}");
                                        continue;
                                    }
                                }
                            }
                            let mixer = sink.as_ref().unwrap().mixer();
                            match load(&path) {
                                Ok((source, duration)) => {
                                    let p = Player::connect_new(mixer);
                                    p.append(source);
                                    p.play();
                                    player = Some(p);
                                    *lock(&st) = PlayState {
                                        key: Some(key),
                                        playing: true,
                                        position: Duration::ZERO,
                                        duration,
                                    };
                                }
                                Err(e) => {
                                    log::warn!("can't play {}: {e}", path.display());
                                    *lock(&st) = PlayState::default();
                                }
                            }
                        }
                        Ok(Cmd::Toggle) => {
                            if let Some(p) = &player {
                                let mut s = lock(&st);
                                if p.is_paused() {
                                    p.play();
                                    s.playing = true;
                                } else {
                                    p.pause();
                                    s.playing = false;
                                }
                            }
                        }
                        Ok(Cmd::Stop) => {
                            if let Some(p) = player.take() {
                                p.stop();
                            }
                            *lock(&st) = PlayState::default();
                        }
                        Err(RecvTimeoutError::Timeout) => {}
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                    if let Some(p) = &player {
                        if p.empty() {
                            player = None;
                            *lock(&st) = PlayState::default();
                        } else {
                            lock(&st).position = p.get_pos();
                        }
                    }
                }
            })
            .expect("spawn audio thread");
        Self { tx, state }
    }

    /// Play `path` under `key`, or pause/resume when `key` is already loaded.
    pub fn toggle(&self, key: &str, path: &Path) {
        let loaded = lock(&self.state).key.as_deref() == Some(key);
        let _ = if loaded {
            self.tx.send(Cmd::Toggle)
        } else {
            self.tx.send(Cmd::Play {
                key: key.to_string(),
                path: path.to_path_buf(),
            })
        };
    }

    pub fn stop(&self) {
        let _ = self.tx.send(Cmd::Stop);
    }

    pub fn snapshot(&self) -> PlayState {
        lock(&self.state).clone()
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

type Loaded = (Box<dyn Source + Send>, Option<Duration>);

fn load(path: &Path) -> anyhow::Result<Loaded> {
    if is_ogg_opus(path) {
        let (channels, samples) = decode_ogg_opus(path)?;
        let frames = samples.len() / channels.max(1) as usize;
        let duration = Duration::from_secs_f64(frames as f64 / 48_000.0);
        let ch = NonZero::new(channels).unwrap_or(NonZero::new(1).unwrap());
        let buf = SamplesBuffer::new(ch, NonZero::new(48_000).unwrap(), samples);
        return Ok((Box::new(buf), Some(duration)));
    }
    let decoder = Decoder::new(BufReader::new(File::open(path)?))?;
    let duration = decoder.total_duration();
    Ok((Box::new(decoder), duration))
}

/// Ogg container whose first packet is an `OpusHead`.
fn is_ogg_opus(path: &Path) -> bool {
    let Ok(mut f) = File::open(path) else {
        return false;
    };
    let mut head = [0u8; 64];
    let n = f.read(&mut head).unwrap_or(0);
    let _ = f.seek(SeekFrom::Start(0));
    head[..n].starts_with(b"OggS") && head[..n].windows(8).any(|w| w == b"OpusHead")
}

/// Decode an Ogg/Opus file to interleaved f32 at 48 kHz.
pub fn decode_ogg_opus(path: &Path) -> anyhow::Result<(u16, Vec<f32>)> {
    let mut reader = ogg::PacketReader::new(BufReader::new(File::open(path)?));
    let head = reader
        .read_packet()?
        .ok_or_else(|| anyhow::anyhow!("empty ogg"))?;
    anyhow::ensure!(
        head.data.starts_with(b"OpusHead") && head.data.len() >= 19,
        "not an Opus stream"
    );
    let channels = head.data[9].clamp(1, 2) as u16;
    let pre_skip = u16::from_le_bytes([head.data[10], head.data[11]]) as usize;
    let mut decoder = opus::Decoder::new(
        48_000,
        if channels == 2 {
            opus::Channels::Stereo
        } else {
            opus::Channels::Mono
        },
    )?;
    let mut out = Vec::new();
    let mut frame = vec![0f32; 5760 * channels as usize];
    while let Some(packet) = reader.read_packet()? {
        if packet.data.starts_with(b"OpusTags") {
            continue;
        }
        let n = decoder.decode_float(&packet.data, &mut frame, false)?;
        out.extend_from_slice(&frame[..n * channels as usize]);
    }
    let skip = (pre_skip * channels as usize).min(out.len());
    out.drain(..skip);
    Ok((channels, out))
}
