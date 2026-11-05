//! Frame building: turns store + view state into the `Send` frame structs the
//! UI thread copies into Slint models.

use std::collections::HashMap;

use crate::core::dto::MessageDto;
use crate::store::{media_label, quoted_text, MessageStatus, Section, Store, UiMessage};
use crate::ui::controller::{lock, Controller, Rec, DIRTY_CHATS, DIRTY_CONV, DIRTY_OVERLAY, DIRTY_SESSION};
use crate::ui::frame::{
    ChatRowData, ChatsFrame, ConvFrame, Frame, MsgRowData, OverlayFrame, PreviewData, SessionFrame, TileData, ViewerData,
};
use crate::ui::images::{self, Pixels};
use crate::util::emoji;
use crate::util::jid::{format_phone, initials, is_group, is_newsletter, is_phone, is_self, is_status, normalize_jid};
use crate::util::time::{clock, day_label, list_stamp, local_day};

/// Stable avatar / sender tint per jid.
pub fn tint_for(jid: &str) -> u32 {
    const PALETTE: [u32; 10] = [
        0x25d366, 0x34b7f1, 0xe542a3, 0xff8a65, 0x7e57c2, 0x26a69a, 0xf4511e, 0x5c6bc0, 0x8d6e63, 0x00897b,
    ];
    let h = jid.bytes().fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32));
    PALETTE[(h % PALETTE.len() as u32) as usize]
}

/// Avatar letters: "#" for groups; a person glyph for unnamed numbers.
fn avatar_initials(jid: &str, title: &str) -> String {
    if is_group(jid) {
        "#".into()
    } else if title.starts_with('+') || title.starts_with('~') || title.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        "👤".into()
    } else {
        initials(title)
    }
}

fn structured_label(kind: &str) -> &'static str {
    match kind {
        "buttons" | "list" | "interactive" | "template" => "Interactive",
        "poll" => "Poll",
        "order" => "Order",
        "product" => "Product",
        "contact" => "Contact",
        "location" => "Location",
        _ => "",
    }
}

fn fmt_duration(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

fn fmt_size(bytes: u64) -> String {
    match bytes {
        0 => String::new(),
        b if b < 1024 => format!("{b} B"),
        b if b < 1024 * 1024 => format!("{:.0} KB", b as f64 / 1024.0),
        b => format!("{:.1} MB", b as f64 / (1024.0 * 1024.0)),
    }
}

fn status_code(s: MessageStatus) -> i32 {
    match s {
        MessageStatus::None => 0,
        MessageStatus::Failed => 1,
        MessageStatus::Sending => 2,
        MessageStatus::Sent => 3,
        MessageStatus::Delivered => 4,
        MessageStatus::Read => 5,
        MessageStatus::Played => 6,
    }
}

// ---------------------------------------------------------------- links

/// Find links (port of Linkify's `/(https?:\/\/[^\s<]+|www\.[^\s<]+\.[^\s<]+)/gi`
/// with trailing punctuation stripped). Returns byte ranges.
pub fn find_links(text: &str) -> Vec<(usize, usize)> {
    let lower = text.to_ascii_lowercase();
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if !text.is_char_boundary(i) {
            i += 1;
            continue;
        }
        let rest = &lower[i..];
        let is_start = rest.starts_with("http://") || rest.starts_with("https://") || rest.starts_with("www.");
        if !is_start {
            i += 1;
            continue;
        }
        let mut end = i;
        while end < bytes.len() && !bytes[end].is_ascii_whitespace() && bytes[end] != b'<' {
            end += 1;
        }
        while end > i && b".,;:!?)]}>'\"".contains(&bytes[end - 1]) {
            end -= 1;
        }
        let token = &lower[i..end];
        let valid = if token.starts_with("www.") { token[4..].contains('.') } else { token.contains("://") && token.len() > 8 };
        if valid && end > i {
            out.push((i, end));
            i = end;
        } else {
            i += 1;
        }
    }
    out
}

