// A compact, dependency-free emoji set grouped by category. Deliberately curated
// (a few hundred common emoji) rather than the full Unicode set so we don't pull
// in a heavy picker dependency or a megabyte of data for an MVP feature.

export interface EmojiCategory {
  key: string;
  label: string;
  icon: string;
  emojis: string[];
}

export const EMOJI_CATEGORIES: EmojiCategory[] = [
  {
    key: "smileys",
    label: "Smileys & People",
    icon: "😀",
    emojis: [
      "😀","😃","😄","😁","😆","😅","🤣","😂","🙂","🙃","😉","😊","😇","🥰","😍","🤩",
      "😘","😗","😚","😙","😋","😛","😜","🤪","😝","🤑","🤗","🤭","🤫","🤔","🤐","🤨",
      "😐","😑","😶","😏","😒","🙄","😬","🤥","😌","😔","😪","🤤","😴","😷","🤒","🤕",
      "🤢","🤮","🤧","🥵","🥶","🥴","😵","🤯","🤠","🥳","😎","🤓","🧐","😕","😟","🙁",
      "😮","😯","😲","😳","🥺","😦","😧","😨","😰","😥","😢","😭","😱","😖","😣","😞",
      "😓","😩","😫","🥱","😤","😡","😠","🤬","😈","👿","💀","💩","🤡","👻","👽","🤖",
      "👋","🤚","✋","🖐️","👌","🤏","✌️","🤞","🤟","🤘","🤙","👈","👉","👆","👇","☝️",
      "👍","👎","✊","👊","🤛","🤜","👏","🙌","👐","🤲","🙏","💪","🦾","✍️","💅","🤳",
    ],
  },
  {
    key: "animals",
    label: "Animals & Nature",
    icon: "🐶",
    emojis: [
      "🐶","🐱","🐭","🐹","🐰","🦊","🐻","🐼","🐻‍❄️","🐨","🐯","🦁","🐮","🐷","🐸","🐵",
      "🐔","🐧","🐦","🐤","🦆","🦅","🦉","🦇","🐺","🐗","🐴","🦄","🐝","🐛","🦋","🐌",
      "🐞","🐜","🦗","🕷️","🦂","🐢","🐍","🦎","🐙","🦑","🦀","🐠","🐟","🐬","🐳","🐋",
      "🌵","🎄","🌲","🌳","🌴","🌱","🌿","☘️","🍀","🎍","🍃","🍂","🍁","🌷","🌹","🌻",
      "🌼","🌸","🌺","🌎","🌍","🌏","⭐","🌟","✨","⚡","☀️","🌤️","⛅","🌧️","🌈","🔥",
    ],
  },
  {
    key: "food",
    label: "Food & Drink",
    icon: "🍔",
    emojis: [
      "🍏","🍎","🍐","🍊","🍋","🍌","🍉","🍇","🍓","🫐","🍈","🍒","🍑","🥭","🍍","🥥",
      "🥝","🍅","🥑","🥦","🥬","🥒","🌶️","🌽","🥕","🧄","🧅","🥔","🍠","🥐","🥯","🍞",
      "🥖","🧀","🥚","🍳","🧇","🥞","🥓","🍗","🍖","🌭","🍔","🍟","🍕","🥪","🌮","🌯",
      "🥗","🍝","🍜","🍲","🍣","🍱","🍛","🍚","🍙","🍘","🍢","🍡","🍧","🍨","🍦","🍰",
      "🎂","🧁","🥧","🍫","🍬","🍭","🍮","🍯","🍼","☕","🍵","🧃","🥤","🍶","🍺","🍷",
    ],
  },
  {
    key: "activity",
    label: "Activity & Travel",
    icon: "⚽",
    emojis: [
      "⚽","🏀","🏈","⚾","🥎","🎾","🏐","🏉","🥏","🎱","🪀","🏓","🏸","🏒","🏑","🥍",
      "🏏","⛳","🏹","🎣","🥊","🥋","🎽","🛹","🛼","🛷","⛸️","🥌","🎿","⛷️","🏂","🏆",
      "🥇","🥈","🥉","🏅","🎖️","🎯","🎮","🎲","🧩","🎸","🎺","🎻","🥁","🎬","🎨","🎤",
      "🚗","🚕","🚙","🚌","🏎️","🚓","🚑","🚒","🚜","🛵","🏍️","✈️","🚀","🛸","🚁","⛵",
      "🚤","🛳️","⚓","🏖️","🏝️","🏔️","🗻","🏕️","🗽","🗼","🎡","🎢","🎠","🌋","🏟️","🎑",
    ],
  },
  {
    key: "objects",
    label: "Objects & Symbols",
    icon: "💡",
    emojis: [
      "⌚","📱","💻","⌨️","🖥️","🖨️","🖱️","🕹️","💽","💾","📷","📸","📹","🎥","📺","📻",
      "⏰","⏱️","🔋","🔌","💡","🔦","🕯️","📡","💰","💵","💳","💎","🔧","🔨","🛠️","🧰",
      "🔑","🗝️","🔒","🔓","📦","📫","📮","✏️","✒️","🖊️","📝","📒","📚","📖","🔖","📎",
      "❤️","🧡","💛","💚","💙","💜","🖤","🤍","🤎","💔","❣️","💕","💞","💓","💗","💖",
      "💘","💝","✅","❌","❓","❗","💯","🔥","⭐","🎉","🎊","🎈","🎁","🏳️","🏴","🚩",
    ],
  },
];

