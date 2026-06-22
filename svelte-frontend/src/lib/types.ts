// TS mirrors of the Rust DTOs (src-tauri/src/dto.rs) and the event envelope.

export interface StatusDto {
  sessionId: string;
  loggedIn: boolean;
  connected: boolean;
  jid: string | null;
  registered: boolean;
  pushName: string | null;
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

export interface MediaDescriptorDto {
  directPath: string;
  mediaKey: string;
  fileSha256: string;
  fileEncSha256: string;
  fileLength: number;
  mediaType: string;
}

export interface MediaDto {
  kind: string;
  mimetype: string | null;
  fileName: string | null;
  width: number | null;
  height: number | null;
  durationSecs: number | null;
  isAnimated: boolean | null;
  descriptor: MediaDescriptorDto;
}

export interface QuotedDto {
  id: string;
  senderJid: string | null;
  text: string | null;
  kind: string;
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
  /** Download descriptor + display info for media messages (null for text). */
  media: MediaDto | null;
  /** The quoted message, when this is a reply. */
  quoted: QuotedDto | null;
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

export interface ChatFlagsDto {
  jid: string;
  muted: boolean | null;
  pinned: boolean | null;
  archived: boolean | null;
}

export interface ReceiptDto {
  chatJid: string;
  senderJid: string;
  messageIds: string[];
  /** "delivered" | "read" | "played" | "sent" */
  status: string;
  timestamp: number;
}

export interface MessageUpdateDto {
  chatJid: string;
  targetId: string;
  /** "revoke" | "edit" | "reaction" */
  kind: string;
  text: string | null;
  senderJid: string | null;
  fromMe: boolean;
  timestamp: number;
}

export interface ResolveJidDto {
  pn: string | null;
  lid: string | null;
}

export interface ContactDto {
  jid: string;
  name: string | null;
  verifiedName: string | null;
  lid: string | null;
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
