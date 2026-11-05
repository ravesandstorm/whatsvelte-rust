//! View frames: `Send` snapshots built on worker threads and copied into the
//! Slint models on the UI thread. Models are kept across frames and patched
//! row-by-row when possible, so a new message doesn't rebuild the whole list.

use std::cell::RefCell;
use std::rc::Rc;

use slint::{Color, ComponentHandle, Image, Model, ModelRc, SharedString, VecModel};

use crate::ui::images::Pixels;
use crate::{AppState, AppWindow, ChatRow, MsgRow};

pub struct SessionFrame {
    pub screen: String,
    pub loading_label: String,
    pub loading_progress: f32,
    pub qr: Option<Pixels>,
    pub pair_code: String,
    pub pair_error: String,
    pub pair_busy: bool,
    pub dead: bool,
    pub connected: bool,
    pub syncing: bool,
    pub self_name: String,
    pub self_jid: String,
    pub toast: String,
    pub theme: String,
    pub zoom: f32,
    pub read_receipts: bool,
    pub auto_load_images: bool,
    /// `Some(Some(..))` opens the viewer, `Some(None)` closes it.
    pub viewer: Option<Option<(String, Pixels)>>,
}

pub struct ChatRowData {
    pub jid: String,
    pub title: String,
    pub preview: String,
    pub stamp: String,
    pub unread: i32,
    pub muted: bool,
    pub pinned: bool,
    pub archived: bool,
    pub draft: bool,
    pub initials: String,
    pub tint: u32,
    pub active: bool,
}

pub struct ChatsFrame {
    pub rows: Vec<ChatRowData>,
    pub section: i32,
    pub archived_count: i32,
    pub channels_count: i32,
    pub status_count: i32,
}

pub struct MsgRowData {
    pub id: String,
    pub from_me: bool,
    pub show_sender: bool,
    pub sender: String,
    pub tint: u32,
    pub text: String,
    pub kind: String,
    pub time: String,
    pub status: i32,
    pub edited: bool,
    pub deleted: bool,
    pub reactions: String,
    pub has_quote: bool,
    pub quote_sender: String,
    pub quote_text: String,
    pub image: Option<Pixels>,
    pub image_w: i32,
    pub image_h: i32,
    pub has_media: bool,
    pub media_label: String,
    pub loading_media: bool,
    pub day: String,
    pub pending: bool,
    pub can_edit: bool,
    pub can_revoke: bool,
}

#[derive(Default)]
pub struct ConvFrame {
    pub active_jid: String,
    pub title: String,
    pub subtitle: String,
    pub initials: String,
    pub tint: u32,
    pub muted: bool,
    pub pinned: bool,
    pub archived: bool,
    pub rows: Vec<MsgRowData>,
    pub has_older: bool,
    pub replying: bool,
    pub editing: bool,
    pub reply_sender: String,
    pub reply_text: String,
    /// Set the composer text (chat switch, after send, start edit).
    pub composer: Option<String>,
    pub scroll_token: i32,
    pub append_token: i32,
}

pub struct Frame {
    pub session: Option<SessionFrame>,
    pub chats: Option<ChatsFrame>,
    pub conv: Option<ConvFrame>,
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
                });
                app.set_chats(ModelRc::from(models.chats.clone()));
                app.set_messages(ModelRc::from(models.messages.clone()));
                models
            })
            .clone()
    })
}

/// Patch `model` to equal `rows`: update changed rows in place, append/trim
/// the tail; fall back to a full reset when the leading ids diverge.
fn sync_model<T: Clone + PartialEq + 'static>(model: &VecModel<T>, rows: Vec<T>, same: impl Fn(&T, &T) -> bool) {
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