// Common reaction emojis surfaced as a quick row above the full picker.
export const COMMON_REACTIONS = ["👍", "❤️", "😂", "😮", "😢", "🙏"];

// `:shortcode:` → emoji, for the `:`-triggered suggestion popover in the
// composer. Curated to the most-typed names; the picker covers the long tail.
export const EMOJI_SHORTCODES: Record<string, string> = {
  smile: "😄", grin: "😁", laugh: "😆", joy: "😂", rofl: "🤣", sweat_smile: "😅",
  wink: "😉", blush: "😊", heart_eyes: "😍", kiss: "😘", yum: "😋", tongue: "😛",
  thinking: "🤔", neutral: "😐", roll_eyes: "🙄", smirk: "😏", relieved: "😌",
  sleepy: "😪", sleeping: "😴", mask: "😷", cool: "😎", nerd: "🤓", party: "🥳",
  cry: "😢", sob: "😭", angry: "😠", rage: "😡", scream: "😱", hushed: "😯",
  worried: "😟", confused: "😕", shush: "🤫", hugs: "🤗", money_mouth: "🤑",
  shrug: "🤷", facepalm: "🤦", clap: "👏", wave: "👋", ok_hand: "👌", v: "✌️",
  thumbsup: "👍", "+1": "👍", thumbsdown: "👎", "-1": "👎", fist: "✊", punch: "👊",
  pray: "🙏", muscle: "💪", point_up: "☝️", raised_hands: "🙌", writing: "✍️",
  heart: "❤️", broken_heart: "💔", orange_heart: "🧡", yellow_heart: "💛",
  green_heart: "💚", blue_heart: "💙", purple_heart: "💜", black_heart: "🖤",
  sparkling_heart: "💖", fire: "🔥", star: "⭐", sparkles: "✨", boom: "💥",
  hundred: "💯", tada: "🎉", confetti: "🎊", balloon: "🎈", gift: "🎁",
  check: "✅", x: "❌", question: "❓", exclamation: "❗", warning: "⚠️",
  eyes: "👀", skull: "💀", poop: "💩", ghost: "👻", alien: "👽", robot: "🤖",
  rocket: "🚀", sun: "☀️", rainbow: "🌈", coffee: "☕", beer: "🍺", pizza: "🍕",
  cake: "🎂", dog: "🐶", cat: "🐱", unicorn: "🦄", bee: "🐝",
};

/** Match `:query` shortcodes for the suggestion popover (prefix-first ranking). */
export function searchEmojiShortcodes(query: string, limit = 8): { code: string; emoji: string }[] {
  const q = query.toLowerCase();
  if (!q) return [];
  const entries = Object.entries(EMOJI_SHORTCODES);
  const starts = entries.filter(([code]) => code.startsWith(q));
  const contains = entries.filter(([code]) => !code.startsWith(q) && code.includes(q));
  return [...starts, ...contains].slice(0, limit).map(([code, emoji]) => ({ code, emoji }));
}

const RECENT_KEY = "emoji-recent";
const RECENT_MAX = 32;

export function getRecentEmojis(): string[] {
  try {
    const raw = localStorage.getItem(RECENT_KEY);
    return raw ? (JSON.parse(raw) as string[]) : [];
  } catch {
    return [];
  }
}

export function pushRecentEmoji(emoji: string) {
  try {
    const next = [emoji, ...getRecentEmojis().filter((e) => e !== emoji)].slice(0, RECENT_MAX);
    localStorage.setItem(RECENT_KEY, JSON.stringify(next));
  } catch {
    /* storage unavailable; recents simply won't persist */
  }
}
