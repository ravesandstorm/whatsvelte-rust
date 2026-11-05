//! View frames: `Send` snapshots built on worker threads and copied into the
//! Slint models on the UI thread. Models persist across frames and are patched
//! row by row; decoded pixel buffers map to cached `slint::Image`s so an
//! unchanged row compares equal and isn't re-rendered.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use slint::{Color, ComponentHandle, Image, Model, ModelRc, SharedString, StyledText, VecModel};

use crate::ui::images::Pixels;
use crate::{AlbumTile, AppState, AppWindow, ChatRow, EmojiTab, GifItem, MsgRow, Suggestion};

pub struct SessionFrame {
    pub screen: String,
    pub loading_label: String,
    pub loading_done: i32,
    pub loading_total: i32,
    pub resetting: bool,
    pub qr: Option<Pixels>,
    pub pair_code: String,
    pub pair_error: String,
    pub pair_busy: bool,
    pub dead: bool,
    pub connected: bool,
    pub syncing: bool,
    pub self_name: String,
    pub self_jid: String,
    pub toasts: Vec<String>,
    pub theme: String,
    pub zoom: f32,
    pub read_receipts: bool,
    pub auto_load_images: bool,
    pub enter_to_send: bool,
    pub wallpaper_choice: String,
    pub wallpaper_kind: i32,
    pub wallpaper_color: u32,
    pub wallpaper_image: Option<Pixels>,
}

pub struct ChatRowData {
    pub jid: String,
    pub title: String,
    pub preview: String,
    pub prefix: String,
    pub tick: i32,
    pub stamp: String,
    pub unread: i32,
    pub muted: bool,
    pub pinned: bool,
    pub archived: bool,
    pub draft: bool,
    pub initials: String,
    pub tint: u32,
    pub avatar: Option<Pixels>,
    pub active: bool,
}

pub struct ChatsFrame {
    pub rows: Vec<ChatRowData>,
    pub section: i32,
    pub archived_count: i32,
    pub channels_count: i32,
    pub status_count: i32,
}

pub struct TileData {
    pub id: String,
    pub image: Option<Pixels>,
    pub is_video: bool,
    pub more: i32,
}

#[derive(Default)]
pub struct MsgRowData {
    pub id: String,
    pub from_me: bool,
    pub show_sender: bool,
    pub grouped: bool,
    pub sender: String,
    pub tint: u32,
    pub text: String,
    /// Markdown with links, when the text contains any.
    pub rich_md: Option<String>,
    pub kind: String,
    pub structured: String,
    pub time: String,
    pub status: i32,
    pub edited: bool,
    pub deleted: bool,
    pub reactions: String,
    pub my_reaction: String,
    pub has_quote: bool,
    pub quote_id: String,
    pub quote_sender: String,
    pub quote_text: String,
    pub quote_thumb: Option<Pixels>,
    pub image: Option<Pixels>,
    pub image_w: i32,
    pub image_h: i32,
    pub sticker: bool,
    pub has_media: bool,
    pub media_label: String,
    pub loading_media: bool,
    pub is_audio: bool,
    pub audio_playing: bool,
    pub audio_progress: f32,
    pub audio_label: String,
    pub album: Vec<TileData>,
    pub day: String,
    pub unread: bool,
    pub unread_divider: bool,
    pub flash: bool,
    pub pending: bool,
    pub can_reply: bool,
    pub can_edit: bool,
    pub can_delete: bool,
}

#[derive(Default)]
pub struct ConvFrame {
    pub active_jid: String,
    pub title: String,
    pub subtitle: String,
    pub initials: String,
    pub tint: u32,
    pub avatar: Option<Pixels>,
    pub muted: bool,
    pub pinned: bool,
    pub archived: bool,
    pub rows: Vec<MsgRowData>,
    pub has_older: bool,
    pub unread_in_view: i32,
    pub replying: bool,
    pub editing: bool,
    pub reply_sender: String,
    pub reply_text: String,
    pub reply_thumb: Option<Pixels>,
    /// Set the composer text (and optionally the caret byte offset).
    pub composer: Option<(String, Option<i32>)>,
    pub scroll_token: i32,
    /// (token, id, index, center)
    pub reveal: Option<(i32, String, i32, bool)>,
    pub focus_token: i32,
    pub suggestions: Vec<(String, String)>,
    pub suggestion_index: i32,
}