fn md_escape(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '\\' | '`' | '*' | '_' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '.' | '!' | '<' | '>' | '~' | '|' | '{' | '}' | '&' => {
                out.push('\\');
                out.push(c);
            }
            '\n' => out.push_str("\\\n"),
            c => out.push(c),
        }
    }
}

/// Markdown for a message with clickable links (None when there are none).
pub fn linkify_markdown(text: &str) -> Option<String> {
    let links = find_links(text);
    if links.is_empty() {
        return None;
    }
    let mut out = String::with_capacity(text.len() + 32);
    let mut last = 0;
    for (s, e) in links {
        md_escape(&text[last..s], &mut out);
        let shown = &text[s..e];
        let href = if shown.to_ascii_lowercase().starts_with("www.") { format!("https://{shown}") } else { shown.to_string() };
        out.push('[');
        md_escape(shown, &mut out);
        out.push_str("](<");
        out.push_str(&href.replace('>', "%3E"));
        out.push_str(">)");
        last = e;
    }
    md_escape(&text[last..], &mut out);
    Some(out)
}

// ---------------------------------------------------------------- albums

fn albumable(m: &UiMessage) -> bool {
    matches!(m.dto.kind.as_str(), "image" | "video")
        && m.dto.media.is_some()
        && !m.deleted
        && m.dto.quoted.is_none()
        && !m.dto.id.starts_with("tmp-")
}

fn same_sender(a: &MessageDto, b: &MessageDto) -> bool {
    a.from_me == b.from_me && a.sender_jid == b.sender_jid
}

fn no_text(m: &UiMessage) -> bool {
    m.dto.text.as_deref().is_none_or(|t| t.trim().is_empty())
}

/// Length of the album starting at `i` (port of album.ts): explicit albums
/// (shared `albumParentId`, ≥2 items) or inferred runs of caption-less media
/// from one sender, each ≤60 s apart (≥4 items). Returns 1 when not an album.
pub fn album_len(msgs: &[UiMessage], i: usize) -> usize {
    let first = &msgs[i];
    if !albumable(first) {
        return 1;
    }
    if let Some(parent) = &first.dto.album_parent_id {
        let n = 1 + msgs[i + 1..]
            .iter()
            .take_while(|m| albumable(m) && m.dto.album_parent_id.as_ref() == Some(parent) && same_sender(&m.dto, &first.dto))
            .count();
        return if n >= 2 { n } else { 1 };
    }
    if !no_text(first) {
        return 1;
    }
    let mut n = 1;
    while i + n < msgs.len() {
        let (prev, m) = (&msgs[i + n - 1], &msgs[i + n]);
        if albumable(m)
            && m.dto.album_parent_id.is_none()
            && no_text(m)
            && same_sender(&m.dto, &first.dto)
            && m.dto.timestamp - prev.dto.timestamp <= 60
        {
            n += 1;
        } else {
            break;
        }
    }
    if n >= 4 { n } else { 1 }
}

impl Controller {
    pub(crate) fn build_frame(&self, bits: u8) -> Frame {
        Frame {
            session: (bits & DIRTY_SESSION != 0).then(|| self.session_frame()),
            chats: (bits & DIRTY_CHATS != 0).then(|| self.chats_frame()),
            conv: (bits & DIRTY_CONV != 0).then(|| self.conv_frame()),
            overlay: (bits & DIRTY_OVERLAY != 0).then(|| self.overlay_frame()),
        }
    }

    /// Chat title, with the self chat shown as "<name> (You)".
    pub(crate) fn chat_title(&self, s: &Store, jid: &str) -> String {
        let me = self.self_jid();
        let lid = lock(&self.view).status.lid.clone();
        if is_self(jid, me.as_deref()) || is_self(jid, lid.as_deref()) {
            let name = lock(&self.view).status.push_name.clone().unwrap_or_else(|| s.title_for(jid));
            return format!("{name} (You)");
        }
        s.title_for(jid)
    }

