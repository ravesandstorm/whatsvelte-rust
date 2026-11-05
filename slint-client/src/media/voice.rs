//! Voice-note recording (replaces MediaRecorder): microphone → mono 48 kHz →
//! Opus (VoIP) → Ogg. WhatsApp plays push-to-talk notes as Ogg/Opus, so this
//! is sent as-is with `ptt: true`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use rodio::Source;
use rodio::microphone::MicrophoneBuilder;

/// (sample rate, channels, interleaved samples) from the capture thread.
type Captured = anyhow::Result<(u32, u16, Vec<f32>)>;

pub struct VoiceRecorder {
    stop: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<Captured>>,
    pub started: Instant,
    /// Set by the capture thread when the microphone can't be opened.
    pub error: Arc<Mutex<Option<String>>>,
}

impl VoiceRecorder {
    pub fn start() -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let error = Arc::new(Mutex::new(None));
        let (stop2, error2) = (stop.clone(), error.clone());
        let worker = std::thread::Builder::new()
            .name("mic".into())
            .spawn(move || {
                let mic = MicrophoneBuilder::new()
                    .default_device()
                    .and_then(|b| b.default_config())
                    .map_err(|e| anyhow::anyhow!("{e}"))
                    .and_then(|b| b.open_stream().map_err(|e| anyhow::anyhow!("{e}")));
                let mut mic = match mic {
                    Ok(m) => m,
                    Err(e) => {
                        *error2.lock().unwrap_or_else(|p| p.into_inner()) = Some(e.to_string());
                        return Err(e);
                    }
                };
                let rate = mic.sample_rate().get();
                let channels = mic.channels().get();
                let mut samples = Vec::with_capacity(rate as usize * 60);
                while !stop2.load(Ordering::Relaxed) {
                    match mic.next() {
                        Some(s) => samples.push(s),
                        None => break,
                    }
                }
                Ok((rate, channels, samples))
            })
            .expect("spawn mic thread");
        Self {
            stop,
            worker: Some(worker),
            started: Instant::now(),
            error,
        }
    }

    pub fn elapsed_label(&self) -> String {
        let s = self.started.elapsed().as_secs();
        format!("{}:{:02}", s / 60, s % 60)
    }

    /// Stop and encode. Returns (ogg path, duration seconds).
    pub fn finish(mut self) -> anyhow::Result<(PathBuf, u32)> {
        self.stop.store(true, Ordering::Relaxed);
        let (rate, channels, samples) = self
            .worker
            .take()
            .ok_or_else(|| anyhow::anyhow!("not recording"))?
            .join()
            .map_err(|_| anyhow::anyhow!("recorder crashed"))??;
        let mono = to_mono(&samples, channels as usize);
        let pcm = resample(&mono, rate, 48_000);
        let path = crate::media::prep::stage_path("ogg");
        encode_ogg_opus(&pcm, &path)?;
        Ok((path, (pcm.len() as f64 / 48_000.0).round() as u32))
    }

    pub fn cancel(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.worker.take();
    }
}

fn to_mono(samples: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return samples.to_vec();
    }
    samples
        .chunks(channels)
        .map(|f| f.iter().sum::<f32>() / f.len() as f32)
        .collect()
}

/// Linear resampler — plenty for speech.
fn resample(input: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to || input.is_empty() {
        return input.to_vec();
    }
    let ratio = from as f64 / to as f64;
    let out_len = (input.len() as f64 / ratio) as usize;
    (0..out_len)
        .map(|i| {
            let pos = i as f64 * ratio;
            let idx = pos as usize;
            let frac = (pos - idx as f64) as f32;
            let a = input[idx.min(input.len() - 1)];
            let b = input[(idx + 1).min(input.len() - 1)];
            a + (b - a) * frac
        })
        .collect()
}

/// Encode mono 48 kHz PCM as Ogg/Opus (RFC 7845 headers, 20 ms frames).
pub fn encode_ogg_opus(pcm: &[f32], path: &Path) -> anyhow::Result<()> {
    const FRAME: usize = 960; // 20 ms @ 48 kHz
    let mut enc = opus::Encoder::new(48_000, opus::Channels::Mono, opus::Application::Voip)?;
    let pre_skip = enc.get_lookahead().unwrap_or(312) as u16;
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut w = ogg::PacketWriter::new(file);
    let serial = 0x5741_7273; // "WArs"

    let mut head = Vec::with_capacity(19);
    head.extend_from_slice(b"OpusHead");
    head.push(1); // version
    head.push(1); // channels
    head.extend_from_slice(&pre_skip.to_le_bytes());
    head.extend_from_slice(&48_000u32.to_le_bytes());
    head.extend_from_slice(&0i16.to_le_bytes()); // output gain
    head.push(0); // mapping family
    w.write_packet(head, serial, ogg::PacketWriteEndInfo::EndPage, 0)?;

    let vendor = b"whatsvelte-rust";
    let mut tags = Vec::new();
    tags.extend_from_slice(b"OpusTags");
    tags.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
    tags.extend_from_slice(vendor);
    tags.extend_from_slice(&0u32.to_le_bytes());
    w.write_packet(tags, serial, ogg::PacketWriteEndInfo::EndPage, 0)?;

    let mut buf = vec![0u8; 4000];
    let frames: Vec<&[f32]> = pcm.chunks(FRAME).collect();
    let total = frames.len().max(1);
    let mut granule = pre_skip as u64;
    for (i, chunk) in frames.iter().enumerate() {
        let mut frame = [0f32; FRAME];
        frame[..chunk.len()].copy_from_slice(chunk);
        let n = enc.encode_float(&frame, &mut buf)?;
        granule += chunk.len() as u64;
        let end = if i + 1 == total {
            ogg::PacketWriteEndInfo::EndStream
        } else {
            ogg::PacketWriteEndInfo::NormalPacket
        };
        w.write_packet(buf[..n].to_vec(), serial, end, granule)?;
    }
    if frames.is_empty() {
        let n = enc.encode_float(&[0f32; FRAME], &mut buf)?;
        w.write_packet(
            buf[..n].to_vec(),
            serial,
            ogg::PacketWriteEndInfo::EndStream,
            pre_skip as u64 + FRAME as u64,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opus_roundtrip() {
        let pcm: Vec<f32> = (0..48_000)
            .map(|i| (i as f32 * 440.0 * std::f32::consts::TAU / 48_000.0).sin() * 0.3)
            .collect();
        let path = crate::media::prep::stage_path("ogg");
        encode_ogg_opus(&pcm, &path).unwrap();
        let (ch, out) = crate::media::audio::decode_ogg_opus(&path).unwrap();
        assert_eq!(ch, 1);
        assert!(
            (out.len() as i64 - 48_000).abs() < 2_000,
            "decoded {} samples",
            out.len()
        );
        crate::media::prep::unstage(&path);
    }

    #[test]
    fn resamples_and_downmixes() {
        assert_eq!(to_mono(&[1.0, 0.0, 0.5, 0.5], 2), vec![0.5, 0.5]);
        assert_eq!(resample(&vec![0.0; 44_100], 44_100, 48_000).len(), 48_000);
    }
}
