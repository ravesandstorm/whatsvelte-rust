//! Media: downloads + decode cache, avatars, lightbox, inline audio, albums,
//! attachments (picker, paste, drop, preview, compression), voice/camera
//! capture, Klipy GIFs and the custom wallpaper.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::core::api::Api;
use crate::core::media::MediaSendOptionsIn;
use crate::core::paths;
use crate::media::camera::CameraSession;
use crate::media::klipy;
use crate::media::prep;
use crate::media::voice::VoiceRecorder;
use crate::store::UiMessage;
use crate::ui::controller::{lock, Controller, Preview, Rec, Viewer, CTRL, DIRTY_ALL, DIRTY_CHATS, DIRTY_CONV, DIRTY_OVERLAY, DIRTY_SESSION};
use crate::ui::images::{self, Pixels};

/// Longest edge decoded for the lightbox.
const VIEWER_EDGE: u32 = 2048;

fn ctrl() -> Option<Arc<Controller>> {
    CTRL.get().cloned()
}

fn rgba_pixels(img: &image::DynamicImage, max_edge: u32) -> Pixels {
    let img = if img.width() > max_edge || img.height() > max_edge { img.thumbnail(max_edge, max_edge) } else { img.clone() };
    let rgba = img.into_rgba8();
    slint::SharedPixelBuffer::clone_from_slice(rgba.as_raw(), rgba.width(), rgba.height())
}

impl Controller {
    // ================================================================ downloads

    pub(crate) fn spawn_image_load(&self, m: UiMessage) {
        if lock(&self.media_failed).contains(&m.dto.id) || !lock(&self.media_inflight).insert(m.dto.id.clone()) {
            return;
        }
        let api = self.api.clone();
        let id = m.dto.id.clone();
        self.rt.spawn(async move {
            let Some(ctrl) = ctrl() else { return };
            if let Some(path) = ctrl.media_path(&api, &m, false).await {
                let px = tokio::task::spawn_blocking(move || images::decode_file(&path, images::BUBBLE_MAX_EDGE))
                    .await
                    .ok()
                    .flatten();
                match px {
                    Some(px) => lock(&ctrl.images).insert(format!("full:{id}"), px),
                    None => {
                        lock(&ctrl.media_failed).insert(id.clone());
                    }
                }
            } else {
                lock(&ctrl.media_failed).insert(id.clone());
            }
            lock(&ctrl.media_inflight).remove(&id);
            ctrl.mark(DIRTY_CONV | DIRTY_OVERLAY);
        });
    }

    /// Download (or hit the cache for) a message's media; returns its path.
    pub(crate) async fn media_path(&self, api: &Api, m: &UiMessage, loud: bool) -> Option<PathBuf> {
        if let Some(p) = lock(&self.media_paths).get(&m.dto.id).filter(|p| p.exists()) {
            return Some(p.clone());
        }
        let md = m.dto.media.as_ref()?;
        match api.download_media(md.descriptor.clone(), md.mimetype.clone(), md.file_name.clone()).await {
            Ok(p) => {
                let p = PathBuf::from(p);
                lock(&self.media_paths).insert(m.dto.id.clone(), p.clone());
                Some(p)
            }
            Err(e) => {
                log::warn!("media download failed for {}: {e}", m.dto.id);
                if loud {
                    self.toast("Couldn't load media");
                }
                None
            }
        }
    }

    async fn with_spinner<T>(&self, id: &str, fut: impl std::future::Future<Output = T>) -> T {
        lock(&self.media_inflight).insert(id.to_string());
        self.mark(DIRTY_CONV);
        let out = fut.await;
        lock(&self.media_inflight).remove(id);
        self.mark(DIRTY_CONV);
        out
    }

    /// Documents / videos: open the decrypted file in the OS default app.
    pub async fn open_media(self: Arc<Self>, id: String) {
        let Some(m) = self.active_message(&id) else { return };
        if m.dto.media.is_none() {
            return;
        }
        let api = self.api.clone();
        let path = self.with_spinner(&id, self.media_path(&api, &m, true)).await;
        if let Some(p) = path
            && let Err(e) = open::that_detached(&p)
        {
            self.toast(format!("Couldn't open file: {e}"));
        }
    }

