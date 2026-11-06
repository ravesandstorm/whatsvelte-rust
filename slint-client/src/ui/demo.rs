//! Debug-only: fill the UI with sample chats (no session boot, no network) so
//! layouts can be checked without linking a real account. `WA_DEMO=1`
//! (`WA_DEMO=dark` forces the dark theme for the run).

use std::sync::Arc;

use crate::core::dto::{ChatDto, MediaDescriptorDto, MediaDto, MessageDto, QuotedDto, StatusDto};
use crate::store::{MessageStatus, UiMessage};
use crate::ui::controller::{Controller, DIRTY_ALL, lock};
use crate::util::time::now_secs;

impl Controller {
    pub async fn run_demo(self: Arc<Self>) {
        let now = now_secs();
        let me = "15550000000@s.whatsapp.net";
        {
            let mut v = lock(&self.view);
            v.booted = true;
            v.status = StatusDto {
                session_id: "default".into(),
                logged_in: true,
                connected: true,
                jid: Some(me.into()),
                registered: true,
                push_name: Some("Demo User".into()),
                ..Default::default()
            };
        }
        if let Ok(t) = std::env::var("WA_DEMO")
            && (t == "dark" || t == "light")
        {
            lock(&self.settings).theme = t;
        }
        let g = "120363000000000001@g.us";
        let ada = "919876543210@s.whatsapp.net";
        let bob = "447700900123@s.whatsapp.net";
        let thumb = {
            let img = image::RgbImage::from_fn(160, 120, |x, y| {
                image::Rgb([(x * 255 / 160) as u8, (y * 255 / 120) as u8, 160])
            });
            crate::media::prep::thumbnail_b64(&image::DynamicImage::ImageRgb8(img)).ok()
        };
        {
            let mut s = lock(&self.store);
            for (jid, name, last, ago, unread) in [
                (ada, Some("Ada Lovelace"), "See you at 6!", 60, 2),
                (g, Some("Rust Hackers"), "cargo build --release 🚀", 600, 3),
                (bob, None, "📷 Photo", 4000, 0),
                ("100000012345678@lid", None, "ok", 90_000, 0),
                (
                    "120363000000000002@newsletter",
                    Some("Slint News"),
                    "1.18 released",
                    200_000,
                    0,
                ),
                ("status@broadcast", None, "📷 Photo", 300_000, 0),
                (
                    "120363000000000003@g.us",
                    Some("Old Project"),
                    "archived chat",
                    900_000,
                    0,
                ),
            ] {
                s.upsert_chat_dto(&ChatDto {
                    jid: jid.into(),
                    name: name.map(Into::into),
                    last_message: Some(last.into()),
                    timestamp: now - ago,
                    unread,
                });
            }
            s.set_flag(bob, "pinned", true);
            s.set_flag(g, "muted", true);
            s.set_flag("120363000000000003@g.us", "archived", true);
            s.set_draft("100000012345678@lid", "unsent thought");
            let mk = |id: &str,
                      sender: &str,
                      from_me: bool,
                      ago: i64,
                      text: Option<&str>,
                      kind: &str| UiMessage {
                dto: MessageDto {
                    id: id.into(),
                    chat_jid: g.into(),
                    sender_jid: sender.into(),
                    from_me,
                    timestamp: now - ago,
                    text: text.map(Into::into),
                    kind: kind.into(),
                    ..Default::default()
                },
                status: if from_me {
                    MessageStatus::Read
                } else {
                    MessageStatus::None
                },
                ..Default::default()
            };
            let media = |kind: &str, name: Option<&str>, secs: Option<u32>| MediaDto {
                kind: kind.into(),
                file_name: name.map(Into::into),
                duration_secs: secs,
                width: Some(160),
                height: Some(120),
                descriptor: MediaDescriptorDto {
                    file_length: 482_133,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut msgs = vec![
                mk(
                    "m1",
                    ada,
                    false,
                    90_000,
                    Some("Has anyone tried Slint for desktop apps? Docs: https://slint.dev/docs"),
                    "text",
                ),
                mk(
                    "m2",
                    me,
                    true,
                    89_000,
                    Some("Yes! Rewriting our Tauri client in it right now.\\nNo WebView at all."),
                    "text",
                ),
                mk(
                    "m2b",
                    me,
                    true,
                    88_990,
                    Some("Memory went way down."),
                    "text",
                ),
            ];
            for k in 0..5 {
                let mut p = mk(&format!("al{k}"), bob, false, 5000 - k * 10, None, "image");
                p.dto.media = Some(media("image", None, None));
                p.dto.thumbnail = thumb.clone();
                msgs.push(p);
            }
            let mut reply = mk(
                "m4",
                me,
                true,
                2000,
                Some("Way lower — no JS heap, no IPC serialization."),
                "text",
            );
            reply.dto.quoted = Some(QuotedDto {
                id: "m1".into(),
                sender_jid: Some(ada.into()),
                text: Some("Has anyone tried Slint for desktop apps?".into()),
                kind: "text".into(),
                thumbnail: None,
            });
            for (k, e) in [("a", "🔥"), ("b", "🔥"), ("c", "👍")] {
                reply.reactions.insert(k.into(), e.into());
            }
            msgs.push(reply);
            let mut doc = mk("m5", ada, false, 1500, None, "document");
            doc.dto.media = Some(media("document", Some("benchmark-results.pdf"), None));
            msgs.push(doc);
            let mut voice = mk("m5b", bob, false, 1400, None, "audio");
            voice.dto.media = Some(media("audio", None, Some(14)));
            msgs.push(voice);
            msgs.push(mk(
                "m5c",
                ada,
                false,
                1300,
                Some("What should we name it?\\n• Wha.rs\\n• Slinky"),
                "poll",
            ));
            let mut edited = mk(
                "m6",
                me,
                true,
                600,
                Some("cargo build --release 🚀"),
                "text",
            );
            edited.edited_at = Some(now - 500);
            edited.status = MessageStatus::Delivered;
            msgs.push(edited);
            let mut deleted = mk("m7", bob, false, 300, None, "text");
            deleted.deleted = true;
            msgs.push(deleted);
            msgs.push(mk("m8", ada, false, 120, Some("Unread one"), "text"));
            msgs.push(mk("m9", bob, false, 60, Some("Unread two"), "text"));
            for m in msgs.iter_mut() {
                m.dto.text = m.dto.text.take().map(|t| t.replace("\\n", "\n"));
            }
            s.add_history(msgs);
        }
        if let Some(jid) = self.seed_long_chat(now, thumb.clone()) {
            self.clone().select_chat(jid).await;
            self.mark(DIRTY_ALL);
            return;
        }
        self.clone().select_chat(g.into()).await;
        // WA_DEMO_OPEN=<overlay> opens one overlay for layout checks.
        match std::env::var("WA_DEMO_OPEN").as_deref() {
            Ok("reactions") => self.open_reactions("m4".into()),
            Ok("delete") => self.ask_delete("m6".into()),
            Ok("album") => {
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                let row = lock(&self.view)
                    .albums
                    .keys()
                    .next()
                    .cloned()
                    .unwrap_or_default();
                self.open_album(row);
            }
            Ok("viewer") => {
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                self.view_image("al1".into());
            }
            Ok("reply") => self.reply_to("m1".into()),
            Ok("edit") => self.start_edit("m6".into()),
            Ok("suggest") => self.composer_edited("hey :smi".into(), 8),
            Ok("voice") => self.attach_voice(),
            Ok("settings") => lock(&self.view).open_settings = true,
            _ => {}
        }
        self.mark(DIRTY_ALL);
    }

    /// `WA_DEMO_LONG=<n>`: a persisted chat with `n` messages, every fourth one
    /// a photo backed by a real 2000x1500 file. For profiling long-chat
    /// scrolling (see `WA_SCROLL_TEST` in `main.rs`) with `WA_APP_DATA_DIR` set
    /// to a throwaway folder.
    fn seed_long_chat(&self, now: i64, thumb: Option<String>) -> Option<String> {
        let n: usize = std::env::var("WA_DEMO_LONG").ok()?.parse().ok()?;
        let jid = "919000000000@s.whatsapp.net".to_string();
        let photo = crate::core::paths::ui_cache_dir().join("long-chat-photo.jpg");
        let _ = std::fs::create_dir_all(crate::core::paths::ui_cache_dir());
        let img = image::RgbImage::from_fn(2000, 1500, |x, y| {
            image::Rgb([(x % 251) as u8, (y % 241) as u8, ((x ^ y) % 239) as u8])
        });
        if let Err(e) = img.save(&photo) {
            log::error!("long chat photo: {e}");
        }
        let mut s = lock(&self.store);
        s.open_cache(crate::store::persist::Persist::new(
            crate::core::paths::ui_cache_dir(),
        ));
        s.upsert_chat_dto(&ChatDto {
            jid: jid.clone(),
            name: Some("Long chat".into()),
            last_message: Some("newest".into()),
            timestamp: now,
            // WA_DEMO_LONG_UNREAD=<n>: opens at the "Unread" divider.
            unread: std::env::var("WA_DEMO_LONG_UNREAD")
                .ok()
                .and_then(|u| u.parse().ok())
                .unwrap_or(0),
        });
        let mut paths = lock(&self.media_paths);
        let msgs: Vec<UiMessage> = (0..n)
            .map(|i| {
                let id = format!("long{i}");
                let is_photo = i % 4 == 3;
                let text = match i % 7 {
                    _ if is_photo => None,
                    0 => Some(format!("Message {i}: a longer line that wraps across the bubble a couple of times, to vary row heights while scrolling through the history of this chat.")),
                    1 => Some(format!("{i} short")),
                    2 => Some(format!("Message {i}\nwith\nseveral\nlines")),
                    3 => Some(format!("Message {i} with a link https://example.com/{i}")),
                    _ => Some(format!("Message number {i}")),
                };
                let mut m = UiMessage {
                    dto: MessageDto {
                        id: id.clone(),
                        chat_jid: jid.clone(),
                        sender_jid: jid.clone(),
                        from_me: i % 3 == 0,
                        // Spaced apart so photos don't merge into inferred albums.
                        timestamp: now - ((n - i) as i64) * 120,
                        text,
                        kind: if is_photo { "image" } else { "text" }.into(),
                        ..Default::default()
                    },
                    status: MessageStatus::Read,
                    ..Default::default()
                };
                if is_photo {
                    paths.insert(id, photo.clone());
                    m.dto.thumbnail = thumb.clone();
                    m.dto.media = Some(MediaDto {
                        kind: "image".into(),
                        width: Some(2000),
                        height: Some(1500),
                        descriptor: MediaDescriptorDto {
                            file_length: 482_133,
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                }
                m
            })
            .collect();
        drop(paths);
        s.add_history(msgs);
        s.flush();
        Some(jid)
    }
}
