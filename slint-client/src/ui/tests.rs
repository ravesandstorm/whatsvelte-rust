//! Controller-level behaviour tests on the offline demo dataset.
//!
//! No window and no network: `WA_DEMO` makes every backend call fail with
//! "not connected", which exercises the optimistic-update rollback paths the
//! Svelte UI had (and the Slint port must keep).

use std::sync::{Arc, Once};

use crate::core::api::Api;
use crate::core::session::SessionManager;
use crate::store::MessageStatus;
use crate::ui::controller::{Controller, DIRTY_CHATS, DIRTY_CONV, DIRTY_SESSION, ROW_WINDOW, lock};

const G: &str = "120363000000000001@g.us";
const ADA: &str = "919876543210@s.whatsapp.net";

fn env() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let dir = std::env::temp_dir().join(format!("wa-ui-tests-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        // SAFETY: set once, before any test thread reads these variables.
        unsafe {
            std::env::set_var("WA_DEMO", "1");
            std::env::set_var("WA_APP_DATA_DIR", &dir);
        }
    });
}

async fn demo() -> Arc<Controller> {
    env();
    let (tx, _rx) = async_channel::unbounded();
    let mgr = Arc::new(SessionManager::new(tx, std::env::temp_dir()));
    let c = Controller::new(
        Api::new(mgr),
        tokio::runtime::Handle::current(),
        slint::Weak::default(),
    );
    c.clone().run_demo().await;
    c
}

fn msg(c: &Controller, id: &str) -> crate::store::UiMessage {
    lock(&c.store)
        .find_message(G, id)
        .cloned()
        .expect("message")
}

