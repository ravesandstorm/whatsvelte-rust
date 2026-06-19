// TS mirrors of the Rust DTOs (src-tauri/src/dto.rs) and the event envelope.

export interface StatusDto {
  sessionId: string;
  loggedIn: boolean;
  connected: boolean;
  jid: string | null;
}

export interface PairCodeDto {
  sessionId: string;
  code: string;
}

export interface SendResultDto {
  sessionId: string;
  messageId: string;
  to: string;
}

export interface MessageDto {
  id: string;
  chatJid: string;
  senderJid: string;
  fromMe: boolean;
  /** Unix epoch seconds. */
  timestamp: number;
  pushName: string | null;
  text: string | null;
  /** "text" | "image" | "video" | "audio" | "document" | "sticker" | "other" */
  kind: string;
  /** base64 JPEG thumbnail, when present. */
  thumbnail: string | null;
}

export interface ChatDto {
  jid: string;
  name: string | null;
  lastMessage: string | null;
  timestamp: number;
  unread: number;
}

export interface HistoryDto {
  chats: ChatDto[];
  messages: MessageDto[];
}

export interface ContactDto {
  jid: string;
  name: string | null;
  pictureUrl: string | null;
}

export interface ApiError {
  code: string;
  message?: string;
}

/** Every wa:// event payload is wrapped in this envelope by the bridge. */
export interface Envelope<T = unknown> {
  sessionId: string;
  kind: string;
  payload: T;
}
