//! The command surface, formerly `#[tauri::command]` handlers invoked over IPC.
//! Each is now a plain async method on [`Api`] that UI callbacks call directly
//! (on the tokio runtime) — no serialization, no WebView round trip.
//! All operate on the default session; [`SessionManager`] stays multi-session.

use std::sync::Arc;

use whatsapp_rust::pair_code::PairCodeOptions;
use whatsapp_rust::wacore::proto_helpers::{build_quote_context, build_reaction_message, MessageExt};
use whatsapp_rust::waproto::whatsapp as wa;
use whatsapp_rust::{Jid, NewsletterMetadata, RevokeType};

use crate::core::dto::{ChatDto, ContactDto, PairCodeDto, ResolveJidDto, SendResultDto, StatusDto};
use crate::core::error::{ApiError, ApiResult};
use crate::core::session::SessionManager;

pub(crate) fn parse_jid(s: &str) -> ApiResult<Jid> {
    s.parse::<Jid>()
        .map_err(|e| ApiError::InvalidJid(format!("{s}: {e}")))
}

#[derive(Clone)]
// connect/disconnect/get_contact/get_profile_picture_url are kept for parity
// with the Tauri command surface; the UI doesn't call them yet.
#[allow(dead_code)]
pub struct Api {
    pub(crate) mgr: Arc<SessionManager>,
}

#[allow(dead_code)]
impl Api {
    pub fn new(mgr: Arc<SessionManager>) -> Self {
        Self { mgr }
    }

    // ---- auth ----

    /// Boot the session if needed and report login/connection state. The QR code
    /// (if not yet paired) arrives separately on the `wa://auth/qr` event.
    pub async fn auth_status(&self) -> ApiResult<StatusDto> {
        let (id, session) = self.mgr.session(None).await?;
        let device = session.client.persistence_manager().get_device_snapshot();
        let push_name = device.push_name.clone();
        Ok(StatusDto {
            session_id: id,
            logged_in: session.client.is_logged_in(),
            connected: session.client.is_connected(),
            jid: session.client.get_pn().map(|j| j.to_string()),
            lid: session.client.get_lid().map(|j| j.to_string()),
            registered: device.is_registered(),
            push_name: if push_name.is_empty() { None } else { Some(push_name) },
        })

    }

    /// Ensure the session is running so QR pairing starts. Idempotent. The code is
    /// pushed via the `wa://auth/qr` event.
    pub async fn auth_start_qr(&self) -> ApiResult<()> {
        self.mgr.session(None).await?;
        Ok(())
    }

    /// Request a phone-number pairing code (alternative to scanning the QR).
    pub async fn auth_start_pair_code(
        &self,
        phone: String,
        custom_code: Option<String>,
    ) -> ApiResult<PairCodeDto> {
        let (id, session) = self.mgr.session(None).await?;
        // The companion-hello IQ needs a live socket. The library's own bot pairs
        // only after wait_for_socket; mirror that so a click right after launch
        // doesn't fail with IqError::NotConnected.
        session
            .client
            .wait_for_socket(std::time::Duration::from_secs(30))
            .await
            .map_err(ApiError::library)?;
        let code = session
            .client
            .pair_with_code(PairCodeOptions {
                phone_number: phone,
                custom_code,
                ..Default::default()
            })
            .await
            .map_err(ApiError::source_chain)?;
        Ok(PairCodeDto {
            session_id: id,
            code,
        })
    }

    pub async fn connect(&self) -> ApiResult<()> {
        let (_, session) = self.mgr.session(None).await?;
        session.client.connect().await.map_err(ApiError::library)
    }

    pub async fn disconnect(&self) -> ApiResult<()> {
        let (_, session) = self.mgr.session(None).await?;
        session.client.disconnect().await;
        Ok(())
    }

    pub async fn auth_logout(&self) -> ApiResult<()> {
        let (_, session) = self.mgr.session(None).await?;
        session.client.logout().await.map_err(ApiError::library)
    }

    /// Hard-reset the on-disk session: delete the SQLite db (regenerating the device
    /// key) and boot a fresh session so a clean QR can be generated. The frontend
    /// calls this after a logout event and on stale-data recovery.
    pub async fn reset_session(&self) -> ApiResult<()> {
        let id = SessionManager::resolve_id(None);
        self.mgr.reset(&id).await?;
        Ok(())
    }

    // ---- messaging ----

    /// Send a plain text message to a JID (user or group).
    pub async fn send_text(
        &self,
        jid: String,
        text: String,
    ) -> ApiResult<SendResultDto> {
        let (id, session) = self.mgr.session(None).await?;
        let to = parse_jid(&jid)?;
        let result = session
            .client
            .send_text(to, text)
            .await
            .map_err(ApiError::library)?;
        Ok(SendResultDto {
            session_id: id,
            message_id: result.message_id,
            to: result.to.to_string(),
        })
    }