    /// Sender label for a message author (canonical jid).
    pub(crate) fn sender_name(&self, s: &Store, m: &UiMessage, chat: &str) -> String {
        if m.dto.from_me {
            return "You".into();
        }
        let sj = if m.dto.sender_jid.is_empty() { chat.to_string() } else { normalize_jid(&m.dto.sender_jid) };
        let t = s.title_for(&sj);
        if (t.starts_with('~') || t.starts_with('+')) && let Some(p) = m.dto.push_name.as_ref().filter(|p| !p.is_empty()) {
            return p.clone();
        }
        t
    }

    fn session_frame(&self) -> SessionFrame {
        let settings = lock(&self.settings).clone();
        let wallpaper_image = lock(&self.wallpaper).clone();
        let chats_known = !lock(&self.store).chats.is_empty();
        let v = lock(&self.view);
        let screen = if !v.booted {
            "loading"
        } else if !v.status.registered && !v.status.logged_in {
            "pairing"
        } else if v.hydrating {
            "loading"
        } else if v.status.connected || chats_known {
            "main"
        } else {
            "loading"
        };
        let (kind, color) = match settings.wallpaper.as_str() {
            "teal" => (1, 0x0b141a),
            "slate" => (1, 0x1f2c33),
            "dusk" => (2, 0),
            "custom" if wallpaper_image.is_some() => (3, 0),
            _ => (0, 0),
        };
        SessionFrame {
            screen: screen.into(),
            loading_label: v.loading_label.clone(),
            loading_done: v.loading_done,
            loading_total: v.loading_total,
            resetting: v.resetting,
            qr: v.qr.clone(),
            pair_code: v.pair_code.clone(),
            pair_error: v.pair_error.clone(),
            pair_busy: v.pair_busy,
            dead: v.dead,
            connected: v.status.connected,
            syncing: v.syncing_shown,
            self_name: v
                .status
                .push_name
                .clone()
                .filter(|n| !n.is_empty())
                .or_else(|| v.status.jid.as_deref().map(format_phone))
                .unwrap_or_else(|| "WhatsApp".into()),
            self_jid: v.status.jid.as_deref().map(format_phone).unwrap_or_default(),
            toasts: v.toasts.iter().map(|(t, _)| t.clone()).collect(),
            theme: settings.theme,
            zoom: settings.zoom,
            read_receipts: settings.send_read_receipts,
            auto_load_images: settings.auto_load_images,
            enter_to_send: settings.enter_to_send,
            wallpaper_choice: settings.wallpaper,
            wallpaper_kind: kind,
            wallpaper_color: color,
            wallpaper_image: if kind == 3 { wallpaper_image } else { None },
        }
    }

    fn chats_frame(&self) -> ChatsFrame {
        let (mut section, search) = {
            let v = lock(&self.view);
            (v.section, v.search.clone())
        };
        let avatars = lock(&self.avatars).clone();
        let s = lock(&self.store);
        let (archived, channels, status) = s.section_counts();
        if section == Section::Archived && archived == 0 {
            // The archive emptied while open: bounce back to All.
            section = Section::Chats;
            lock(&self.view).section = Section::Chats;
        }
        let active = s.active.clone();
        let rows = s
            .chat_list(section, &search)
            .into_iter()
            .map(|c| {
                let title = self.chat_title(&s, &c.jid);
                let draft = c.draft.as_deref().filter(|d| !d.trim().is_empty() && active.as_deref() != Some(&c.jid));
                let last = s.messages(&c.jid).last();
                let (prefix, tick, body) = match (draft, last) {
                    (Some(d), _) => (String::new(), 0, d.lines().next().unwrap_or("").to_string()),
                    (None, Some(m)) => {
                        let body = if m.deleted {
                            "🚫 This message was deleted".to_string()
                        } else {
                            m.dto.text.clone().filter(|t| !t.is_empty()).unwrap_or_else(|| media_label(&m.dto.kind).to_string())
                        };
                        let prefix = if is_group(&c.jid) {
                            if m.dto.from_me { "You:".to_string() } else { format!("{}:", self.sender_name(&s, m, &c.jid)) }
                        } else {
                            String::new()
                        };
                        let tick = if m.dto.from_me && !m.deleted { status_code(m.status) } else { 0 };
                        (prefix, tick, body.lines().next().unwrap_or("").to_string())
                    }
                    (None, None) => (String::new(), 0, c.last_message.as_deref().and_then(|t| t.lines().next()).unwrap_or("").to_string()),
                };
                ChatRowData {
                    jid: c.jid.clone(),
                    initials: avatar_initials(&c.jid, &title),
                    title,
                    preview: body,
                    prefix,
                    tick,
                    stamp: list_stamp(c.timestamp),
                    unread: c.unread as i32,
                    muted: c.is_muted(),
                    pinned: c.is_pinned(),
                    archived: c.is_archived(),
                    draft: draft.is_some(),
                    tint: tint_for(&c.jid),
                    avatar: avatars.get(&c.jid).cloned(),
                    active: active.as_deref() == Some(&c.jid),
                }
            })
            .collect();
        ChatsFrame {
            rows,
            section: match section {
                Section::Chats => 0,
                Section::Archived => 1,
                Section::Channels => 2,
                Section::Status => 3,
            },
            archived_count: archived as i32,
            channels_count: channels as i32,
            status_count: status as i32,
        }
    }

