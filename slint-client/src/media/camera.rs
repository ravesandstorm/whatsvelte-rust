//! Camera photo capture (replaces getUserMedia + canvas). A capture thread
//! streams downscaled preview frames to the UI and keeps the latest full
//! frame; "Capture" encodes that frame as a JPEG (q95).

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use image::RgbImage;
use nokhwa::Camera;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};

pub struct CameraSession {
    stop: Arc<AtomicBool>,
    latest: Arc<Mutex<Option<RgbImage>>>,
}

impl CameraSession {
    /// Start streaming. `on_frame` receives preview frames (≤640px);
    /// `on_error` is called once if the camera can't be opened.
    pub fn start(
        on_frame: impl Fn(RgbImage) + Send + 'static,
        on_error: impl Fn(String) + Send + 'static,
    ) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let latest = Arc::new(Mutex::new(None));
        let (stop2, latest2) = (stop.clone(), latest.clone());
        std::thread::Builder::new()
            .name("camera".into())
            .spawn(move || {
                // macOS asks for camera permission on first use.
                #[cfg(target_os = "macos")]
                {
                    let (tx, rx) = std::sync::mpsc::channel();
                    nokhwa::nokhwa_initialize(move |granted| {
                        let _ = tx.send(granted);
                    });
                    if !rx
                        .recv_timeout(std::time::Duration::from_secs(60))
                        .unwrap_or(false)
                    {
                        on_error("Camera permission was denied.".into());
                        return;
                    }
                }
                let format = RequestedFormat::new::<RgbFormat>(
                    RequestedFormatType::AbsoluteHighestFrameRate,
                );
                let mut cam = match Camera::new(CameraIndex::Index(0), format)
                    .and_then(|mut c| c.open_stream().map(|_| c))
                {
                    Ok(c) => c,
                    Err(e) => {
                        on_error(e.to_string());
                        return;
                    }
                };
                let mut last_preview =
                    std::time::Instant::now() - std::time::Duration::from_secs(1);
                while !stop2.load(Ordering::Relaxed) {
                    let frame = match cam.frame().and_then(|b| b.decode_image::<RgbFormat>()) {
                        Ok(f) => f,
                        Err(e) => {
                            on_error(e.to_string());
                            break;
                        }
                    };
                    let Some(img) =
                        RgbImage::from_raw(frame.width(), frame.height(), frame.into_raw())
                    else {
                        continue;
                    };
                    // ~15 fps preview, downscaled to keep UI memory flat.
                    if last_preview.elapsed() >= std::time::Duration::from_millis(66) {
                        last_preview = std::time::Instant::now();
                        let small = image::DynamicImage::ImageRgb8(img.clone())
                            .thumbnail(640, 640)
                            .into_rgb8();
                        on_frame(small);
                    }
                    *latest2.lock().unwrap_or_else(|p| p.into_inner()) = Some(img);
                }
                let _ = cam.stop_stream();
            })
            .expect("spawn camera thread");
        Self { stop, latest }
    }

    /// Encode the newest full-resolution frame to a JPEG (q95).
    pub fn capture(&self) -> anyhow::Result<PathBuf> {
        let img = self
            .latest
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no frame yet"))?;
        let path = crate::media::prep::stage_path("jpg");
        let mut out = std::io::BufWriter::new(std::fs::File::create(&path)?);
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 95).encode_image(&img)?;
        Ok(path)
    }
}

impl Drop for CameraSession {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