    /// Send a text reply that quotes an existing message. The quoted preview is a
    /// best-effort text snapshot (the backend keeps no message store to reconstruct
    /// the original proto); the `quotedId` is what actually links the reply.
    pub async fn send_reply(
        &self,
        jid: String,
        text: String,
        quoted_id: String,
        quoted_sender: String,
        quoted_text: Option<String>,
    ) -> ApiResult<SendResultDto> {
        let (id, session) = self.mgr.session(None).await?;
        let to = parse_jid(&jid)?;

        let mut msg = wa::Message {
            extended_text_message: Some(Box::new(wa::message::ExtendedTextMessage {
                text: Some(text),
                ..Default::default()
            })),
            ..Default::default()
        };
        let quoted = wa::Message {
            conversation: quoted_text.filter(|s| !s.is_empty()),
            ..Default::default()
        };
        msg.set_context_info(build_quote_context(quoted_id, quoted_sender, &quoted));

        let result = session
            .client
            .send_message(to, msg)
            .await
            .map_err(ApiError::library)?;
        Ok(SendResultDto {
            session_id: id,
            message_id: result.message_id,
            to: result.to.to_string(),
        })
    }

    /// Edit one of our own text messages in place (legacy protocolMessage path).
    pub async fn edit_message(
        &self,
        jid: String,
        original_id: String,
        new_text: String,
    ) -> ApiResult<SendResultDto> {
        let (id, session) = self.mgr.session(None).await?;
        let to = parse_jid(&jid)?;
        let new_content = wa::Message {
            conversation: Some(new_text),
            ..Default::default()
        };
        let message_id = session
            .client
            .edit_message(to.clone(), original_id, new_content)
            .await
            .map_err(ApiError::library)?;
        Ok(SendResultDto {
            session_id: id,
            message_id,
            to: to.to_string(),
        })
    }

    /// React to a message with an emoji (empty string removes our reaction). For a
    /// group message, pass the target's sender as `participant`.
    pub async fn send_reaction(
        &self,
        jid: String,
        target_id: String,
        from_me: bool,
        emoji: String,
        participant: Option<String>,
    ) -> ApiResult<()> {
        let (_, session) = self.mgr.session(None).await?;
        let chat = parse_jid(&jid)?;
        let key = wa::MessageKey {
            remote_jid: Some(jid),
            from_me: Some(from_me),
            id: Some(target_id),
            participant,
        };
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let msg = build_reaction_message(key, emoji, now_ms);
        session
            .client
            .send_message(chat, msg)
            .await
            .map_err(ApiError::library)?;
        Ok(())
    }

    /// Delete a message for everyone (revoke). For your own message pass
    /// `participant: None`; as a group admin deleting another user's message, pass
    /// that user's JID (as-is, no LID→PN canonicalization) as `participant`.
    pub async fn revoke_message(
        &self,
        jid: String,
        message_id: String,
        participant: Option<String>,
    ) -> ApiResult<()> {
        let (_, session) = self.mgr.session(None).await?;
        let to = parse_jid(&jid)?;
        let revoke_type = match participant {
            Some(p) => RevokeType::Admin {
                original_sender: parse_jid(&p)?,
            },
            None => RevokeType::Sender,
        };
        session
            .client
            .revoke_message(to, message_id, revoke_type)
            .await
            .map_err(ApiError::library)?;
        Ok(())
    }

    /// Delete a message locally only ("delete for me"). For a group message from
    /// someone else, pass their JID as `participant`.
    pub async fn delete_for_me(
        &self,
        jid: String,
        message_id: String,
        from_me: bool,
        participant: Option<String>,
        timestamp: Option<i64>,
    ) -> ApiResult<()> {
        let (_, session) = self.mgr.session(None).await?;
        let chat = parse_jid(&jid)?;
        let participant_jid = match participant {
            Some(p) => Some(parse_jid(&p)?),
            None => None,
        };
        session
            .client
            .chat_actions()
            .delete_message_for_me(&chat, participant_jid.as_ref(), &message_id, from_me, false, timestamp)
            .await
            .map_err(ApiError::library)?;
        Ok(())
    }

    /// Send read receipts for specific message ids (viewport-driven). For group
    /// messages pass the message's sender; for DMs/status pass `None`.
    pub async fn mark_read_messages(
        &self,
        jid: String,
        sender: Option<String>,
        message_ids: Vec<String>,
    ) -> ApiResult<()> {
        if message_ids.is_empty() {
            return Ok(());
        }
        let (_, session) = self.mgr.session(None).await?;
        let chat = parse_jid(&jid)?;
        let sender_jid = match sender {
            Some(s) => Some(parse_jid(&s)?),
            None => None,
        };
        let ids: Vec<&str> = message_ids.iter().map(String::as_str).collect();
        session
            .client
            .mark_as_read(&chat, sender_jid.as_ref(), &ids)
            .await
            .map_err(ApiError::library)
    }

