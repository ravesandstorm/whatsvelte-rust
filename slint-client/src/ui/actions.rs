//! Chat / message / composer / settings actions (ports of the Svelte
//! components' handlers, with the same optimistic + rollback semantics).

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use crate::core::dto::QuotedDto;
use crate::core::error::ApiError;
use crate::store::settings::ZOOM_STEP;
use crate::store::{MessageStatus, Section, UiMessage, quoted_text};
use crate::ui::controller::{
    Controller, DIRTY_ALL, DIRTY_CHATS, DIRTY_CONV, DIRTY_OVERLAY, DIRTY_SESSION, FOCUS_TAIL, lock,
};
use crate::util::emoji;
use crate::util::jid::{is_group, normalize_jid};
use crate::util::time::now_secs;

impl Controller {
    pub(crate) fn active_message(&self, id: &str) -> Option<UiMessage> {
        let s = lock(&self.store);
        let jid = s.active.clone()?;
        s.find_message(&jid, id).cloned()
    }

    // ================================================================ chat list

    pub async fn select_chat(self: Arc<Self>, jid: String) {
        let me = self.clone();
        let j = jid.clone();
        let _ = tokio::task::spawn_blocking(move || lock(&me.store).open_chat(&j)).await;
        // Decoded images belong to the chat being left.
        lock(&self.images).clear();
        self.focus.store(FOCUS_TAIL, Ordering::Relaxed);
        self.audio.stop();
        let (draft, first_unread) = {
            let s = lock(&self.store);
            let draft = s
                .chats
                .get(&jid)
                .and_then(|c| c.draft.clone())
                .unwrap_or_default();
            let first_unread = s
                .messages(&jid)
                .iter()
                .find(|m| m.unread)
                .map(|m| m.dto.id.clone());
            (draft, first_unread)
        };
        {
            let mut v = lock(&self.view);
            v.reply_to = None;
            v.editing = None;
            v.edit_stash = None;
            v.suggestions.clear();
            v.composer_text = draft.clone();
            v.composer_override = Some((draft, None));
            v.focus_token += 1;
            v.unread_divider = first_unread.clone();
            match first_unread {
                // Put the "Unread" divider at the top of the view.
                Some(id) => {
                    v.reveal_token += 1;
                    v.reveal = Some((v.reveal_token, id, false));
                }
                None => {
                    v.reveal = None;
                    v.scroll_token += 1;
                }
            }
        }
        self.mark(DIRTY_CHATS | DIRTY_CONV);
    }

    pub fn set_section(&self, section: i32) {
        lock(&self.view).section = match section {
            1 => Section::Archived,
            2 => Section::Channels,
            3 => Section::Status,
            _ => Section::Chats,
        };
        self.mark(DIRTY_CHATS);
    }

    pub fn set_search(&self, q: String) {
        lock(&self.view).search = q;
        self.mark(DIRTY_CHATS);
    }

    pub async fn new_chat(self: Arc<Self>, input: String) {
        let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() < 6 {
            self.toast("Enter a phone number with country code");
            return;
        }
        lock(&self.view).section = Section::Chats;
        self.select_chat(format!("{digits}@s.whatsapp.net")).await;
    }

    /// Pin / mute / archive toggle — optimistic, reverted on error.
    pub async fn chat_action(self: Arc<Self>, jid: String, flag: String) {
        let current = {
            let s = lock(&self.store);
            let Some(c) = s.chats.get(&jid) else { return };
            match flag.as_str() {
                "muted" => c.is_muted(),
                "pinned" => c.is_pinned(),
                "archived" => c.is_archived(),
                _ => return,
            }
        };
        let value = !current;
        lock(&self.store).set_flag(&jid, &flag, value);
        self.mark(DIRTY_CHATS | DIRTY_CONV);
        let result = match flag.as_str() {
            "muted" => self.api.set_chat_muted(jid.clone(), value).await,
            "pinned" => self.api.set_chat_pinned(jid.clone(), value).await,
            _ => self.api.set_chat_archived(jid.clone(), value).await,
        };
        if let Err(e) = result {
            log::warn!("chat {flag} failed: {e}");
            lock(&self.store).set_flag(&jid, &flag, current);
            self.mark(DIRTY_CHATS | DIRTY_CONV);
        }
    }