    pub async fn save_media(self: Arc<Self>, id: String) {
        let Some(m) = self.active_message(&id) else { return };
        let Some(md) = m.dto.media.clone() else { return };
        let in_viewer = self.viewer_set_label("Saving…", &id);
        match self.api.save_media_to_downloads(md.descriptor, md.mimetype, md.file_name).await {
            Ok(p) => {
                if in_viewer {
                    self.viewer_set_label("✓ Saved", &id);
                } else {
                    let name = Path::new(&p).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or(p);
                    self.toast(format!("Saved to Downloads: {name}"));
                }
            }
            Err(e) => {
                log::warn!("save failed: {e}");
                if in_viewer {
                    self.viewer_set_label("⬇ Download", &id);
                }
                self.toast("Couldn't save the file");
            }
        }
    }

    // ================================================================ lightbox

    fn viewer_set_label(&self, label: &str, id: &str) -> bool {
        let mut v = lock(&self.view);
        let Some(viewer) = v.viewer.as_mut() else { return false };
        if viewer.ids.get(viewer.index).map(String::as_str) != Some(id) {
            return false;
        }
        viewer.save_label = label.into();
        drop(v);
        self.mark(DIRTY_OVERLAY);
        true
    }

    /// Open the lightbox on `id`; album items become a gallery.
    pub fn view_image(self: &Arc<Self>, id: String) {
        let gallery = {
            let v = lock(&self.view);
            v.albums.values().find(|ids| ids.contains(&id)).cloned().unwrap_or_else(|| vec![id.clone()])
        };
        let index = gallery.iter().position(|x| *x == id).unwrap_or(0);
        lock(&self.view).viewer = Some(Viewer {
            ids: gallery,
            index,
            image: None,
            loading: true,
            error: String::new(),
            is_video: false,
            save_label: "⬇ Download".into(),
            generation: 0,
        });
        self.load_viewer();
    }

    pub fn viewer_step(self: &Arc<Self>, dir: i32) {
        {
            let mut v = lock(&self.view);
            let Some(viewer) = v.viewer.as_mut() else { return };
            let next = viewer.index as i32 + dir;
            if next < 0 || next >= viewer.ids.len() as i32 {
                return;
            }
            viewer.index = next as usize;
            viewer.save_label = "⬇ Download".into();
        }
        self.load_viewer();
    }

    pub fn close_viewer(&self) {
        lock(&self.view).viewer = None;
        self.mark(DIRTY_OVERLAY);
    }

    /// Show the thumbnail immediately, then the decoded full image (stale
    /// loads are dropped via the generation counter).
    fn load_viewer(self: &Arc<Self>) {
        let (id, generation) = {
            let mut v = lock(&self.view);
            let Some(viewer) = v.viewer.as_mut() else { return };
            viewer.generation += 1;
            (viewer.ids[viewer.index].clone(), viewer.generation)
        };
        let Some(m) = self.active_message(&id) else { return };
        let is_video = m.dto.kind == "video";
        let thumb = {
            let cache = lock(&self.images);
            cache.get(&format!("full:{id}")).or_else(|| cache.get(&format!("thumb:{id}")))
        };
        let thumb = thumb.or_else(|| m.dto.thumbnail.as_deref().and_then(images::decode_b64));
        {
            let mut v = lock(&self.view);
            if let Some(viewer) = v.viewer.as_mut() {
                viewer.image = thumb;
                viewer.is_video = is_video;
                viewer.loading = !is_video;
                viewer.error.clear();
            }
        }
        self.mark(DIRTY_OVERLAY);
        if is_video {
            return; // poster + "Play in system player"
        }
        let me = self.clone();
        self.rt.spawn(async move {
            let api = me.api.clone();
            let path = me.media_path(&api, &m, false).await;
            let px = match path {
                Some(p) => tokio::task::spawn_blocking(move || images::decode_file(&p, VIEWER_EDGE)).await.ok().flatten(),
                None => None,
            };
            let mut v = lock(&me.view);
            let Some(viewer) = v.viewer.as_mut().filter(|x| x.generation == generation) else { return };
            viewer.loading = false;
            match px {
                Some(px) => viewer.image = Some(px),
                None => viewer.error = "Couldn't load image".into(),
            }
            drop(v);
            me.mark(DIRTY_OVERLAY);
        });
    }