pub struct PreviewData {
    pub title: String,
    pub kind: String,
    pub image: Option<Pixels>,
    pub name: String,
    pub hd_available: bool,
    pub playing: bool,
}

pub struct ViewerData {
    pub id: String,
    pub image: Option<Pixels>,
    pub is_video: bool,
    pub loading: bool,
    pub error: String,
    pub index: i32,
    pub count: i32,
    pub save_label: String,
}

#[derive(Default)]
pub struct OverlayFrame {
    pub picker_tab: i32,
    pub emoji_tabs: Vec<(String, String, String)>,
    pub emoji_tab: i32,
    pub emoji_grid: Vec<String>,
    pub emoji_empty: String,
    pub gif_configured: bool,
    pub gifs: Vec<(String, Option<Pixels>)>,
    pub gif_status: String,
    /// One-shot opens (Slint closes these itself).
    pub open_delete: Option<(String, bool)>,
    pub open_reaction: Option<(String, String)>,
    pub open_album: Option<(String, Vec<TileData>)>,
    pub open_settings: bool,
    /// Rust-owned overlays: None = closed.
    pub preview: Option<PreviewData>,
    pub rec: Option<(String, String, Option<Pixels>, String)>,
    pub viewer: Option<ViewerData>,
}

pub struct Frame {
    pub session: Option<SessionFrame>,
    pub chats: Option<ChatsFrame>,
    pub conv: Option<ConvFrame>,
    pub overlay: Option<OverlayFrame>,
}