fn toasts(c: &Controller) -> Vec<String> {
    lock(&c.view)
        .toasts
        .iter()
        .map(|(t, _)| t.clone())
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_send_is_marked_and_toasted() {
    let c = demo().await;
    c.clone().send("hello there".into()).await;
    let last = lock(&c.store).messages(G).last().cloned().unwrap();
    assert_eq!(last.dto.text.as_deref(), Some("hello there"));
    assert_eq!(last.status, MessageStatus::Failed);
    assert!(toasts(&c).contains(&"Message not sent".to_string()));
    assert_eq!(lock(&c.view).composer_text, "");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reaction_rolls_back_on_failure() {
    let c = demo().await;
    let before = msg(&c, "m4").reactions;
    c.clone().react("m4".into(), "😮".into()).await;
    assert_eq!(msg(&c, "m4").reactions, before);
    assert!(toasts(&c).contains(&"Couldn't send reaction".to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn deletes_roll_back_on_failure() {
    let c = demo().await;
    c.clone().delete_for_me("m2".into()).await;
    assert!(lock(&c.store).find_message(G, "m2").is_some());
    c.clone().delete_for_everyone("m2".into()).await;
    assert!(!msg(&c, "m2").deleted);
    let t = toasts(&c);
    assert!(t.contains(&"Couldn't delete the message".to_string()));
    assert!(t.contains(&"Couldn't delete the message for everyone".to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn edit_stashes_and_restores_the_composer() {
    let c = demo().await;
    c.composer_edited("half-typed".into(), 10);
    c.start_edit("m6".into());
    assert_eq!(lock(&c.view).composer_text, "cargo build --release 🚀");
    c.clone().send("cargo build".into()).await;
    // rolled back + stash restored
    assert_eq!(
        msg(&c, "m6").dto.text.as_deref(),
        Some("cargo build --release 🚀")
    );
    assert_eq!(lock(&c.view).composer_text, "half-typed");
    assert!(lock(&c.view).editing.is_none());
    assert!(toasts(&c).contains(&"Couldn't edit the message".to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reply_quotes_the_target() {
    let c = demo().await;
    c.reply_to("m1".into());
    c.clone().send("agreed".into()).await;
    let last = lock(&c.store).messages(G).last().cloned().unwrap();
    assert_eq!(last.dto.quoted.as_ref().map(|q| q.id.as_str()), Some("m1"));
    assert!(lock(&c.view).reply_to.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn seen_counts_unread_down() {
    let c = demo().await;
    let before = lock(&c.store).chats[G].unread;
    assert!(msg(&c, "m9").unread);
    c.seen("m9".into());
    assert!(!msg(&c, "m9").unread);
    assert_eq!(lock(&c.store).chats[G].unread, before - 1);
    // seeing it again is a no-op
    c.seen("m9".into());
    assert_eq!(lock(&c.store).chats[G].unread, before - 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shortcode_and_emoji_insertion() {
    let c = demo().await;
    c.composer_edited("hey :smi".into(), 8);
    assert!(!lock(&c.view).suggestions.is_empty());
    c.accept_suggestion(0);
    assert_eq!(lock(&c.view).composer_text, "hey 😄");
    assert!(lock(&c.view).suggestions.is_empty());

    c.composer_edited("ab".into(), 1);
    c.emoji_picked("🔥".into(), 1);
    assert_eq!(lock(&c.view).composer_text, "a🔥b");
    assert_eq!(
        lock(&c.settings).emoji_recent.first().map(String::as_str),
        Some("🔥")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn quote_jump_reveals_and_flashes() {
    let c = demo().await;
    c.reveal_quote("does-not-exist".into());
    assert!(toasts(&c).contains(&"The original message isn't available".to_string()));
    c.reveal_quote("m1".into());
    let v = lock(&c.view);
    assert_eq!(
        v.reveal
            .as_ref()
            .map(|(_, id, center)| (id.as_str(), *center)),
        Some(("m1", true))
    );
    assert_eq!(v.flash.as_ref().map(|(id, _)| id.as_str()), Some("m1"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn chat_flags_roll_back_on_failure() {
    let c = demo().await;
    c.clone().chat_action(ADA.into(), "pinned".into()).await;
    assert!(!lock(&c.store).chats[ADA].is_pinned());
}

/// A chat of `n` text messages, opened.
async fn long_chat(n: usize) -> (Arc<Controller>, String) {
    let c = demo().await;
    let jid = "447000000000@s.whatsapp.net".to_string();
    let msgs = (0..n)
        .map(|i| crate::store::UiMessage {
            dto: crate::core::dto::MessageDto {
                id: format!("l{i}"),
                chat_jid: jid.clone(),
                sender_jid: jid.clone(),
                timestamp: 1000 + i as i64,
                text: Some(format!("message {i}")),
                kind: "text".into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .collect();
    lock(&c.store).add_history(msgs);
    c.clone().select_chat(jid.clone()).await;
    (c, jid)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn row_window_follows_the_list_and_covers_the_whole_chat() {
    let (c, _) = long_chat(5000).await;
    let tail = c.build_frame(DIRTY_CONV).conv.unwrap();
    assert_eq!(tail.total_rows, 5000);
    assert_eq!(tail.rows.last().unwrap().id, "l4999");
    assert!(tail.rows.len() <= 2 * ROW_WINDOW);
    // Far up the history, and all the way to the top: no limit, no paging.
    for (row, first) in [(1000, 1000 - ROW_WINDOW), (0, 0)] {
        c.row_requested(row);
        let f = c.build_frame(DIRTY_CONV).conv.unwrap();
        assert_eq!(f.first_row, first);
        assert_eq!(f.rows.len(), 2 * ROW_WINDOW);
        assert_eq!(f.rows[row - first].id, format!("l{row}"));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rows_inside_the_window_do_not_rebuild_it() {
    let (c, _) = long_chat(5000).await;
    c.row_requested(2000);
    c.build_frame(DIRTY_CONV);
    c.dirty_bits();
    c.row_requested(2001);
    assert_eq!(c.dirty_bits() & DIRTY_CONV, 0, "same place: no rebuild");
    c.row_requested(2000 + ROW_WINDOW - 1);
    assert_ne!(c.dirty_bits() & DIRTY_CONV, 0, "near the edge: rebuild");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn history_inserted_above_shifts_the_window() {
    let (c, jid) = long_chat(1000).await;
    c.row_requested(500);
    let before = c.build_frame(DIRTY_CONV).conv.unwrap();
    let older = (0..25)
        .map(|i| crate::store::UiMessage {
            dto: crate::core::dto::MessageDto {
                id: format!("old{i}"),
                chat_jid: jid.clone(),
                sender_jid: jid.clone(),
                timestamp: 10 + i,
                text: Some("older".into()),
                kind: "text".into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .collect();
    lock(&c.store).add_history(older);
    let after = c.build_frame(DIRTY_CONV).conv.unwrap();
    assert_eq!(after.prepended, 25);
    assert_eq!(after.first_row, before.first_row + 25);
    assert_eq!(after.rows[0].id, before.rows[0].id);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reveal_centres_the_window_once() {
    let (c, _) = long_chat(5000).await;
    c.reveal_quote("l100".into());
    let f = c.build_frame(DIRTY_CONV).conv.unwrap();
    let (_, id, index, _) = f.reveal.expect("reveal");
    assert_eq!((id.as_str(), index), ("l100", 100));
    assert!(f.first_row <= 100 && f.first_row + f.rows.len() > 100);
    // Later frames must not pull the list back to the target.
    c.row_requested(3000);
    let f = c.build_frame(DIRTY_CONV).conv.unwrap();
    assert!(f.reveal.is_none());
    assert!(f.first_row > 2000);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reaching_the_bottom_marks_everything_read() {
    let (c, jid) = long_chat(300).await;
    lock(&c.store).chats.get_mut(&jid).unwrap().unread = 200;
    c.clone().select_chat(jid.clone()).await;
    c.reached_bottom();
    assert_eq!(lock(&c.store).chats[&jid].unread, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unread_divider_stays_put_while_reading() {
    let (c, jid) = long_chat(300).await;
    lock(&c.store).chats.get_mut(&jid).unwrap().unread = 50;
    c.clone().select_chat(jid.clone()).await;
    let divider = |c: &Controller| {
        let f = c.build_frame(DIRTY_CONV).conv.unwrap();
        f.rows
            .iter()
            .find(|r| r.unread_divider)
            .map(|r| r.id.clone())
    };
    assert_eq!(divider(&c).as_deref(), Some("l250"));
    c.seen("l250".into());
    c.seen("l251".into());
    assert_eq!(divider(&c).as_deref(), Some("l250"));
    c.jump_to_bottom();
    assert_eq!(divider(&c), None);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pairing_pill_is_connected_once_a_qr_arrives() {
    let c = demo().await;
    lock(&c.view).status.connected = false;
    assert!(!c.build_frame(DIRTY_SESSION).session.unwrap().connected);
    lock(&c.view).qr = Some(slint::SharedPixelBuffer::new(1, 1));
    assert!(c.build_frame(DIRTY_SESSION).session.unwrap().connected);
    lock(&c.view).dead = true;
    assert!(!c.build_frame(DIRTY_SESSION).session.unwrap().connected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn toasts_dedupe_and_cap_at_three() {
    let c = demo().await;
    for t in ["a", "a", "b", "c", "d"] {
        c.toast(t);
    }
    assert_eq!(toasts(&c), vec!["b", "c", "d"]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn frames_group_albums_and_parse_links() {
    let c = demo().await;
    let frame = c.build_frame(DIRTY_CONV | DIRTY_CHATS);
    let conv = frame.conv.expect("conv frame");
    let album = conv
        .rows
        .iter()
        .find(|r| !r.album.is_empty())
        .expect("album row");
    assert_eq!(album.album.len(), 4);
    assert_eq!(album.album[3].more, 2);
    let poll = conv.rows.iter().find(|r| r.id == "m5c").unwrap();
    assert_eq!(poll.structured, "Poll");
    let link = conv.rows.iter().find(|r| r.id == "m1").unwrap();
    let md = link.rich_md.as_deref().expect("markdown for a link");
    assert!(
        slint::StyledText::from_markdown(md).is_ok(),
        "bad markdown: {md}"
    );
    let deleted = conv.rows.iter().find(|r| r.id == "m7").unwrap();
    assert!(deleted.deleted && deleted.text.is_empty());
    // chat list: group rows get a sender prefix, drafts show in place of the preview
    let chats = frame.chats.expect("chats frame");
    let g = chats.rows.iter().find(|r| r.jid == G).unwrap();
    assert!(g.prefix.ends_with(':'));
    let lid = chats.rows.iter().find(|r| r.jid.ends_with("@lid")).unwrap();
    assert!(lid.draft && lid.preview == "unsent thought");
}

#[test]
fn tricky_link_markdown_always_parses() {
    for text in [
        "a*b_c [x](y) <tag> `code` #h https://x.io/a_(b)",
        "1. item\n- dash\n> quote www.example.com/path?q=1&r=2",
        "trailing https://a.b/c). and **bold** ~~s~~ | pipe {brace} \\ backslash",
        "emoji 🚀 https://x.io 😀\n\nnext paragraph",
    ] {
        let md = crate::ui::views::linkify_markdown(text).expect("has links");
        assert!(
            slint::StyledText::from_markdown(&md).is_ok(),
            "failed for {text:?}: {md}"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn retry_replaces_the_failed_bubble() {
    let c = demo().await;
    c.clone().send("again".into()).await;
    let failed = lock(&c.store).messages(G).last().cloned().unwrap();
    c.clone().retry(failed.dto.id.clone()).await;
    let n = lock(&c.store)
        .messages(G)
        .iter()
        .filter(|m| m.dto.text.as_deref() == Some("again"))
        .count();
    assert_eq!(n, 1, "retry must not duplicate the bubble");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn drafts_follow_the_chat() {
    let c = demo().await;
    c.composer_edited("note to self".into(), 12);
    assert_eq!(
        lock(&c.store).chats[G].draft.as_deref(),
        Some("note to self")
    );
    c.clone().select_chat("100000012345678@lid".into()).await;
    assert_eq!(lock(&c.view).composer_text, "unsent thought");
    c.clone().select_chat(G.into()).await;
    assert_eq!(lock(&c.view).composer_text, "note to self");
    c.clone().send("note to self".into()).await;
    assert!(
        lock(&c.store).chats[G].draft.is_none(),
        "sending clears the draft"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn deleting_a_failed_bubble_is_local() {
    let c = demo().await;
    c.clone().send("oops".into()).await;
    let failed = lock(&c.store).messages(G).last().cloned().unwrap();
    c.ask_delete(failed.dto.id.clone());
    assert_eq!(
        lock(&c.view).open_delete.clone(),
        Some((failed.dto.id.clone(), false))
    );
    c.clone().delete_for_me(failed.dto.id.clone()).await;
    assert!(lock(&c.store).find_message(G, &failed.dto.id).is_none());
    assert!(!toasts(&c).contains(&"Couldn't delete the message".to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn new_chat_validates_the_number() {
    let c = demo().await;
    c.clone().new_chat("12a".into()).await;
    assert!(toasts(&c).contains(&"Enter a phone number with country code".to_string()));
    c.clone().new_chat("+1 (555) 123-4567".into()).await;
    assert_eq!(c.active().as_deref(), Some("15551234567@s.whatsapp.net"));
}
