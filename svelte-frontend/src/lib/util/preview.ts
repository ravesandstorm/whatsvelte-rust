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
    default:
      return "Message";
  }
}