fn rgb(c: u32) -> Color {
    Color::from_rgb_u8((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

fn ss(s: String) -> SharedString {
    SharedString::from(s)
}

struct Models {
    chats: Rc<VecModel<ChatRow>>,
    messages: Rc<VecModel<MsgRow>>,
    active_jid: RefCell<String>,
    /// Pixel buffer identity -> Image, so re-sent buffers compare equal.
    images: RefCell<HashMap<(usize, u32, u32), Image>>,
}

thread_local! {
    static MODELS: RefCell<Option<Rc<Models>>> = const { RefCell::new(None) };
}

fn models(app: &AppState<'_>) -> Rc<Models> {
    MODELS.with(|m| {
        m.borrow_mut()
            .get_or_insert_with(|| {
                let models = Rc::new(Models {
                    chats: Rc::new(VecModel::default()),
                    messages: Rc::new(VecModel::default()),
                    active_jid: RefCell::new(String::new()),
                    images: RefCell::new(HashMap::new()),
                });
                app.set_chats(ModelRc::from(models.chats.clone()));
                app.set_messages(ModelRc::from(models.messages.clone()));
                models
            })
            .clone()
    })
}

impl Models {
    fn image(&self, px: Option<Pixels>) -> (Image, bool) {
        let Some(px) = px else {
            return (Image::default(), false);
        };
        let key = (px.as_bytes().as_ptr() as usize, px.width(), px.height());
        let mut cache = self.images.borrow_mut();
        if cache.len() > 1500 {
            cache.clear();
        }
        let img = cache
            .entry(key)
            .or_insert_with(|| Image::from_rgba8(px))
            .clone();
        (img, true)
    }
}

/// Patch `model` to equal `rows`: update changed rows in place, append/trim
/// the tail; fall back to a full reset only when `same` says the leading rows
/// diverge. In-place updates keep the ListView's scroll position.
fn sync_model<T: Clone + PartialEq + 'static>(
    model: &VecModel<T>,
    rows: Vec<T>,
    same: impl Fn(&T, &T) -> bool,
) {
    let old_len = model.row_count();
    let common = old_len.min(rows.len());
    let prefix_ok = (0..common).all(|i| model.row_data(i).is_some_and(|o| same(&o, &rows[i])));
    if !prefix_ok {
        model.set_vec(rows);
        return;
    }
    for (i, row) in rows.iter().enumerate().take(old_len) {
        if model.row_data(i).as_ref() != Some(row) {
            model.set_row_data(i, row.clone());
        }
    }
    if rows.len() > old_len {
        for row in rows.into_iter().skip(old_len) {
            model.push(row);
        }
    } else {
        while model.row_count() > rows.len() {
            model.remove(model.row_count() - 1);
        }
    }
}

fn tiles(m: &Models, tiles: Vec<TileData>) -> ModelRc<AlbumTile> {
    let v: Vec<AlbumTile> = tiles
        .into_iter()
        .map(|t| {
            let (image, has_image) = m.image(t.image);
            AlbumTile {
                id: ss(t.id),
                image,
                has_image,
                is_video: t.is_video,
                more: t.more,
            }
        })
        .collect();
    ModelRc::new(VecModel::from(v))
}

impl Frame {
    pub fn apply(self, w: &AppWindow) {
        let app = w.global::<AppState>();
        let m = models(&app);

        if let Some(s) = self.session {
            app.set_screen(ss(s.screen));
            app.set_loading_label(ss(s.loading_label));
            app.set_loading_done(s.loading_done);
            app.set_loading_total(s.loading_total);
            app.set_resetting(s.resetting);
            app.set_has_qr(s.qr.is_some());
            if s.qr.is_some() {
                app.set_qr(m.image(s.qr).0);
            }
            app.set_pair_code(ss(s.pair_code));
            app.set_pair_error(ss(s.pair_error));
            app.set_pair_busy(s.pair_busy);
            app.set_client_dead(s.dead);
            app.set_connected(s.connected);
            app.set_syncing(s.syncing);
            app.set_self_name(ss(s.self_name));
            app.set_self_jid(ss(s.self_jid));
            let toasts: Vec<SharedString> = s.toasts.into_iter().map(ss).collect();
            app.set_toasts(ModelRc::new(VecModel::from(toasts)));
            app.set_theme_choice(ss(s.theme));
            app.set_zoom(s.zoom);
            app.set_read_receipts(s.read_receipts);
            app.set_auto_load_images(s.auto_load_images);
            app.set_enter_to_send(s.enter_to_send);
            app.set_wallpaper_choice(ss(s.wallpaper_choice));
            app.set_wallpaper_kind(s.wallpaper_kind);
            app.set_wallpaper_color(rgb(s.wallpaper_color));
            app.set_wallpaper_image(m.image(s.wallpaper_image).0);
        }

        if let Some(c) = self.chats {
            let rows: Vec<ChatRow> = c
                .rows
                .into_iter()
                .map(|r| {
                    let (avatar, has_avatar) = m.image(r.avatar);
                    ChatRow {
                        jid: ss(r.jid),
                        title: ss(r.title),
                        preview: ss(r.preview),
                        prefix: ss(r.prefix),
                        tick: r.tick,
                        stamp: ss(r.stamp),
                        unread: r.unread,
                        muted: r.muted,
                        pinned: r.pinned,
                        archived: r.archived,
                        draft: r.draft,
                        initials: ss(r.initials),
                        tint: rgb(r.tint),
                        avatar,
                        has_avatar,
                        active: r.active,
                    }
                })
                .collect();
            // Reorders (a new message bumps a chat) patch rows in place rather
            // than resetting the model, so the list keeps its scroll position.
            sync_model(&m.chats, rows, |_, _| true);
            app.set_section(c.section);
            app.set_archived_count(c.archived_count);
            app.set_channels_count(c.channels_count);
            app.set_status_count(c.status_count);
        }

        if let Some(c) = self.conv {
            let switched = *m.active_jid.borrow() != c.active_jid;
            *m.active_jid.borrow_mut() = c.active_jid.clone();
            let rows: Vec<MsgRow> = c
                .rows
                .into_iter()
                .map(|r| {
                    let (image, has_image) = m.image(r.image);
                    let (quote_thumb, has_quote_thumb) = m.image(r.quote_thumb);
                    let rich = r
                        .rich_md
                        .as_deref()
                        .and_then(|md| StyledText::from_markdown(md).ok());
                    let is_album = !r.album.is_empty();
                    MsgRow {
                        id: ss(r.id),
                        from_me: r.from_me,
                        show_sender: r.show_sender,
                        grouped: r.grouped,
                        sender: ss(r.sender),
                        tint: rgb(r.tint),
                        text: ss(r.text),
                        has_links: rich.is_some(),
                        rich: rich.unwrap_or_default(),
                        kind: ss(r.kind),
                        structured: ss(r.structured),
                        time: ss(r.time),
                        status: r.status,
                        edited: r.edited,
                        deleted: r.deleted,
                        reactions: ss(r.reactions),
                        my_reaction: ss(r.my_reaction),
                        has_quote: r.has_quote,
                        quote_id: ss(r.quote_id),
                        quote_sender: ss(r.quote_sender),
                        quote_text: ss(r.quote_text),
                        quote_thumb,
                        has_quote_thumb,
                        has_image,
                        image,
                        image_w: r.image_w,
                        image_h: r.image_h,
                        sticker: r.sticker,
                        has_media: r.has_media,
                        media_label: ss(r.media_label),
                        loading_media: r.loading_media,
                        is_audio: r.is_audio,
                        audio_playing: r.audio_playing,
                        audio_progress: r.audio_progress,
                        audio_label: ss(r.audio_label),
                        is_album,
                        album: if is_album {
                            tiles(&m, r.album)
                        } else {
                            ModelRc::default()
                        },
                        day: ss(r.day),
                        unread: r.unread,
                        unread_divider: r.unread_divider,
                        flash: r.flash,
                        pending: r.pending,
                        can_reply: r.can_reply,
                        can_edit: r.can_edit,
                        can_delete: r.can_delete,
                    }
                })
                .collect();
            if switched {
                m.messages.set_vec(rows);
            } else {
                // Same chat: patch in place (older pages prepend rows; the view
                // restores its position from the height delta).
                sync_model(&m.messages, rows, |_, _| true);
            }
            app.set_active_jid(ss(c.active_jid));
            app.set_active_title(ss(c.title));
            app.set_active_subtitle(ss(c.subtitle));
            app.set_active_initials(ss(c.initials));
            app.set_active_tint(rgb(c.tint));
            let (avatar, has_avatar) = m.image(c.avatar);
            app.set_active_avatar(avatar);
            app.set_active_has_avatar(has_avatar);
            app.set_active_muted(c.muted);
            app.set_active_pinned(c.pinned);
            app.set_active_archived(c.archived);
            app.set_has_older(c.has_older);
            app.set_unread_in_view(c.unread_in_view);
            app.set_replying(c.replying);
            app.set_editing(c.editing);
            app.set_reply_sender(ss(c.reply_sender));
            app.set_reply_text(ss(c.reply_text));
            let (thumb, has_thumb) = m.image(c.reply_thumb);
            app.set_reply_thumb(thumb);
            app.set_has_reply_thumb(has_thumb);
            let sugg: Vec<Suggestion> = c
                .suggestions
                .into_iter()
                .map(|(emoji, code)| Suggestion {
                    emoji: ss(emoji),
                    code: ss(code),
                })
                .collect();
            app.set_suggestions(ModelRc::new(VecModel::from(sugg)));
            app.set_suggestion_index(c.suggestion_index);
            if let Some((text, caret)) = c.composer {
                app.set_composer_text(ss(text));
                if let Some(caret) = caret {
                    app.set_composer_caret(caret);
                    app.set_caret_token(app.get_caret_token() + 1);
                }
            }
            if app.get_scroll_token() != c.scroll_token {
                app.set_scroll_token(c.scroll_token);
            }
            if let Some((token, id, index, center)) = c.reveal
                && app.get_reveal_token() != token
            {
                app.set_reveal_id(ss(id));
                app.set_reveal_index(index);
                app.set_reveal_center(center);
                app.set_reveal_token(token);
            }
            if app.get_focus_token() != c.focus_token {
                app.set_focus_token(c.focus_token);
            }
        }

        if let Some(o) = self.overlay {
            app.set_picker_tab(o.picker_tab);
            let tabs: Vec<EmojiTab> = o
                .emoji_tabs
                .into_iter()
                .map(|(key, icon, label)| EmojiTab {
                    key: ss(key),
                    icon: ss(icon),
                    label: ss(label),
                })
                .collect();
            app.set_emoji_tabs(ModelRc::new(VecModel::from(tabs)));
            app.set_emoji_tab(o.emoji_tab);
            let grid: Vec<SharedString> = o.emoji_grid.into_iter().map(ss).collect();
            app.set_emoji_grid(ModelRc::new(VecModel::from(grid)));
            app.set_emoji_empty(ss(o.emoji_empty));
            app.set_gif_configured(o.gif_configured);
            let gifs: Vec<GifItem> = o
                .gifs
                .into_iter()
                .map(|(id, px)| {
                    let (image, has_image) = m.image(px);
                    GifItem {
                        id: ss(id),
                        image,
                        has_image,
                    }
                })
                .collect();
            app.set_gifs(ModelRc::new(VecModel::from(gifs)));
            app.set_gif_status(ss(o.gif_status));

            if let Some((id, everyone)) = o.open_delete {
                app.set_delete_id(ss(id));
                app.set_delete_can_everyone(everyone);
                app.set_delete_open(true);
            }
            if let Some((id, current)) = o.open_reaction {
                app.set_reaction_id(ss(id));
                app.set_reaction_current(ss(current));
                app.set_reaction_open(true);
            }
            if o.open_settings {
                app.set_settings_open(true);
            }
            if let Some((title, t)) = o.open_album {
                app.set_album_title(ss(title));
                app.set_album_tiles(tiles(&m, t));
                app.set_album_open(true);
            }

            match o.preview {
                Some(p) => {
                    if !app.get_preview_open() {
                        app.set_preview_caption(SharedString::default());
                        app.set_preview_hd(false);
                    }
                    app.set_preview_title(ss(p.title));
                    app.set_preview_kind(ss(p.kind));
                    let (img, has) = m.image(p.image);
                    app.set_preview_image(img);
                    app.set_preview_has_image(has);
                    app.set_preview_name(ss(p.name));
                    app.set_preview_hd_available(p.hd_available);
                    app.set_preview_playing(p.playing);
                    app.set_preview_open(true);
                }
                None => app.set_preview_open(false),
            }
            match o.rec {
                Some((mode, elapsed, frame, error)) => {
                    app.set_rec_mode(ss(mode));
                    app.set_rec_elapsed(ss(elapsed));
                    // Camera frames are fresh buffers each time; skip the cache.
                    app.set_rec_has_frame(frame.is_some());
                    if let Some(px) = frame {
                        app.set_rec_frame(Image::from_rgba8(px));
                    }
                    app.set_rec_error(ss(error));
                    app.set_rec_open(true);
                }
                None => {
                    app.set_rec_open(false);
                    app.set_rec_frame(Image::default());
                }
            }
            match o.viewer {
                Some(v) => {
                    app.set_viewer_id(ss(v.id));
                    let (img, has) = m.image(v.image);
                    app.set_viewer_image(img);
                    app.set_viewer_has_image(has);
                    app.set_viewer_is_video(v.is_video);
                    app.set_viewer_loading(v.loading);
                    app.set_viewer_error(ss(v.error));
                    app.set_viewer_index(v.index);
                    app.set_viewer_count(v.count);
                    app.set_viewer_save_label(ss(v.save_label));
                    app.set_viewer_open(true);
                }
                None => {
                    app.set_viewer_open(false);
                    app.set_viewer_image(Image::default());
                }
            }
        }
    }
}
