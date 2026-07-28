// Chat-list preview helpers: turn a message kind into a short labelled string
// (e.g. "📷 Photo") for non-text messages.

export function mediaLabel(kind: string): string {
  switch (kind) {
    case "image":
      return "📷 Photo";
    case "video":
      return "🎥 Video";
    case "audio":
      return "🎙 Audio";
    case "document":
      return "📄 Document";
    case "sticker":
      return "🃏 Sticker";
    case "buttons":
    case "list":
    case "interactive":
    case "template":
      return "📋 Interactive";
    case "poll":
      return "📊 Poll";
    case "order":
      return "🛒 Order";
    case "product":
      return "🛍 Product";
    case "contact":
      return "👤 Contact";
    case "location":
      return "📍 Location";
    default:
      return "Message";
  }
}

export function formatQuotedText(msg: {
  text?: string | null;
  kind?: string | null;
  media?: { fileName?: string | null } | null;
}): string {
  if (msg.text && msg.text.trim()) {
    if (msg.kind && msg.kind !== "text") {
      return `${mediaLabel(msg.kind)} ${msg.text}`;
    }
    return msg.text;
  }
  if (msg.media?.fileName) {
    return `📄 ${msg.media.fileName}`;
  }
  if (msg.kind) {
    return mediaLabel(msg.kind);
  }
  return "Message";
}