    /// Best image for a message: decoded full media, else its inline thumbnail.
    fn message_image(&self, m: &UiMessage) -> Option<Pixels> {
        let mut cache = lock(&self.images);
        if let Some(px) = cache.get(&format!("full:{}", m.dto.id)) {
            return Some(px);
        }
        let key = format!("thumb:{}", m.dto.id);
        if let Some(px) = cache.get(&key) {
            return Some(px);
        }
        let px = m.dto.thumbnail.as_deref().filter(|_| !m.deleted).and_then(images::decode_b64)?;
        cache.insert(key, px.clone());
        Some(px)
    }

    fn conv_frame(&self) -> ConvFrame {
        let (limit, reply_to, editing, composer, scroll_token, reveal, focus_token, suggestions, sugg_idx, flash) = {
            let mut v = lock(&self.view);
            (
                v.window_limit,
                v.reply_to.clone(),
                v.editing.clone(),
                v.composer_override.take(),
                v.scroll_token,
                v.reveal.clone().map(|(t, id, c)| (t, id, c)),
                v.focus_token,
                v.suggestions.clone(),
                v.suggestion_index,
                v.flash.as_ref().map(|(id, _)| id.clone()),
            )
        };
        let me_norm = self.self_jid().map(|j| normalize_jid(&j)).unwrap_or_default();
        let auto_load = lock(&self.settings).auto_load_images;
        let playing = self.audio.snapshot();
        let avatars = lock(&self.avatars).clone();
        let inflight = lock(&self.media_inflight).clone();
        let s = lock(&self.store);
        let Some(jid) = s.active.clone() else {
            return ConvFrame { composer, scroll_token, focus_token, ..Default::default() };
        };
        let chat = s.chats.get(&jid);
        let title = self.chat_title(&s, &jid);
        let group = is_group(&jid);
        let subtitle = if group {
            "Group".to_string()
        } else if is_newsletter(&jid) {
            "Channel".to_string()
        } else if is_status(&jid) {
            "Status updates".to_string()
        } else if is_phone(&jid) && format_phone(&jid) != title {
            format_phone(&jid)
        } else {
            String::new()
        };

        let all = s.messages(&jid);
        let start = all.len().saturating_sub(limit);
        let shown = &all[start..];
        let mut to_load: Vec<UiMessage> = Vec::new();
        let mut rows: Vec<MsgRowData> = Vec::with_capacity(shown.len());
        let mut albums: HashMap<String, Vec<String>> = HashMap::new();
        let mut prev_day: Option<i64> = None;
        let mut prev: Option<&UiMessage> = None;
        let mut divider_done = false;
        let mut unread_in_view = 0;
        let mut reveal_index: Option<i32> = None;

        let mut i = 0;
        while i < shown.len() {
            let n = album_len(shown, i);
            let group_items = &shown[i..i + n];
            let m = &group_items[n - 1]; // the row speaks for the last item (time, ticks)
            let first = &group_items[0];
            let d = &m.dto;
            i += n;

            let day = local_day(first.dto.timestamp);
            let day_text = if prev_day != Some(day) { day_label(first.dto.timestamp) } else { String::new() };
            prev_day = Some(day);
            let grouped = day_text.is_empty()
                && prev.is_some_and(|p| !p.deleted && p.dto.from_me == d.from_me && p.dto.sender_jid == d.sender_jid);
            prev = Some(m);

            let unread = group_items.iter().any(|x| x.unread);
            if unread {
                unread_in_view += 1;
            }
            let unread_divider = unread && !divider_done;
            divider_done |= unread;
            if reveal.as_ref().is_some_and(|(_, id, _)| group_items.iter().any(|x| &x.dto.id == id)) {
                reveal_index = Some(rows.len() as i32);
            }

            // ---- album
            let mut album = Vec::new();
            if n > 1 {
                albums.insert(d.id.clone(), group_items.iter().map(|x| x.dto.id.clone()).collect());
                let shown_tiles = if n > 4 { 4 } else { n };
                for (k, item) in group_items.iter().take(shown_tiles).enumerate() {
                    let wants = item.dto.kind == "image" && auto_load;
                    let has_full = lock(&self.images).get(&format!("full:{}", item.dto.id)).is_some();
                    if wants && !has_full && !inflight.contains(&item.dto.id) {
                        to_load.push(item.clone());
                    }
                    album.push(TileData {
                        id: item.dto.id.clone(),
                        image: self.message_image(item),
                        is_video: item.dto.kind == "video",
                        more: if n > 4 && k == 3 { (n - 3) as i32 } else { 0 },
                    });
                }
            }

            // ---- single image / sticker / video poster
            let sticker = d.kind == "sticker" && !m.deleted;
            let mut image = None;
            if n == 1 && matches!(d.kind.as_str(), "image" | "sticker" | "video") && !m.deleted {
                image = self.message_image(m);
                let wants_full = matches!(d.kind.as_str(), "image" | "sticker") && d.media.is_some();
                let has_full = lock(&self.images).get(&format!("full:{}", d.id)).is_some();
                if wants_full && auto_load && !has_full && !inflight.contains(&d.id) {
                    to_load.push(m.clone());
                }
            }
            let (image_w, image_h) = image
                .as_ref()
                .map(|p| (p.width() as i32, p.height() as i32))
                .or_else(|| d.media.as_ref().map(|md| (md.width.unwrap_or(0) as i32, md.height.unwrap_or(0) as i32)))
                .unwrap_or((0, 0));

            // ---- audio
            let is_audio = d.kind == "audio" && d.media.is_some() && !m.deleted;
            let (audio_playing, audio_progress, audio_label) = if is_audio {
                let secs = d.media.as_ref().and_then(|md| md.duration_secs).unwrap_or(0) as u64;
                if playing.key.as_deref() == Some(&d.id) {
                    let dur = playing.duration.map(|x| x.as_secs_f32()).unwrap_or(secs as f32).max(0.001);
                    let pos = playing.position.as_secs_f32();
                    (playing.playing, pos / dur, format!("{} / {}", fmt_duration(pos as u64), fmt_duration(dur as u64)))
                } else {
                    (false, 0.0, if secs > 0 { format!("Voice / audio {}", fmt_duration(secs)) } else { "Voice / audio".into() })
                }
            } else {
                (false, 0.0, String::new())
            };

            let media_label = match (&d.media, d.kind.as_str()) {
                (Some(md), "document") => {
                    let name = md.file_name.clone().unwrap_or_else(|| "Document".into());
                    let size = fmt_size(md.descriptor.file_length);
                    if size.is_empty() { format!("📄 {name}") } else { format!("📄 {name} · {size}") }
                }
                (Some(md), "video") => format!("🎥 Video{}", md.duration_secs.map(|x| format!(" · {}", fmt_duration(x as u64))).unwrap_or_default()),
                (Some(_), k) => media_label(k).to_string(),
                (None, _) => String::new(),
            };

            // ---- quote
            let (has_quote, quote_id, quote_sender, quote_text, quote_thumb) = match &d.quoted {
                Some(q) => {
                    let qs = match q.sender_jid.as_deref() {
                        Some(sj) if normalize_jid(&s.canonical(&normalize_jid(sj))) == me_norm || normalize_jid(sj) == me_norm => "You".to_string(),
                        Some(sj) => s.title_for(&s.canonical(&normalize_jid(sj))),
                        None if d.from_me => title.clone(),
                        None => "You".to_string(),
                    };
                    let target = s.find_message(&jid, &q.id);
                    let qt = quoted_text(&q.kind, q.text.as_deref(), target.and_then(|t| t.dto.media.as_ref()).and_then(|md| md.file_name.as_deref()));
                    let mut thumb = None;
                    if let Some(b64) = q.thumbnail.as_deref() {
                        let key = format!("qthumb:{}", d.id);
                        let mut cache = lock(&self.images);
                        thumb = cache.get(&key).or_else(|| {
                            let px = images::decode_b64(b64)?;
                            cache.insert(key, px.clone());
                            Some(px)
                        });
                    }
                    if thumb.is_none()
                        && let Some(t) = target.filter(|t| matches!(t.dto.kind.as_str(), "image" | "sticker" | "video"))
                    {
                        thumb = self.message_image(t);
                    }
                    (true, q.id.clone(), qs, qt, thumb)
                }
                None => (false, String::new(), String::new(), String::new(), None),
            };

            // ---- reactions
            let mut counts: Vec<(String, usize)> = Vec::new();
            for e in m.reactions.values() {
                match counts.iter_mut().find(|(x, _)| x == e) {
                    Some((_, c)) => *c += 1,
                    None => counts.push((e.clone(), 1)),
                }
            }
            let reactions = counts
                .iter()
                .map(|(e, c)| if *c > 1 { format!("{e} {c}") } else { e.clone() })
                .collect::<Vec<_>>()
                .join("  ");

            let caption = if n > 1 {
                group_items.iter().find_map(|x| x.dto.text.clone().filter(|t| !t.trim().is_empty())).unwrap_or_default()
            } else {
                d.text.clone().unwrap_or_default()
            };
            let structured = structured_label(&d.kind);
            let text = if m.deleted {
                String::new()
            } else if caption.is_empty() && d.media.is_none() && d.thumbnail.is_none() && structured.is_empty() {
                format!("[{}]", d.kind)
            } else {
                caption
            };
            let pending = m.is_pending();
            rows.push(MsgRowData {
                id: d.id.clone(),
                from_me: d.from_me,
                show_sender: group && !d.from_me && !grouped,
                grouped,
                tint: tint_for(&d.sender_jid),
                sender: self.sender_name(&s, m, &jid),
                rich_md: if m.deleted { None } else { linkify_markdown(&text) },
                text,
                kind: d.kind.clone(),
                structured: structured.to_string(),
                time: clock(d.timestamp),
                status: status_code(m.status),
                edited: m.edited_at.is_some(),
                deleted: m.deleted,
                reactions,
                my_reaction: m.reactions.get(&me_norm).cloned().unwrap_or_default(),
                has_quote,
                quote_id,
                quote_sender,
                quote_text,
                quote_thumb,
                image_w,
                image_h,
                image,
                sticker,
                has_media: d.media.is_some() && !m.deleted,
                media_label,
                loading_media: group_items.iter().any(|x| inflight.contains(&x.dto.id)),
                is_audio,
                audio_playing,
                audio_progress,
                audio_label,
                album,
                day: day_text,
                unread,
                unread_divider,
                flash: flash.as_deref().is_some_and(|f| group_items.iter().any(|x| x.dto.id == f)),
                pending,
                can_reply: !m.deleted && !pending,
                can_edit: d.from_me && !m.deleted && !pending && d.media.is_none() && d.text.as_deref().is_some_and(|t| !t.is_empty()),
                can_delete: !m.deleted && (!pending || m.status == MessageStatus::Failed),
            });
        }

        let quote_target = reply_to.as_deref().or(editing.as_deref()).and_then(|id| s.find_message(&jid, id));
        let (reply_sender, reply_text, reply_thumb) = quote_target
            .map(|q| {
                let who = self.sender_name(&s, q, &jid);
                let text = quoted_text(&q.dto.kind, q.dto.text.as_deref(), q.dto.media.as_ref().and_then(|md| md.file_name.as_deref()));
                let thumb = matches!(q.dto.kind.as_str(), "image" | "sticker" | "video").then(|| self.message_image(q)).flatten();
                (who, text, thumb)
            })
            .unwrap_or_default();

        let frame = ConvFrame {
            active_jid: jid.clone(),
            initials: avatar_initials(&jid, &title),
            title,
            subtitle,
            tint: tint_for(&jid),
            avatar: avatars.get(&jid).cloned(),
            muted: chat.is_some_and(|c| c.is_muted()),
            pinned: chat.is_some_and(|c| c.is_pinned()),
            archived: chat.is_some_and(|c| c.is_archived()),
            rows,
            has_older: all.len() > limit,
            unread_in_view,
            replying: reply_to.is_some(),
            editing: editing.is_some(),
            reply_sender,
            reply_text,
            reply_thumb,
            composer,
            scroll_token,
            reveal: reveal.and_then(|(t, id, c)| reveal_index.map(|ix| (t, id, ix, c))),
            focus_token,
            suggestions,
            suggestion_index: sugg_idx,
        };
        drop(s);
        lock(&self.view).albums = albums;
        for m in to_load {
            self.spawn_image_load(m);
        }
        frame
    }