    pub async fn viewer_play(self: Arc<Self>) {
        let id = {
            let v = lock(&self.view);
            v.viewer.as_ref().and_then(|x| x.ids.get(x.index).cloned())
        };
        if let Some(id) = id {
            self.open_media(id).await;
        }
    }

    pub fn open_album(&self, row_id: String) {
        lock(&self.view).open_album = Some(row_id);
        self.mark(DIRTY_OVERLAY);
    }

    // ================================================================ audio

    /// Play / pause a voice note or audio message inline (one at a time).
    pub async fn toggle_audio(self: Arc<Self>, id: String) {
        let Some(m) = self.active_message(&id) else { return };
        let api = self.api.clone();
        let path = self.with_spinner(&id, self.media_path(&api, &m, true)).await;
        if let Some(p) = path {
            self.audio.toggle(&id, &p);
        }
        self.mark(DIRTY_CONV);
    }

    // ================================================================ avatars

    /// Lazily fetch a chat's profile picture (once per session per jid).
    pub fn need_avatar(self: &Arc<Self>, jid: String) {
        if jid.is_empty() || !lock(&self.avatar_fetched).insert(jid.clone()) {
            return;
        }
        let me = self.clone();
        self.rt.spawn(async move {
            let Ok(_permit) = me.avatar_permits.acquire().await else { return };
            let canonical = lock(&me.store).canonical(&jid);
            let contact = match me.api.get_contact(canonical.clone()).await {
                Ok(c) => c,
                Err(e) => {
                    log::debug!("contact lookup failed for {jid}: {e}");
                    return;
                }
            };
            if let Some(name) = contact.name.as_deref().filter(|n| !n.is_empty()) {
                lock(&me.store).learn_name(&canonical, name);
            }
            let Some(url) = contact.picture_url else {
                me.mark(DIRTY_CHATS);
                return;
            };
            let px = tokio::task::spawn_blocking(move || {
                let bytes = klipy::download(&url).ok()?;
                let img = image::load_from_memory(&bytes).ok()?;
                Some(rgba_pixels(&img, 128))
            })
            .await
            .ok()
            .flatten();
            if let Some(px) = px {
                lock(&me.avatars).insert(jid, px);
                me.mark(DIRTY_CHATS | DIRTY_CONV);
            }
        });
    }

    // ================================================================ attachments