    pub async fn toggle_active_flag(self: Arc<Self>, flag: &str) {
        if let Some(jid) = self.active() {
            self.chat_action(jid, flag.to_string()).await;
        }
    }

    // ================================================================ message list

    fn mark_shown_read(self: &Arc<Self>) {
        let ids: Vec<String> = {
            let s = lock(&self.store);
            let Some(jid) = s.active.clone() else { return };
            s.messages(&jid)
                .iter()
                .filter(|m| m.unread)
                .map(|m| m.dto.id.clone())
                .collect()
        };
        for id in ids {
            self.seen(id);
        }
    }

    /// Back at the bottom: everything above has been passed, so mark it read.
    pub fn reached_bottom(self: &Arc<Self>) {
        self.mark_shown_read();
    }

    pub fn jump_to_bottom(self: &Arc<Self>) {
        let mut v = lock(&self.view);
        v.reveal = None;
        v.unread_divider = None;
        v.scroll_token += 1;
        drop(v);
        self.focus.store(FOCUS_TAIL, Ordering::Relaxed);
        self.mark(DIRTY_CONV);
        self.mark_shown_read();
    }

    /// Scroll to a quoted message and flash it.
    pub fn reveal_quote(&self, id: String) {
        let Some(jid) = self.active() else { return };
        if lock(&self.store).find_message(&jid, &id).is_none() {
            self.toast("The original message isn't available");
            return;
        }
        let mut v = lock(&self.view);
        v.reveal_token += 1;
        v.reveal = Some((v.reveal_token, id.clone(), true));
        v.flash = Some((id, Instant::now()));
        drop(v);
        self.mark(DIRTY_CONV);
    }

    pub fn open_link(&self, url: String) {
        let lower = url.to_ascii_lowercase();
        if !(lower.starts_with("http://") || lower.starts_with("https://")) {
            return;
        }
        if let Err(e) = open::that_detached(&url) {
            self.toast(format!("Couldn't open link: {e}"));
        }
    }

    // ================================================================ composer

    pub fn composer_edited(&self, text: String, caret: usize) {
        let (editing, jid) = (lock(&self.view).editing.is_some(), self.active());
        if !editing && let Some(jid) = &jid {
            lock(&self.store).set_draft(jid, &text);
        }
        let caret = caret.min(text.len());
        let before = text.get(..caret).unwrap_or(&text);
        let mut v = lock(&self.view);
        v.composer_text = text.clone();
        v.suggestions = match emoji::shortcode_token(before) {
            Some(token) => {
                v.suggestion_start = caret - token.len() - 1;
                v.suggestion_caret = caret;
                emoji::search(token, 8)
                    .into_iter()
                    .map(|(code, e)| (e.to_string(), code.to_string()))
                    .collect()
            }
            None => Vec::new(),
        };
        v.suggestion_index = 0;
        drop(v);
        self.mark(DIRTY_CONV | DIRTY_CHATS);
    }

    pub fn accept_suggestion(&self, index: usize) {
        let mut v = lock(&self.view);
        let Some((emoji, _)) = v.suggestions.get(index).cloned() else {
            return;
        };
        let text = v.composer_text.clone();
        let (start, caret) = (
            v.suggestion_start.min(text.len()),
            v.suggestion_caret.min(text.len()),
        );
        let new_text = format!("{}{}{}", &text[..start], emoji, &text[caret..]);
        let new_caret = (start + emoji.len()) as i32;
        v.suggestions.clear();
        v.composer_text = new_text.clone();
        v.composer_override = Some((new_text.clone(), Some(new_caret)));
        let editing = v.editing.is_some();
        drop(v);
        if !editing && let Some(jid) = self.active() {
            lock(&self.store).set_draft(&jid, &new_text);
        }
        self.mark(DIRTY_CONV);
    }