    fn overlay_frame(&self) -> OverlayFrame {
        let settings = lock(&self.settings).clone();
        let preview_playing = {
            let p = self.audio.snapshot();
            p.key.as_deref() == Some("preview") && p.playing
        };
        let album_open = lock(&self.view).open_album.take();
        let album = album_open.map(|row| {
            let ids = lock(&self.view).albums.get(&row).cloned().unwrap_or_default();
            let s = lock(&self.store);
            let jid = s.active.clone().unwrap_or_default();
            let items: Vec<UiMessage> = ids.iter().filter_map(|id| s.find_message(&jid, id).cloned()).collect();
            drop(s);
            let any_video = items.iter().any(|m| m.dto.kind == "video");
            let title = if any_video { format!("{} items", items.len()) } else { format!("{} photos", items.len()) };
            let tiles = items
                .iter()
                .map(|m| TileData { id: m.dto.id.clone(), image: self.message_image(m), is_video: m.dto.kind == "video", more: 0 })
                .collect();
            (title, tiles)
        });

        let mut v = lock(&self.view);
        // emoji tabs: recents (when any) + categories
        let mut tabs: Vec<(String, String, String)> = Vec::new();
        if !settings.emoji_recent.is_empty() {
            tabs.push(("recent".into(), "🕘".into(), "Recent".into()));
        }
        for c in emoji::CATEGORIES {
            tabs.push((c.key.into(), c.icon.into(), c.label.into()));
        }
        let tab = (v.emoji_tab.max(0) as usize).min(tabs.len() - 1);
        let (grid, empty): (Vec<String>, &str) = if !v.emoji_search.trim().is_empty() {
            (emoji::search(v.emoji_search.trim(), 64).into_iter().map(|(_, e)| e.to_string()).collect(), "No emoji found")
        } else if tabs[tab].0 == "recent" {
            (settings.emoji_recent.clone(), "No emoji yet")
        } else {
            let c = emoji::CATEGORIES.iter().find(|c| c.key == tabs[tab].0).unwrap();
            (c.emojis.iter().map(|e| e.to_string()).collect(), "No emoji yet")
        };
        let gifs = v
            .gif
            .items
            .iter()
            .map(|g| (g.id.clone(), v.gif.previews.get(&g.id).cloned()))
            .collect();
        let gif_status = if v.gif.loading && v.gif.items.is_empty() {
            "Loading…".to_string()
        } else if !v.gif.status.is_empty() {
            v.gif.status.clone()
        } else if !v.gif.loading && v.gif.items.is_empty() && v.gif.page > 0 {
            "No GIFs found".to_string()
        } else {
            String::new()
        };
        let preview = v.preview.as_ref().map(|p| PreviewData {
            title: match p.kind.as_str() {
                "image" => "Send photo",
                "video" => "Send video",
                "audio" => "Send voice message",
                _ => "Send document",
            }
            .to_string(),
            kind: p.kind.clone(),
            image: p.image.clone(),
            name: match (p.kind.as_str(), p.duration) {
                ("audio", Some(d)) => format!("Voice message · {}", fmt_duration(d as u64)),
                _ => p.name.clone(),
            },
            hd_available: p.kind == "image" && !crate::media::prep::is_gif(&p.path),
            playing: preview_playing,
        });
        let rec = v.rec.as_ref().map(|r| match r {
            Rec::Voice(rec) => (
                "voice".to_string(),
                rec.elapsed_label(),
                None,
                rec.error.lock().unwrap_or_else(|p| p.into_inner()).clone().unwrap_or_default(),
            ),
            Rec::Photo { frame, error, .. } => ("photo".to_string(), String::new(), frame.clone(), error.clone().unwrap_or_default()),
        });
        let viewer = v.viewer.as_ref().map(|x| ViewerData {
            id: x.ids.get(x.index).cloned().unwrap_or_default(),
            image: x.image.clone(),
            is_video: x.is_video,
            loading: x.loading,
            error: x.error.clone(),
            index: x.index as i32,
            count: x.ids.len() as i32,
            save_label: x.save_label.clone(),
        });
        OverlayFrame {
            picker_tab: v.picker_tab,
            emoji_tabs: tabs,
            emoji_tab: tab as i32,
            emoji_grid: grid,
            emoji_empty: empty.into(),
            gif_configured: crate::media::klipy::api_key().is_some(),
            gifs,
            gif_status,
            open_delete: v.open_delete.take(),
            open_reaction: v.open_reaction.take(),
            open_album: album,
            preview,
            rec,
            viewer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_found_and_trimmed() {
        assert!(find_links("café 🚀 https://x.io ok").len() == 1);
        let t = "see https://a.com/x). and www.b.org, not www.c";
        let found: Vec<&str> = find_links(t).into_iter().map(|(s, e)| &t[s..e]).collect();
        assert_eq!(found, vec!["https://a.com/x", "www.b.org"]);
        let md = linkify_markdown("go to www.x.io now").unwrap();
        assert!(md.contains("](<https://www.x.io>)"), "{md}");
        assert!(linkify_markdown("no links here").is_none());
    }

    fn media_msg(id: &str, ts: i64, parent: Option<&str>) -> UiMessage {
        UiMessage {
            dto: MessageDto {
                id: id.into(),
                kind: "image".into(),
                timestamp: ts,
                sender_jid: "a".into(),
                album_parent_id: parent.map(Into::into),
                media: Some(Default::default()),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn album_rules() {
        let explicit = vec![media_msg("1", 0, Some("p")), media_msg("2", 1, Some("p")), media_msg("3", 500, None)];
        assert_eq!(album_len(&explicit, 0), 2);
        let inferred: Vec<UiMessage> = (0..5).map(|i| media_msg(&i.to_string(), i * 30, None)).collect();
        assert_eq!(album_len(&inferred, 0), 5);
        let short: Vec<UiMessage> = (0..3).map(|i| media_msg(&i.to_string(), i, None)).collect();
        assert_eq!(album_len(&short, 0), 1);
        let gappy = vec![media_msg("a", 0, None), media_msg("b", 10, None), media_msg("c", 100, None), media_msg("d", 110, None)];
        assert_eq!(album_len(&gappy, 0), 1);
    }
}