impl Frame {
    pub fn apply(self, w: &AppWindow) {
        let app = w.global::<AppState>();
        let m = models(&app);

        if let Some(s) = self.session {
            app.set_screen(ss(s.screen));
            app.set_loading_label(ss(s.loading_label));
            app.set_loading_progress(s.loading_progress);
            app.set_has_qr(s.qr.is_some());
            if let Some(px) = s.qr {
                app.set_qr(Image::from_rgba8(px));
            }
            app.set_pair_code(ss(s.pair_code));
            app.set_pair_error(ss(s.pair_error));
            app.set_pair_busy(s.pair_busy);
            app.set_client_dead(s.dead);
            app.set_connected(s.connected);
            app.set_syncing(s.syncing);
            app.set_self_name(ss(s.self_name));
            app.set_self_jid(ss(s.self_jid));
            app.set_toast(ss(s.toast));
            app.set_theme_choice(ss(s.theme));
            app.set_zoom(s.zoom);
            app.set_read_receipts(s.read_receipts);
            app.set_auto_load_images(s.auto_load_images);
            match s.viewer {
                Some(Some((id, px))) => {
                    app.set_viewer_id(ss(id));
                    app.set_viewer_image(Image::from_rgba8(px));
                    app.set_viewer_open(true);
                }
                Some(None) => {
                    app.set_viewer_open(false);
                    app.set_viewer_image(Image::default());
                }
                None => {}
            }
        }

        if let Some(c) = self.chats {
            let rows: Vec<ChatRow> = c
                .rows
                .into_iter()
                .map(|r| ChatRow {
                    jid: ss(r.jid),
                    title: ss(r.title),
                    preview: ss(r.preview),
                    stamp: ss(r.stamp),
                    unread: r.unread,
                    muted: r.muted,
                    pinned: r.pinned,
                    archived: r.archived,
                    draft: r.draft,
                    initials: ss(r.initials),
                    tint: rgb(r.tint),
                    active: r.active,
                })
                .collect();
            // The chat list reorders constantly; positional patching only helps
            // when the order is unchanged.
            sync_model(&m.chats, rows, |a, b| a.jid == b.jid);
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
                .map(|r| MsgRow {
                    id: ss(r.id),
                    from_me: r.from_me,
                    show_sender: r.show_sender,
                    sender: ss(r.sender),
                    tint: rgb(r.tint),
                    text: ss(r.text),
                    kind: ss(r.kind),
                    time: ss(r.time),
                    status: r.status,
                    edited: r.edited,
                    deleted: r.deleted,
                    reactions: ss(r.reactions),
                    has_quote: r.has_quote,
                    quote_sender: ss(r.quote_sender),
                    quote_text: ss(r.quote_text),
                    has_image: r.image.is_some(),
                    image: r.image.map(Image::from_rgba8).unwrap_or_default(),
                    image_w: r.image_w,
                    image_h: r.image_h,
                    has_media: r.has_media,
                    media_label: ss(r.media_label),
                    loading_media: r.loading_media,
                    day: ss(r.day),
                    pending: r.pending,
                    can_edit: r.can_edit,
                    can_revoke: r.can_revoke,
                })
                .collect();
            if switched {
                m.messages.set_vec(rows);
            } else {
                // Images compare by identity, so re-wrapped pixel buffers would
                // count as changes: compare everything but the image, and keep
                // the existing image unless it actually changed size/presence.
                let old: Vec<MsgRow> = m.messages.iter().collect();
                let rows = rows
                    .into_iter()
                    .map(|mut r| {
                        if let Some(o) = old.iter().find(|o| o.id == r.id)
                            && o.has_image == r.has_image
                            && o.image_w == r.image_w
                            && o.image_h == r.image_h
                        {
                            r.image = o.image.clone();
                        }
                        r
                    })
                    .collect();
                sync_model(&m.messages, rows, |a, b| a.id == b.id);
            }
            app.set_active_jid(ss(c.active_jid));
            app.set_active_title(ss(c.title));
            app.set_active_subtitle(ss(c.subtitle));
            app.set_active_initials(ss(c.initials));
            app.set_active_tint(rgb(c.tint));
            app.set_active_muted(c.muted);
            app.set_active_pinned(c.pinned);
            app.set_active_archived(c.archived);
            app.set_has_older(c.has_older);
            app.set_replying(c.replying);
            app.set_editing(c.editing);
            app.set_reply_sender(ss(c.reply_sender));
            app.set_reply_text(ss(c.reply_text));
            if let Some(t) = c.composer {
                app.set_composer_text(ss(t));
            }
            if app.get_scroll_token() != c.scroll_token {
                app.set_scroll_token(c.scroll_token);
            }
            if app.get_append_token() != c.append_token {
                app.set_append_token(c.append_token);
            }
        }
    }
}