    /// Insert an emoji at the caret (from the emoji panel) and remember it.
    pub fn emoji_picked(&self, emoji: String, caret: usize) {
        lock(&self.settings).push_recent_emoji(&emoji);
        lock(&self.settings).save();
        let mut v = lock(&self.view);
        let text = v.composer_text.clone();
        let mut caret = caret.min(text.len());
        while !text.is_char_boundary(caret) {
            caret -= 1;
        }
        let new_text = format!("{}{}{}", &text[..caret], emoji, &text[caret..]);
        v.composer_text = new_text.clone();
        v.composer_override = Some((new_text.clone(), Some((caret + emoji.len()) as i32)));
        v.focus_token += 1;
        let editing = v.editing.is_some();
        drop(v);
        if !editing && let Some(jid) = self.active() {
            lock(&self.store).set_draft(&jid, &new_text);
        }
        self.mark(DIRTY_CONV | DIRTY_OVERLAY);
    }

    pub fn cancel_compose(&self) {
        let mut v = lock(&self.view);
        if v.editing.take().is_some() {
            let stash = v.edit_stash.take().unwrap_or_default();
            v.composer_text = stash.clone();
            v.composer_override = Some((stash, None));
        } else {
            v.reply_to = None;
        }
        drop(v);
        self.mark(DIRTY_CONV);
    }

    pub fn reply_to(&self, id: String) {
        let Some(m) = self.active_message(&id) else {
            return;
        };
        if m.deleted || m.is_pending() {
            return;
        }
        let mut v = lock(&self.view);
        if v.editing.take().is_some() {
            let stash = v.edit_stash.take().unwrap_or_default();
            v.composer_text = stash.clone();
            v.composer_override = Some((stash, None));
        }
        v.reply_to = Some(id);
        v.focus_token += 1;
        drop(v);
        self.mark(DIRTY_CONV);
    }

    pub fn start_edit(&self, id: String) {
        let Some(m) = self.active_message(&id) else {
            return;
        };
        let mut v = lock(&self.view);
        if v.editing.is_none() {
            v.edit_stash = Some(v.composer_text.clone());
        }
        v.reply_to = None;
        v.editing = Some(id);
        let text = m.dto.text.unwrap_or_default();
        v.composer_text = text.clone();
        v.composer_override = Some((text.clone(), Some(text.len() as i32)));
        v.focus_token += 1;
        drop(v);
        self.mark(DIRTY_CONV);
    }

    pub async fn send(self: Arc<Self>, text: String) {
        let body = text.trim().to_string();
        let Some(jid) = self.active() else { return };
        if body.is_empty() {
            return;
        }
        let (reply_to, editing) = {
            let mut v = lock(&self.view);
            let editing = v.editing.take();
            let restore = if editing.is_some() {
                v.edit_stash.take().unwrap_or_default()
            } else {
                String::new()
            };
            v.composer_text = restore.clone();
            v.composer_override = Some((restore, None));
            v.suggestions.clear();
            v.scroll_token += 1;
            (v.reply_to.take(), editing)
        };

        if let Some(id) = editing {
            let Some(before) = self.active_message(&id) else {
                return;
            };
            lock(&self.store).apply_edit(&jid, &id, Some(body.clone()), now_secs());
            self.mark(DIRTY_CHATS | DIRTY_CONV);
            if let Err(e) = self
                .api
                .edit_message(before.proto_chat_jid().to_string(), id, body)
                .await
            {
                log::warn!("edit failed: {e}");
                lock(&self.store).restore_message(before);
                self.toast("Couldn't edit the message");
                self.mark(DIRTY_CHATS | DIRTY_CONV);
            }
            return;
        }

        lock(&self.store).set_draft(&jid, "");
        let target = reply_to.and_then(|id| self.active_message(&id));
        let quoted = target.as_ref().map(|q| QuotedDto {
            id: q.dto.id.clone(),
            sender_jid: Some(if q.dto.from_me {
                self.self_jid().unwrap_or_default()
            } else {
                q.dto.sender_jid.clone()
            }),
            text: q.dto.text.clone(),
            kind: q.dto.kind.clone(),
            thumbnail: q.dto.thumbnail.clone(),
        });
        let tmp = lock(&self.store).add_optimistic(&jid, "text", Some(body.clone()), None, quoted);
        self.mark(DIRTY_CHATS | DIRTY_CONV);

        let result = match &target {
            Some(q) => {
                let sender = if q.dto.from_me {
                    self.self_jid()
                        .map(|j| normalize_jid(&j))
                        .unwrap_or_default()
                } else if q.proto_sender_jid().is_empty() {
                    q.proto_chat_jid().to_string()
                } else {
                    q.proto_sender_jid().to_string()
                };
                let qt = quoted_text(
                    &q.dto.kind,
                    q.dto.text.as_deref(),
                    q.dto.media.as_ref().and_then(|m| m.file_name.as_deref()),
                );
                self.api
                    .send_reply(
                        q.proto_chat_jid().to_string(),
                        body,
                        q.dto.id.clone(),
                        sender,
                        Some(qt),
                    )
                    .await
            }
            None => self.api.send_text(jid.clone(), body).await,
        };
        self.finish_send(&jid, &tmp, result.map(|r| r.message_id), "Message not sent");
    }

