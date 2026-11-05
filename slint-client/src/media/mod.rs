//! Native replacements for the WebView media stack: attachment preparation
//! (canvas compression/thumbnails), audio playback (<audio>), voice recording
//! (MediaRecorder), camera capture (getUserMedia) and the Klipy GIF API (fetch).

pub mod audio;
pub mod camera;
pub mod klipy;
pub mod prep;
pub mod voice;