    /// Mark a chat as read (clears the unread badge for the whole chat).
    pub async fn mark_read(
        &self,
        jid: String,
    ) -> ApiResult<()> {
        let (_, session) = self.mgr.session(None).await?;
        let chat = parse_jid(&jid)?;
        session
            .client
            .chat_actions()
            .mark_chat_as_read(&chat, true, None)
            .await
            .map_err(ApiError::library)
    }

    // ---- chat_settings ----

    pub async fn set_chat_muted(
        &self,
        jid: String,
        muted: bool,
    ) -> ApiResult<()> {
        let (_, session) = self.mgr.session(None).await?;
        let chat = parse_jid(&jid)?;
        let actions = session.client.chat_actions();
        if muted {
            actions.mute_chat(&chat).await
        } else {
            actions.unmute_chat(&chat).await
        }
        .map_err(ApiError::library)
    }

    pub async fn set_chat_pinned(
        &self,
        jid: String,
        pinned: bool,
    ) -> ApiResult<()> {
        let (_, session) = self.mgr.session(None).await?;
        let chat = parse_jid(&jid)?;
        let actions = session.client.chat_actions();
        if pinned {
            actions.pin_chat(&chat).await
        } else {
            actions.unpin_chat(&chat).await
        }
        .map_err(ApiError::library)
    }

    pub async fn set_chat_archived(
        &self,
        jid: String,
        archived: bool,
    ) -> ApiResult<()> {
        let (_, session) = self.mgr.session(None).await?;
        let chat = parse_jid(&jid)?;
        let actions = session.client.chat_actions();
        if archived {
            actions.archive_chat(&chat, None).await
        } else {
            actions.unarchive_chat(&chat, None).await
        }
        .map_err(ApiError::library)
    }

    // ---- contacts ----

    pub async fn get_contact(
        &self,
        jid: String,
    ) -> ApiResult<ContactDto> {
        let (_, session) = self.mgr.session(None).await?;
        let parsed = parse_jid(&jid)?;

        // Only verified businesses expose a name here; regular names come from the
        // event/history stream. Best-effort: don't fail the whole call on lookup error.
        let info = session
            .client
            .contacts()
            .get_user_info(std::slice::from_ref(&parsed))
            .await
            .ok()
            .and_then(|mut m| m.remove(&parsed));

        let verified_name = info
            .as_ref()
            .and_then(|u| u.verified_name.as_ref().and_then(|vn| vn.name.clone()));
        let lid = info.as_ref().and_then(|u| u.lid.as_ref().map(|l| l.to_string()));

        let picture_url = session
            .client
            .contacts()
            .get_profile_picture(&parsed, true)
            .await
            .ok()
            .flatten()
            .map(|p| p.url);

        Ok(ContactDto {
            jid,
            name: verified_name.clone(),
            verified_name,
            lid,
            picture_url,
        })
    }

    /// Resolve a JID's LID ↔ phone-number identities so the UI can unify the two
    /// addressing forms of one contact into a single conversation.
    pub async fn resolve_jid(
        &self,
        jid: String,
    ) -> ApiResult<ResolveJidDto> {
        let (_, session) = self.mgr.session(None).await?;
        let parsed = parse_jid(&jid)?;
        let entry = session
            .client
            .get_lid_pn_entry(&parsed)
            .await
            .map_err(ApiError::library)?;
        Ok(match entry {
            Some(e) => ResolveJidDto {
                pn: Some(format!("{}@s.whatsapp.net", e.phone_number)),
                lid: Some(format!("{}@lid", e.lid)),
            },
            None => ResolveJidDto { pn: None, lid: None },
        })
    }

    pub async fn get_profile_picture_url(
        &self,
        jid: String,
        preview: Option<bool>,
    ) -> ApiResult<Option<String>> {
        let (_, session) = self.mgr.session(None).await?;
        let parsed = parse_jid(&jid)?;
        let url = session
            .client
            .contacts()
            .get_profile_picture(&parsed, preview.unwrap_or(true))
            .await
            .map_err(ApiError::library)?
            .map(|p| p.url);
        Ok(url)
    }

    // ---- newsletters ----

    /// List the channels (newsletters) this account is subscribed to, as chat rows.
    /// Live network call (MEX over the socket) — requires an active connection.
    pub async fn list_newsletters(&self) -> ApiResult<Vec<ChatDto>> {
        let (_, session) = self.mgr.session(None).await?;
        let newsletters: Vec<NewsletterMetadata> = session
            .client
            .newsletter()
            .list_subscribed()
            .await
            .map_err(ApiError::library)?;

        Ok(newsletters
            .into_iter()
            .map(|meta| ChatDto {
                jid: meta.jid.to_string(),
                name: Some(meta.name),
                last_message: None,
                timestamp: meta.creation_time.map(|t| t as i64).unwrap_or(0),
                unread: 0,
            })
            .collect())
    }
}