    pub(crate) fn finish_send(
        &self,
        jid: &str,
        tmp: &str,
        result: Result<String, ApiError>,
        failure: &str,
    ) {
        match result {
            Ok(real) => lock(&self.store).confirm_optimistic(jid, tmp, &real),
            Err(e) => {
                log::warn!("send failed: {e}");
                lock(&self.store).fail_optimistic(jid, tmp);
                self.toast(failure);
            }
        }
        self.mark(DIRTY_CHATS | DIRTY_CONV);
    }

    pub async fn retry(self: Arc<Self>, id: String) {
        let Some(m) = self.active_message(&id) else {
            return;
        };
        if m.status != MessageStatus::Failed || m.dto.kind != "text" {
            return;
        }
        lock(&self.store).remove_message(&m.dto.chat_jid, &id);
        self.send(m.dto.text.unwrap_or_default()).await;
    }

    // ================================================================ reactions / delete

    pub fn open_reactions(&self, id: String) {
        let Some(m) = self.active_message(&id) else {
            return;
        };
        if m.deleted || m.is_pending() {
            return;
        }
        let me = self
            .self_jid()
            .map(|j| normalize_jid(&j))
            .unwrap_or_default();
        let current = m.reactions.get(&me).cloned().unwrap_or_default();
        lock(&self.view).open_reaction = Some((id, current));
        self.mark(DIRTY_OVERLAY);
    }

    pub async fn react(self: Arc<Self>, id: String, emoji: String) {
        let Some(before) = self.active_message(&id) else {
            return;
        };
        if before.is_pending() || before.deleted {
            return;
        }
        let jid = before.dto.chat_jid.clone();
        let me = self
            .self_jid()
            .map(|j| normalize_jid(&j))
            .unwrap_or_default();
        // Picking your current reaction again removes it.
        let emoji = if before.reactions.get(&me) == Some(&emoji) {
            String::new()
        } else {
            emoji
        };
        if !emoji.is_empty() {
            lock(&self.settings).push_recent_emoji(&emoji);
        }
        lock(&self.store).apply_reaction(&jid, &id, &me, &emoji);
        self.mark(DIRTY_CONV);
        let participant =
            (is_group(&jid) && !before.dto.from_me).then(|| before.proto_sender_jid().to_string());
        if let Err(e) = self
            .api
            .send_reaction(
                before.proto_chat_jid().to_string(),
                id,
                before.dto.from_me,
                emoji,
                participant,
            )
            .await
        {
            log::warn!("reaction failed: {e}");
            lock(&self.store).restore_message(before);
            self.toast("Couldn't send reaction");
            self.mark(DIRTY_CONV);
        }
    }

    pub fn ask_delete(&self, id: String) {
        let Some(m) = self.active_message(&id) else {
            return;
        };
        let everyone = m.dto.from_me && !m.deleted && !m.is_pending();
        lock(&self.view).open_delete = Some((id, everyone));
        self.mark(DIRTY_OVERLAY);
    }

