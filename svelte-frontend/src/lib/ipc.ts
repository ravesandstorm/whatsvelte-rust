// Typed wrappers over Tauri IPC. Commands omit sessionId (backend defaults to
// "default"); the param exists in the backend for future multi-session use.
//
// Note: Tauri v2 maps camelCase JS arg keys to the snake_case Rust params.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ContactDto,
  Envelope,
  PairCodeDto,
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
  sendText: (jid: string, text: string) =>
    invoke<SendResultDto>("send_text", { jid, text }),
  markRead: (jid: string) => invoke<void>("mark_read", { jid }),
  getContact: (jid: string) => invoke<ContactDto>("get_contact", { jid }),
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