    fn open_preview(&self, kind: &str, path: PathBuf, staged: bool, duration: Option<u32>) {
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "file".into());
        let image = if kind == "image" {
            prep::decode(&path).ok().map(|img| rgba_pixels(&img, 1200))
        } else {
            None
        };
        lock(&self.view).preview = Some(Preview { kind: kind.into(), path, name, image, duration, staged });
        self.mark(DIRTY_OVERLAY);
    }

    pub async fn attach_photos(self: Arc<Self>) {
        if self.active().is_none() {
            return;
        }
        let picked = rfd::AsyncFileDialog::new()
            .set_title("Send photos")
            .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
            .pick_file()
            .await;
        if let Some(f) = picked {
            let me = self.clone();
            let path = f.path().to_path_buf();
            let _ = tokio::task::spawn_blocking(move || me.open_preview("image", path, false, None)).await;
        }
    }

    pub async fn attach_document(self: Arc<Self>) {
        if self.active().is_none() {
            return;
        }
        if let Some(f) = rfd::AsyncFileDialog::new().set_title("Send a document").pick_file().await {
            self.open_preview("document", f.path().to_path_buf(), false, None);
        }
    }

    /// A file dropped on the window: images preview as photos, the rest as documents.
    pub fn dropped_file(self: &Arc<Self>, path: PathBuf) {
        if self.active().is_none() || !path.is_file() {
            return;
        }
        let me = self.clone();
        self.rt.spawn_blocking(move || {
            let kind = if prep::is_image_path(&path) { "image" } else { "document" };
            me.open_preview(kind, path, false, None);
        });
    }

    /// Ctrl/Cmd+V with an image on the clipboard → photo preview.
    pub fn paste_image(self: &Arc<Self>) {
        if self.active().is_none() {
            return;
        }
        let me = self.clone();
        self.rt.spawn_blocking(move || {
            let Ok(mut clip) = arboard::Clipboard::new() else { return };
            let Ok(img) = clip.get_image() else { return };
            let Some(buf) = image::RgbaImage::from_raw(img.width as u32, img.height as u32, img.bytes.into_owned()) else { return };
            let path = prep::stage_path("png").with_file_name(format!("pasted-image-{}.png", crate::util::time::now_secs()));
            if buf.save(&path).is_ok() {
                me.open_preview("image", path, true, None);
            }
        });
    }

    pub fn preview_cancel(&self) {
        self.audio.stop();
        if let Some(p) = lock(&self.view).preview.take()
            && p.staged
        {
            let _ = std::fs::remove_file(&p.path);
        }
        self.mark(DIRTY_OVERLAY);
    }

    pub fn preview_play(&self) {
        let path = lock(&self.view).preview.as_ref().map(|p| p.path.clone());
        if let Some(p) = path {
            self.audio.toggle("preview", &p);
            self.mark(DIRTY_OVERLAY);
        }
    }

    /// Send the previewed attachment (port of send-media.ts rules).
    pub async fn preview_send(self: Arc<Self>, caption: String, hd: bool) {
        self.audio.stop();
        let Some(p) = lock(&self.view).preview.take() else { return };
        self.mark(DIRTY_OVERLAY);
        let Some(jid) = self.active() else { return };
        let caption = Some(caption.trim().to_string()).filter(|c| !c.is_empty() && p.kind != "audio");

        // Prepare off the async workers: compression / thumbnails.
        let kind = p.kind.clone();
        let src = p.path.clone();
        let prepared = tokio::task::spawn_blocking(move || -> anyhow::Result<(PathBuf, Option<String>, Option<String>, bool)> {
            match kind.as_str() {
                "image" if prep::is_gif(&src) => {
                    let thumb = prep::decode(&src).ok().and_then(|i| prep::thumbnail_b64(&i).ok());
                    let staged = prep::stage_path("gif");
                    std::fs::copy(&src, &staged)?;
                    Ok((staged, thumb, Some("image/gif".into()), true))
                }
                "image" => {
                    let (path, thumb) = prep::compress_image(&src, hd)?;
                    Ok((path, Some(thumb), Some("image/jpeg".into()), true))
                }
                "audio" => Ok((src, None, Some("audio/ogg; codecs=opus".into()), false)),
                _ => Ok((src.clone(), None, prep::mime_for_path(&src).map(str::to_string), false)),
            }
        })
        .await;
        let (path, thumb, mimetype, staged_here) = match prepared {
            Ok(Ok(x)) => x,
            _ => {
                self.toast("Couldn't prepare the attachment");
                return;
            }
        };

        let label = match p.kind.as_str() {
            "image" => caption.clone().or_else(|| Some("📷 Photo".into())),
            "audio" => Some("🎤 Voice message".into()),
            _ => caption.clone().or_else(|| Some(format!("📄 {}", p.name))),
        };
        // With a thumbnail the bubble shows the caption; otherwise the label.
        let text = if thumb.is_some() { caption.clone() } else { label };
        let tmp = lock(&self.store).add_optimistic(&jid, &p.kind, text, thumb.clone(), None);
        lock(&self.view).scroll_token += 1;
        self.mark(DIRTY_CHATS | DIRTY_CONV);

        let opts = MediaSendOptionsIn {
            caption,
            mimetype,
            file_name: (p.kind == "document").then(|| p.name.clone()),
            duration_secs: p.duration,
            ptt: (p.kind == "audio").then_some(true),
            jpeg_thumbnail: thumb,
            gif_playback: None,
        };
        let result = self
            .api
            .send_media(jid.clone(), path.to_string_lossy().into_owned(), p.kind.clone(), Some(opts))
            .await;
        if staged_here {
            prep::unstage(&path);
        }
        if p.staged {
            let _ = std::fs::remove_file(&p.path);
        }
        self.finish_send(&jid, &tmp, result.map(|r| r.message_id), "Attachment not sent");
    }

    // ================================================================ recording

    pub fn attach_voice(self: &Arc<Self>) {
        if self.active().is_none() {
            return;
        }
        self.audio.stop();
        lock(&self.view).rec = Some(Rec::Voice(VoiceRecorder::start()));
        self.mark(DIRTY_OVERLAY);
    }

    pub fn attach_camera(self: &Arc<Self>) {
        if self.active().is_none() {
            return;
        }
        let (a, b) = (self.clone(), self.clone());
        let session = CameraSession::start(
            move |frame| {
                let px = rgba_pixels(&image::DynamicImage::ImageRgb8(frame), 640);
                if let Some(Rec::Photo { frame, .. }) = lock(&a.view).rec.as_mut() {
                    *frame = Some(px);
                }
                a.mark(DIRTY_OVERLAY);
            },
            move |err| {
                if let Some(Rec::Photo { error, .. }) = lock(&b.view).rec.as_mut() {
                    *error = Some(err);
                }
                b.mark(DIRTY_OVERLAY);
            },
        );
        lock(&self.view).rec = Some(Rec::Photo { session, frame: None, error: None });
        self.mark(DIRTY_OVERLAY);
    }

    pub fn rec_cancel(&self) {
        if let Some(Rec::Voice(r)) = lock(&self.view).rec.take() {
            r.cancel();
        }
        self.mark(DIRTY_OVERLAY);
    }

    /// Voice: stop, encode to Ogg/Opus and open the preview.
    pub async fn rec_stop(self: Arc<Self>) {
        let rec = lock(&self.view).rec.take();
        self.mark(DIRTY_OVERLAY);
        let Some(Rec::Voice(r)) = rec else { return };
        match tokio::task::spawn_blocking(move || r.finish()).await {
            Ok(Ok((path, secs))) => self.open_preview("audio", path, true, Some(secs)),
            Ok(Err(e)) => self.toast(format!("Recording failed: {e}")),
            Err(_) => self.toast("Recording failed"),
        }
    }

    /// Camera: grab the current frame as a JPEG and open the photo preview.
    pub async fn rec_capture(self: Arc<Self>) {
        let rec = lock(&self.view).rec.take();
        self.mark(DIRTY_OVERLAY);
        let Some(Rec::Photo { session, .. }) = rec else { return };
        let me = self.clone();
        let _ = tokio::task::spawn_blocking(move || match session.capture() {
            Ok(path) => me.open_preview("image", path, true, None),
            Err(e) => me.toast(format!("Capture failed: {e}")),
        })
        .await;
    }

    // ================================================================ GIFs

    /// New search (debounced 350 ms; blank = trending).
    pub fn gif_search(self: &Arc<Self>, query: String) {
        let seq = {
            let mut v = lock(&self.view);
            v.gif.seq += 1;
            v.gif.query = query;
            v.gif.seq
        };
        let me = self.clone();
        self.rt.spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(350)).await;
            {
                let mut v = lock(&me.view);
                if v.gif.seq != seq {
                    return;
                }
                v.gif.items.clear();
                v.gif.previews.clear();
                v.gif.page = 0;
                v.gif.has_next = true;
                v.gif.status.clear();
            }
            me.gif_load_page(seq).await;
        });
    }

    pub fn gif_more(self: &Arc<Self>) {
        let seq = {
            let v = lock(&self.view);
            if v.gif.loading || !v.gif.has_next || v.gif.page == 0 {
                return;
            }
            v.gif.seq
        };
        let me = self.clone();
        self.rt.spawn(async move { me.gif_load_page(seq).await });
    }

    async fn gif_load_page(self: Arc<Self>, seq: u64) {
        let Some(key) = klipy::api_key() else { return };
        let (query, page) = {
            let mut v = lock(&self.view);
            if v.gif.loading {
                return;
            }
            v.gif.loading = true;
            (v.gif.query.clone(), v.gif.page + 1)
        };
        self.mark(DIRTY_OVERLAY);
        let result = tokio::task::spawn_blocking(move || klipy::fetch(&key, &query, page)).await;
        let items = {
            let mut v = lock(&self.view);
            v.gif.loading = false;
            if v.gif.seq != seq {
                return;
            }
            match result {
                Ok(Ok(p)) => {
                    v.gif.page = page;
                    v.gif.has_next = p.has_next;
                    v.gif.items.extend(p.items.iter().cloned());
                    p.items
                }
                Ok(Err(e)) => {
                    v.gif.status = e.to_string();
                    Vec::new()
                }
                Err(_) => Vec::new(),
            }
        };
        self.mark(DIRTY_OVERLAY);
        // Previews: first frame of each (static), a few at a time.
        for chunk in items.chunks(6) {
            let mut handles = Vec::new();
            for g in chunk.iter().cloned() {
                handles.push(tokio::task::spawn_blocking(move || {
                    let bytes = klipy::download(&g.preview).ok()?;
                    let img = image::load_from_memory(&bytes).ok()?;
                    Some((g.id, rgba_pixels(&img, 240)))
                }));
            }
            for h in handles {
                if let Ok(Some((id, px))) = h.await {
                    let mut v = lock(&self.view);
                    if v.gif.seq == seq {
                        v.gif.previews.insert(id, px);
                    }
                }
            }
            self.mark(DIRTY_OVERLAY);
        }
    }

    /// Send a GIF: the MP4 rendition with gif-playback (loops inline), else
    /// the GIF bytes as an image. No preview step (like the Svelte picker).
    pub async fn gif_picked(self: Arc<Self>, id: String) {
        let Some(jid) = self.active() else { return };
        let (gif, preview) = {
            let v = lock(&self.view);
            (v.gif.items.iter().find(|g| g.id == id).cloned(), v.gif.previews.get(&id).cloned())
        };
        let Some(gif) = gif else { return };
        let thumb = preview.and_then(|px| {
            let img = image::RgbaImage::from_raw(px.width(), px.height(), px.as_bytes().to_vec())?;
            prep::thumbnail_b64(&image::DynamicImage::ImageRgba8(img)).ok()
        });
        let (url, is_mp4) = match &gif.mp4 {
            Some(u) => (u.clone(), true),
            None => (gif.gif.clone(), false),
        };
        let downloaded = tokio::task::spawn_blocking(move || {
            let bytes = klipy::download(&url)?;
            let path = prep::stage_path(if is_mp4 { "mp4" } else { "gif" });
            std::fs::write(&path, bytes)?;
            anyhow::Ok(path)
        })
        .await;
        let Ok(Ok(path)) = downloaded else {
            self.toast("Couldn't download the GIF");
            return;
        };
        let kind = if is_mp4 { "video" } else { "image" };
        let tmp = lock(&self.store).add_optimistic(&jid, kind, None, thumb.clone(), None);
        if let Some(c) = lock(&self.store).chats.get_mut(&jid) {
            c.last_message = Some("🎞️ GIF".into());
        }
        lock(&self.view).scroll_token += 1;
        self.mark(DIRTY_CHATS | DIRTY_CONV);
        let opts = MediaSendOptionsIn {
            mimetype: Some(if is_mp4 { "video/mp4" } else { "image/gif" }.into()),
            jpeg_thumbnail: thumb,
            gif_playback: is_mp4.then_some(true),
            ..Default::default()
        };
        let result = self.api.send_media(jid.clone(), path.to_string_lossy().into_owned(), kind.into(), Some(opts)).await;
        prep::unstage(&path);
        self.finish_send(&jid, &tmp, result.map(|r| r.message_id), "GIF not sent");
    }

    // ================================================================ wallpaper

    pub(crate) fn load_wallpaper(&self) {
        let path = lock(&self.settings).wallpaper_path.clone();
        let px = path.and_then(|p| prep::decode(Path::new(&p)).ok()).map(|img| rgba_pixels(&img, 1600));
        *lock(&self.wallpaper) = px;
        self.mark(DIRTY_SESSION);
    }

    pub async fn set_wallpaper(self: Arc<Self>, choice: String) {
        if choice != "custom" {
            self.update_settings(|s| s.wallpaper = choice);
            return;
        }
        let picked = rfd::AsyncFileDialog::new()
            .set_title("Choose a wallpaper")
            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "bmp", "gif"])
            .pick_file()
            .await;
        let Some(f) = picked else { return };
        let src = f.path().to_path_buf();
        let ext = src.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_else(|| "img".into());
        let dest = paths::app_data_dir().join(format!("wallpaper.{ext}"));
        if let Err(e) = std::fs::copy(&src, &dest) {
            self.toast(format!("Couldn't use that image: {e}"));
            return;
        }
        self.update_settings(|s| {
            s.wallpaper = "custom".into();
            s.wallpaper_path = Some(dest.to_string_lossy().into_owned());
        });
        let me = self.clone();
        let _ = tokio::task::spawn_blocking(move || me.load_wallpaper()).await;
        self.mark(DIRTY_ALL);
    }
}