    pub async fn delete_for_me(self: Arc<Self>, id: String) {
        let Some(before) = self.active_message(&id) else {
            return;
        };
        let jid = before.dto.chat_jid.clone();
        lock(&self.store).remove_message(&jid, &id);
        self.mark(DIRTY_CHATS | DIRTY_CONV);
        if before.is_pending() {
            return; // local-only bubble
        }
        let participant =
            (is_group(&jid) && !before.dto.from_me).then(|| before.proto_sender_jid().to_string());
        if let Err(e) = self
            .api
            .delete_for_me(
                before.proto_chat_jid().to_string(),
                id,
                before.dto.from_me,
                participant,
                Some(before.dto.timestamp),
            )
            .await
        {
            log::warn!("delete for me failed: {e}");
            lock(&self.store).restore_message(before);
            self.toast("Couldn't delete the message");
            self.mark(DIRTY_CHATS | DIRTY_CONV);
        }
    }

    pub async fn delete_for_everyone(self: Arc<Self>, id: String) {
        let Some(before) = self.active_message(&id) else {
            return;
        };
        if !before.dto.from_me || before.is_pending() || before.deleted {
            return;
        }
        let jid = before.dto.chat_jid.clone();
        lock(&self.store).apply_revoke(&jid, &id);
        self.mark(DIRTY_CHATS | DIRTY_CONV);
        if let Err(e) = self
            .api
            .revoke_message(before.proto_chat_jid().to_string(), id, None)
            .await
        {
            log::warn!("revoke failed: {e}");
            lock(&self.store).restore_message(before);
            self.toast("Couldn't delete the message for everyone");
            self.mark(DIRTY_CHATS | DIRTY_CONV);
        }
    }

    // ================================================================ pairing / account

    pub async fn request_pair_code(self: Arc<Self>, phone: String) {
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() < 6 {
            lock(&self.view).pair_error = "Enter your full phone number with country code.".into();
            self.mark(DIRTY_SESSION);
            return;
        }
        {
            let mut v = lock(&self.view);
            v.pair_busy = true;
            v.pair_error.clear();
        }
        self.mark(DIRTY_SESSION);
        let result = self.api.auth_start_pair_code(digits, None).await;
        let mut v = lock(&self.view);
        v.pair_busy = false;
        match result {
            Ok(c) => v.pair_code = c.code,
            Err(e) => v.pair_error = format!("Couldn't get a pairing code: {e}"),
        }
        drop(v);
        self.mark(DIRTY_SESSION);
    }

    /// Log out; the server's `LoggedOut` event (and this) reset everything.
    pub async fn logout(self: Arc<Self>) {
        if let Err(e) = self.api.auth_logout().await {
            log::warn!("logout failed: {e}");
        }
        self.reset_all().await;
    }

    pub async fn reset_session(self: Arc<Self>) {
        self.reset_all().await;
    }

    // ================================================================ settings

    pub fn update_settings(&self, f: impl FnOnce(&mut crate::store::settings::Settings)) {
        let mut s = lock(&self.settings);
        f(&mut s);
        s.save();
        drop(s);
        self.mark(DIRTY_ALL);
    }

    /// Ctrl/Cmd + `+`/`-`/`0` (step 0, +1 or -1; 0 resets).
    pub fn zoom_step(&self, dir: i32) {
        self.update_settings(|s| {
            if dir == 0 {
                s.set_zoom(1.0);
            } else {
                let z = s.zoom + dir as f32 * ZOOM_STEP;
                s.set_zoom(z);
            }
        });
    }

    pub fn set_picker_tab(self: &Arc<Self>, tab: i32) {
        lock(&self.view).picker_tab = tab;
        self.mark(DIRTY_OVERLAY);
        if tab == 1 {
            let empty = lock(&self.view).gif.page == 0;
            if empty {
                self.gif_search(String::new());
            }
        }
    }

    pub fn set_emoji_tab(&self, tab: i32) {
        lock(&self.view).emoji_tab = tab;
        self.mark(DIRTY_OVERLAY);
    }

    pub fn set_emoji_search(&self, q: String) {
        lock(&self.view).emoji_search = q;
        self.mark(DIRTY_OVERLAY);
    }
}
