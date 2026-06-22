// Typed wrappers over Tauri IPC. Commands omit sessionId (backend defaults to
// "default"); the param exists in the backend for future multi-session use.
//
// Note: Tauri v2 maps camelCase JS arg keys to the snake_case Rust params.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ContactDto,
  Envelope,
  MediaDescriptorDto,
  PairCodeDto,
  ResolveJidDto,
  SendResultDto,
  StatusDto,
} from "./types";

export const api = {
  authStatus: () => invoke<StatusDto>("auth_status", {}),
  authStartQr: () => invoke<void>("auth_start_qr", {}),
  authStartPairCode: (phone: string, customCode?: string | null) =>
    invoke<PairCodeDto>("auth_start_pair_code", { phone, customCode: customCode ?? null }),
  connect: () => invoke<void>("connect", {}),
  disconnect: () => invoke<void>("disconnect", {}),
  logout: () => invoke<void>("auth_logout", {}),
  resetSession: () => invoke<void>("reset_session", {}),
  sendText: (jid: string, text: string) =>
    invoke<SendResultDto>("send_text", { jid, text }),
  sendReply: (
    jid: string,
    text: string,
    quotedId: string,
    quotedSender: string,
    quotedText: string | null,
  ) => invoke<SendResultDto>("send_reply", { jid, text, quotedId, quotedSender, quotedText }),
  editMessage: (jid: string, originalId: string, newText: string) =>
    invoke<SendResultDto>("edit_message", { jid, originalId, newText }),
  sendReaction: (
    jid: string,
    targetId: string,
    fromMe: boolean,
    emoji: string,
    participant: string | null,
  ) => invoke<void>("send_reaction", { jid, targetId, fromMe, emoji, participant }),
  markRead: (jid: string) => invoke<void>("mark_read", { jid }),
  markReadMessages: (jid: string, sender: string | null, messageIds: string[]) =>
    invoke<void>("mark_read_messages", { jid, sender, messageIds }),
  setChatMuted: (jid: string, muted: boolean) =>
    invoke<void>("set_chat_muted", { jid, muted }),
  setChatPinned: (jid: string, pinned: boolean) =>
    invoke<void>("set_chat_pinned", { jid, pinned }),
  setChatArchived: (jid: string, archived: boolean) =>
    invoke<void>("set_chat_archived", { jid, archived }),
  getContact: (jid: string) => invoke<ContactDto>("get_contact", { jid }),
  resolveJid: (jid: string) => invoke<ResolveJidDto>("resolve_jid", { jid }),
  downloadMedia: (descriptor: MediaDescriptorDto, mimetype: string | null) =>
    invoke<string>("download_media", { descriptor, mimetype }),
  getProfilePictureUrl: (jid: string, preview = true) =>
    invoke<string | null>("get_profile_picture_url", { jid, preview }),
};

/** Subscribe to a wa:// topic; the handler receives the inner payload + sessionId. */
export function on<T>(
  topic: string,
  handler: (payload: T, sessionId: string) => void,
): Promise<UnlistenFn> {
  return listen<Envelope<T>>(topic, (e) =>
    handler(e.payload.payload, e.payload.sessionId),
  );
}
