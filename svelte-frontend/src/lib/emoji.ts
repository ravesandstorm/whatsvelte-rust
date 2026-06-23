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
      "😀","😃","😄","😁","😆","😅","🤣","😂","🙂","🙃","🫠","😉","😊","😇","🥰","😍",
      "🤩","😘","😗","☺️","😚","😙","🥲","😋","😛","😜","🤪","😝","🤑","🤗","🤭","🫢",
      "🫣","🤫","🤔","🫡","🤐","🤨","😐","😑","😶","🫥","😶‍🌫️","😏","😒","🙄","😬","😮‍💨",
      "🤥","😌","😔","😪","🤤","😴","😷","🤒","🤕","🤢","🤮","🤧","🥵","🥶","🥴","😵",
      "😵‍💫","🤯","🤠","🥳","🥸","😎","🤓","🧐","😕","🫤","😟","🙁","☹️","😮","😯","😲",
      "😳","🥺","🥹","😦","😧","😨","😰","😥","😢","😭","😱","😖","😣","😞","😓","😩",
      "😫","🥱","😤","😡","😠","🤬","😈","👿","💀","☠️","💩","🤡","👹","👺","👻","👽",
      "👾","🤖","😺","😸","😹","😻","😼","😽","🙀","😿","😾","🙈","🙉","🙊","💋","💌",
      "💘","💝","💖","💗","💓","💞","💕","💟","❣️","💔","❤️‍🔥","❤️‍🩹","❤️","🧡","💛","💚",
      "💙","💜","🤎","🖤","🤍","💯","💢","💥","💫","💦","💨","🕳️","💬","👁️‍🗨️","🗨️","💭",
      "👋","🤚","🖐️","✋","🖖","🫱","🫲","🫳","🫴","👌","🤌","🤏","✌️","🤞","🫰","🤟",
      "🤘","🤙","👈","👉","👆","🖕","👇","☝️","🫵","👍","👎","✊","👊","🤛","🤜","👏",
      "🙌","🫶","👐","🤲","🤝","🙏","✍️","💅","🤳","💪","🦾","🦵","🦿","🦶","👣","👂",
      "🦻","👃","🧠","🫀","🫁","🦷","🦴","👀","👁️","👅","👄","🫦","👶","🧒","👦","👧",
      "🧑","👱","👨","🧔","👩","🧓","👴","👵","🙍","🙎","🙅","🙆","💁","🙋","🧏","🙇",
      "🤦","🤷","👮","🕵️","💂","👷","🤴","👸","👳","👲","🧕","🤵","👰","🤰","🤱","👼",
    ],
  },
  {
    key: "animals",
    label: "Animals & Nature",
    icon: "🐶",
    emojis: [
      "🐶","🐱","🐭","🐹","🐰","🦊","🐻","🐼","🐻‍❄️","🐨","🐯","🦁","🐮","🐷","🐽","🐸",
      "🐵","🙈","🙉","🙊","🐒","🐔","🐧","🐦","🐤","🐣","🐥","🦆","🦅","🦉","🦇","🐺",
      "🐗","🐴","🦄","🐝","🪱","🐛","🦋","🐌","🐞","🐜","🪰","🪲","🪳","🦟","🦗","🕷️",
      "🕸️","🦂","🐢","🐍","🦎","🦖","🦕","🐙","🦑","🦐","🦞","🦀","🐡","🐠","🐟","🐬",
      "🐳","🐋","🦈","🐊","🐅","🐆","🦓","🦍","🦧","🐘","🦛","🦏","🐪","🐫","🦒","🦘",
      "🐃","🐂","🐄","🐎","🐖","🐏","🐑","🦙","🐐","🦌","🐕","🐩","🦮","🐈","🐈‍⬛","🐓",
      "🦃","🦚","🦜","🦢","🦩","🕊️","🐇","🦝","🦨","🦡","🦦","🦥","🐁","🐀","🐿️","🦔",
      "🐾","🐉","🐲","🌵","🎄","🌲","🌳","🌴","🪵","🌱","🌿","☘️","🍀","🎍","🪴","🎋",
      "🍃","🍂","🍁","🍄","🐚","🪨","🌾","💐","🌷","🌹","🥀","🌺","🌸","🌼","🌻","🌞",
      "🌝","🌛","🌜","🌚","🌕","🌖","🌗","🌘","🌑","🌒","🌓","🌔","🌙","🌎","🌍","🌏",
      "🪐","💫","⭐","🌟","✨","⚡","☄️","💥","🔥","🌪️","🌈","☀️","🌤️","⛅","🌥️","☁️",
      "🌦️","🌧️","⛈️","🌩️","🌨️","❄️","☃️","⛄","🌬️","💨","💧","💦","🌊","🫧","🌫️","🌁",
    ],
  },
  {
    key: "food",
    label: "Food & Drink",
    icon: "🍔",
    emojis: [
      "🍏","🍎","🍐","🍊","🍋","🍌","🍉","🍇","🍓","🫐","🍈","🍒","🍑","🥭","🍍","🥥",
      "🥝","🍅","🍆","🥑","🥦","🥬","🥒","🌶️","🫑","🌽","🥕","🫒","🧄","🧅","🥔","🍠",
      "🥐","🥯","🍞","🥖","🥨","🧀","🥚","🍳","🧈","🥞","🧇","🥓","🥩","🍗","🍖","🦴",
      "🌭","🍔","🍟","🍕","🫓","🥪","🥙","🧆","🌮","🌯","🫔","🥗","🥘","🫕","🥫","🍝",
      "🍜","🍲","🍛","🍣","🍱","🥟","🦪","🍤","🍙","🍚","🍘","🍥","🥠","🥮","🍢","🍡",
      "🍧","🍨","🍦","🥧","🧁","🍰","🎂","🍮","🍭","🍬","🍫","🍿","🍩","🍪","🌰","🥜",
      "🍯","🥛","🍼","🫖","☕","🍵","🧃","🥤","🧋","🍶","🍺","🍻","🥂","🍷","🥃","🍸",
      "🍹","🧉","🍾","🧊","🥄","🍴","🍽️","🥣","🥡","🥢","🧂","🫙","🍡","🍧","🍨","🍳",
    ],
  },
  {
    key: "activity",
    label: "Activity & Travel",
    icon: "⚽",
    emojis: [
      "⚽","🏀","🏈","⚾","🥎","🎾","🏐","🏉","🥏","🎱","🪀","🏓","🏸","🏒","🏑","🥍",
      "🏏","🪃","🥅","⛳","🪁","🏹","🎣","🤿","🥊","🥋","🎽","🛹","🛼","🛷","⛸️","🥌",
      "🎿","⛷️","🏂","🪂","🏋️","🤼","🤸","⛹️","🤺","🤾","🏌️","🏇","🧘","🏄","🏊","🤽",
      "🚣","🧗","🚵","🚴","🏆","🥇","🥈","🥉","🏅","🎖️","🏵️","🎗️","🎫","🎟️","🎪","🤹",
      "🎭","🩰","🎨","🎬","🎤","🎧","🎼","🎹","🥁","🪘","🎷","🎺","🪗","🎸","🪕","🎻",
      "🎲","♟️","🎯","🎳","🎮","🎰","🧩","🚗","🚕","🚙","🚌","🚎","🏎️","🚓","🚑","🚒",
      "🚐","🛻","🚚","🚛","🚜","🦯","🦽","🦼","🛴","🚲","🛵","🏍️","🛺","🚨","🚔","🚍",
      "🚘","🚖","🚡","🚠","🚟","🚃","🚋","🚞","🚝","🚄","🚅","🚈","🚂","🚆","🚇","🚊",
      "🚉","✈️","🛫","🛬","🛩️","💺","🛰️","🚀","🛸","🚁","🛶","⛵","🚤","🛥️","🛳️","⛴️",
      "🚢","⚓","🪝","⛽","🚧","🚦","🚥","🗺️","🗿","🗽","🗼","🏰","🏯","🏟️","🎡","🎢",
      "🎠","⛲","⛱️","🏖️","🏝️","🏜️","🌋","⛰️","🏔️","🗻","🏕️","⛺","🛖","🏠","🏡","🏘️",
    ],
  },
  {
    key: "objects",
    label: "Objects & Symbols",
    icon: "💡",
    emojis: [
      "⌚","📱","📲","💻","⌨️","🖥️","🖨️","🖱️","🖲️","🕹️","🗜️","💽","💾","💿","📀","📼",
      "📷","📸","📹","🎥","📽️","🎞️","📞","☎️","📟","📠","📺","📻","🎙️","🎚️","🎛️","🧭",
      "⏱️","⏲️","⏰","🕰️","⌛","⏳","📡","🔋","🪫","🔌","💡","🔦","🕯️","🪔","🧯","🛢️",
      "💸","💵","💴","💶","💷","🪙","💰","💳","💎","⚖️","🪜","🧰","🪛","🔧","🔨","⚒️",
      "🛠️","⛏️","🪚","🔩","⚙️","🪤","🧱","⛓️","🧲","🔫","💣","🧨","🪓","🔪","🗡️","⚔️",
      "🛡️","🚬","⚰️","🪦","⚱️","🏺","🔮","📿","🧿","💈","⚗️","🔭","🔬","🕳️","🩹","🩺",
      "💊","💉","🩸","🧬","🦠","🧫","🧪","🌡️","🧹","🪠","🧺","🧻","🚽","🚰","🚿","🛁",
      "🛀","🧼","🪥","🪒","🧽","🪣","🧴","🛎️","🔑","🗝️","🚪","🪑","🛋️","🛏️","🛌","🧸",
      "🖼️","🪞","🪟","🛍️","🛒","🎁","🎈","🎏","🎀","🪄","🪅","🎊","🎉","🎎","🏮","🎐",
      "🧧","✉️","📩","📨","📧","💌","📥","📤","📦","🏷️","🪧","📪","📫","📬","📭","📮",
      "📯","📜","📃","📄","📑","🧾","📊","📈","📉","🗒️","🗓️","📆","📅","🗑️","📇","🗃️",
      "🗳️","🗄️","📋","📁","📂","🗂️","🗞️","📰","📓","📔","📒","📕","📗","📘","📙","📚",
      "📖","🔖","🧷","🔗","📎","🖇️","📐","📏","🧮","📌","📍","✂️","🖊️","🖋️","✒️","🖌️",
      "🖍️","📝","✏️","🔍","🔎","🔏","🔐","🔒","🔓","❤️","🧡","💛","💚","💙","💜","🖤",
      "🤍","🤎","✅","☑️","✔️","❌","❎","➕","➖","➗","✖️","🟰","♾️","❓","❔","❗",
      "❕","‼️","⁉️","💯","🔅","🔆","⚠️","🚸","🔱","⚜️","🔰","♻️","✳️","❇️","✴️","🆚",
      "🅰️","🅱️","🆎","🆑","🅾️","🆘","🈚","🈳","🈂️","🛂","🛃","🛄","🛅","🚭","🔞","📵",
      "⭐","🌟","💫","✨","🎵","🎶","➰","➿","〽️","🔚","🔙","🔛","🔝","🔜","🔘","🔴",
      "🟠","🟡","🟢","🔵","🟣","🟤","⚫","⚪","🟥","🟧","🟨","🟩","🟦","🟪","🟫","⬛",
      "⬜","◼️","◻️","◾","◽","▪️","▫️","🔶","🔷","🔸","🔹","🔺","🔻","💠","🔳","🔲",
      "🏁","🚩","🎌","🏴","🏳️","🏳️‍🌈","🏳️‍⚧️","🏴‍☠️","🕐","🕑","🕒","🕓","🕔","🕕","🕖","🕗",
    ],
  },
];

// Common reaction emojis surfaced as a quick row above the full picker.
export const COMMON_REACTIONS = ["👍", "❤️", "😂", "😮", "😢", "🙏"];

// `:shortcode:` → emoji, for the `:`-triggered suggestion popover in the
// composer. Curated to the most-typed names; the picker covers the long tail.
export const EMOJI_SHORTCODES: Record<string, string> = {
  // faces & emotions
  smile: "😄", smiley: "😃", grin: "😁", grinning: "😀", laughing: "😆", satisfied: "😆",
  joy: "😂", rofl: "🤣", lol: "🤣", sweat_smile: "😅", slightly_smiling: "🙂",
  upside_down: "🙃", melting: "🫠", wink: "😉", blush: "😊", innocent: "😇",
  smiling_face_with_hearts: "🥰", heart_eyes: "😍", star_struck: "🤩", kissing_heart: "😘",
  kiss: "😘", kissing: "😗", relaxed: "☺️", yum: "😋", stuck_out_tongue: "😛", tongue: "😛",
  stuck_out_tongue_winking_eye: "😜", zany: "🤪", stuck_out_tongue_closed_eyes: "😝",
  money_mouth: "🤑", hugs: "🤗", hand_over_mouth: "🤭", shushing: "🤫", shush: "🤫",
  thinking: "🤔", salute: "🫡", zipper_mouth: "🤐", raised_eyebrow: "🤨", neutral: "😐",
  expressionless: "😑", no_mouth: "😶", smirk: "😏", unamused: "😒", roll_eyes: "🙄",
  grimacing: "😬", lying: "🤥", relieved: "😌", pensive: "😔", sleepy: "😪", drooling: "🤤",
  sleeping: "😴", mask: "😷", thermometer_face: "🤒", head_bandage: "🤕", nauseated: "🤢",
  vomiting: "🤮", sneezing: "🤧", hot_face: "🥵", cold_face: "🥶", woozy: "🥴", dizzy_face: "😵",
  exploding_head: "🤯", cowboy: "🤠", partying_face: "🥳", party: "🥳", disguised: "🥸",
  sunglasses: "😎", cool: "😎", nerd: "🤓", monocle: "🧐", confused: "😕", worried: "😟",
  slightly_frowning: "🙁", frowning: "☹️", open_mouth: "😮", hushed: "😯", astonished: "😲",
  flushed: "😳", pleading: "🥺", holding_back_tears: "🥹", anguished: "😧", fearful: "😨",
  cold_sweat: "😰", disappointed_relieved: "😥", cry: "😢", sob: "😭", scream: "😱",
  confounded: "😖", persevere: "😣", disappointed: "😞", sweat: "😓", weary: "😩",
  tired_face: "😫", yawning: "🥱", triumph: "😤", angry: "😠", rage: "😡", pout: "😡",
  cursing: "🤬", smiling_imp: "😈", imp: "👿", skull: "💀", skull_crossbones: "☠️",
  poop: "💩", clown: "🤡", ghost: "👻", alien: "👽", space_invader: "👾", robot: "🤖",
  // gestures & people
  wave: "👋", raised_hand: "✋", vulcan: "🖖", ok_hand: "👌", pinched_fingers: "🤌",
  pinch: "🤏", v: "✌️", crossed_fingers: "🤞", love_you: "🤟", metal: "🤘", call_me: "🤙",
  point_left: "👈", point_right: "👉", point_up_2: "👆", middle_finger: "🖕", point_down: "👇",
  point_up: "☝️", thumbsup: "👍", "+1": "👍", thumbsdown: "👎", "-1": "👎", fist: "✊",
  punch: "👊", facepunch: "👊", clap: "👏", raised_hands: "🙌", heart_hands: "🫶",
  open_hands: "👐", palms_up: "🤲", handshake: "🤝", pray: "🙏", thanks: "🙏",
  writing: "✍️", nail_care: "💅", selfie: "🤳", muscle: "💪", flex: "💪", leg: "🦵",
  ear: "👂", nose: "👃", brain: "🧠", tooth: "🦷", eyes: "👀", eye: "👁️", tongue_body: "👅",
  lips: "👄", baby: "👶", shrug: "🤷", facepalm: "🤦", angel: "👼",
  // hearts & symbols
  heart: "❤️", red_heart: "❤️", orange_heart: "🧡", yellow_heart: "💛", green_heart: "💚",
  blue_heart: "💙", purple_heart: "💜", brown_heart: "🤎", black_heart: "🖤", white_heart: "🤍",
  heart_on_fire: "❤️‍🔥", mending_heart: "❤️‍🩹", broken_heart: "💔", two_hearts: "💕",
  revolving_hearts: "💞", heartbeat: "💓", heartpulse: "💗", sparkling_heart: "💖",
  cupid: "💘", gift_heart: "💝", heart_decoration: "💟", love_letter: "💌",
  hundred: "💯", anger: "💢", boom: "💥", collision: "💥", dizzy: "💫", sweat_drops: "💦",
  dash: "💨", speech_balloon: "💬", thought_balloon: "💭", zzz: "💤",
  // nature & weather
  fire: "🔥", star: "⭐", star2: "🌟", sparkles: "✨", zap: "⚡", lightning: "⚡",
  comet: "☄️", rainbow: "🌈", sunny: "☀️", sun: "☀️", partly_sunny: "⛅", cloud: "☁️",
  rain: "🌧️", snow: "❄️", snowman: "⛄", snowflake: "❄️", tornado: "🌪️", droplet: "💧",
  ocean: "🌊", moon: "🌙", full_moon: "🌕", earth: "🌍", globe: "🌎", milky_way: "🌌",
  // plants & animals
  seedling: "🌱", herb: "🌿", four_leaf_clover: "🍀", clover: "☘️", evergreen: "🌲",
  tree: "🌳", palm_tree: "🌴", cactus: "🌵", maple_leaf: "🍁", fallen_leaf: "🍂",
  mushroom: "🍄", bouquet: "💐", tulip: "🌷", rose: "🌹", wilted_flower: "🥀",
  hibiscus: "🌺", cherry_blossom: "🌸", blossom: "🌼", sunflower: "🌻",
  dog: "🐶", cat: "🐱", mouse: "🐭", hamster: "🐹", rabbit: "🐰", fox: "🦊", bear: "🐻",
  panda: "🐼", koala: "🐨", tiger: "🐯", lion: "🦁", cow: "🐮", pig: "🐷", frog: "🐸",
  monkey: "🐵", chicken: "🐔", penguin: "🐧", bird: "🐦", baby_chick: "🐤", duck: "🦆",
  eagle: "🦅", owl: "🦉", bat: "🦇", wolf: "🐺", horse: "🐴", unicorn: "🦄", bee: "🐝",
  bug: "🐛", butterfly: "🦋", snail: "🐌", ladybug: "🐞", ant: "🐜", spider: "🕷️",
  turtle: "🐢", snake: "🐍", octopus: "🐙", squid: "🦑", shrimp: "🦐", crab: "🦀",
  fish: "🐟", tropical_fish: "🐠", blowfish: "🐡", dolphin: "🐬", whale: "🐳", shark: "🦈",
  crocodile: "🐊", elephant: "🐘", giraffe: "🦒", dragon: "🐉", paw_prints: "🐾",
  // food & drink
  apple: "🍎", green_apple: "🍏", pear: "🍐", orange: "🍊", lemon: "🍋", banana: "🍌",
  watermelon: "🍉", grapes: "🍇", strawberry: "🍓", melon: "🍈", cherries: "🍒",
  peach: "🍑", mango: "🥭", pineapple: "🍍", coconut: "🥥", kiwi: "🥝", tomato: "🍅",
  eggplant: "🍆", avocado: "🥑", broccoli: "🥦", corn: "🌽", carrot: "🥕", potato: "🥔",
  bread: "🍞", croissant: "🥐", cheese: "🧀", egg: "🥚", bacon: "🥓", pancakes: "🥞",
  meat: "🍖", poultry: "🍗", hotdog: "🌭", hamburger: "🍔", burger: "🍔", fries: "🍟",
  pizza: "🍕", sandwich: "🥪", taco: "🌮", burrito: "🌯", salad: "🥗", spaghetti: "🍝",
  ramen: "🍜", stew: "🍲", sushi: "🍣", bento: "🍱", curry: "🍛", rice: "🍚",
  dumpling: "🥟", fried_shrimp: "🍤", icecream: "🍦", shaved_ice: "🍧", ice_cream: "🍨",
  doughnut: "🍩", donut: "🍩", cookie: "🍪", cake: "🎂", birthday: "🎂", cupcake: "🧁",
  pie: "🥧", chocolate: "🍫", candy: "🍬", lollipop: "🍭", popcorn: "🍿", honey: "🍯",
  milk: "🥛", baby_bottle: "🍼", coffee: "☕", tea: "🍵", bubble_tea: "🧋", boba: "🧋",
  sake: "🍶", beer: "🍺", beers: "🍻", champagne: "🍾", wine: "🍷", cocktail: "🍸",
  tropical_drink: "🍹", whisky: "🥃", cup_straw: "🥤", ice_cube: "🧊", fork_knife: "🍴",
  // activity & objects
  soccer: "⚽", basketball: "🏀", football: "🏈", baseball: "⚾", tennis: "🎾",
  volleyball: "🏐", rugby: "🏉", pool_8_ball: "🎱", ping_pong: "🏓", badminton: "🏸",
  goal: "🥅", golf: "⛳", bow_and_arrow: "🏹", fishing: "🎣", boxing_glove: "🥊",
  ski: "🎿", trophy: "🏆", first_place: "🥇", second_place: "🥈", third_place: "🥉",
  medal: "🏅", dart: "🎯", bowling: "🎳", video_game: "🎮", joystick: "🕹️", dice: "🎲",
  puzzle: "🧩", chess_pawn: "♟️", art: "🎨", clapper: "🎬", microphone: "🎤", mic: "🎤",
  headphones: "🎧", musical_note: "🎵", notes: "🎶", drum: "🥁", saxophone: "🎷",
  trumpet: "🎺", guitar: "🎸", violin: "🎻", piano: "🎹",
  car: "🚗", taxi: "🚕", bus: "🚌", police_car: "🚓", ambulance: "🚑", fire_engine: "🚒",
  truck: "🚚", tractor: "🚜", bike: "🚲", scooter: "🛵", motorcycle: "🏍️", airplane: "✈️",
  rocket: "🚀", ufo: "🛸", helicopter: "🚁", sailboat: "⛵", speedboat: "🚤", ship: "🚢",
  anchor: "⚓", train: "🚆", metro: "🚇", construction: "🚧", traffic_light: "🚦",
  world_map: "🗺️", statue_of_liberty: "🗽", castle: "🏰", ferris_wheel: "🎡",
  roller_coaster: "🎢", volcano: "🌋", mountain: "⛰️", camping: "🏕️", beach: "🏖️",
  desert_island: "🏝️", house: "🏠", office: "🏢", hospital: "🏥", school: "🏫",
  // tech & objects
  watch: "⌚", iphone: "📱", phone: "📱", computer: "💻", desktop: "🖥️", keyboard: "⌨️",
  printer: "🖨️", camera: "📷", camera_flash: "📸", video_camera: "📹", movie_camera: "🎥",
  tv: "📺", radio: "📻", telephone: "☎️", battery: "🔋", electric_plug: "🔌", bulb: "💡",
  flashlight: "🔦", candle: "🕯️", satellite: "📡", money: "💰", dollar: "💵",
  credit_card: "💳", gem: "💎", diamond: "💎", balance_scale: "⚖️", toolbox: "🧰",
  wrench: "🔧", hammer: "🔨", nut_and_bolt: "🔩", gear: "⚙️", gun: "🔫", bomb: "💣",
  firecracker: "🧨", knife: "🔪", dagger: "🗡️", shield: "🛡️", crystal_ball: "🔮",
  magnet: "🧲", telescope: "🔭", microscope: "🔬", pill: "💊", syringe: "💉", dna: "🧬",
  microbe: "🦠", test_tube: "🧪", broom: "🧹", soap: "🧼", key: "🔑", lock: "🔒",
  unlock: "🔓", door: "🚪", couch: "🛋️", bed: "🛏️", teddy_bear: "🧸", framed_picture: "🖼️",
  shopping_cart: "🛒", shopping_bags: "🛍️", gift: "🎁", balloon: "🎈", ribbon: "🎀",
  tada: "🎉", confetti: "🎊", magic_wand: "🪄", lantern: "🏮", red_envelope: "🧧",
  email: "✉️", envelope: "✉️", incoming_envelope: "📩", inbox: "📥", outbox: "📤",
  package: "📦", label: "🏷️", mailbox: "📮", scroll: "📜", page: "📄", bar_chart: "📊",
  chart_up: "📈", chart_down: "📉", calendar: "📅", clipboard: "📋", file_folder: "📁",
  open_folder: "📂", newspaper: "📰", book: "📖", books: "📚", notebook: "📓",
  bookmark: "🔖", paperclip: "📎", straight_ruler: "📏", abacus: "🧮", pushpin: "📌",
  round_pushpin: "📍", scissors: "✂️", pen: "🖊️", fountain_pen: "🖋️", crayon: "🖍️",
  memo: "📝", pencil: "✏️", mag: "🔍", mag_right: "🔎",
  // marks & symbols
  check: "✅", white_check_mark: "✅", ballot_check: "☑️", heavy_check: "✔️", x: "❌",
  cross_mark: "❌", negative_squared_cross: "❎", heavy_plus: "➕", heavy_minus: "➖",
  heavy_division: "➗", heavy_multiplication: "✖️", infinity: "♾️", question: "❓",
  grey_question: "❔", exclamation: "❗", grey_exclamation: "❕", bangbang: "‼️",
  interrobang: "⁉️", warning: "⚠️", children_crossing: "🚸", recycle: "♻️",
  sparkle: "❇️", trident: "🔱", fleur_de_lis: "⚜️", beginner: "🔰", sos: "🆘",
  no_entry: "⛔", prohibited: "🚫", no_smoking: "🚭", underage: "🔞", radioactive: "☢️",
  red_circle: "🔴", orange_circle: "🟠", yellow_circle: "🟡", green_circle: "🟢",
  blue_circle: "🔵", purple_circle: "🟣", black_circle: "⚫", white_circle: "⚪",
  red_square: "🟥", green_square: "🟩", blue_square: "🟦", large_orange_diamond: "🔶",
  large_blue_diamond: "🔷", diamond_shape: "💠", checkered_flag: "🏁", triangular_flag: "🚩",
  rainbow_flag: "🏳️‍🌈", pirate_flag: "🏴‍☠️", crossed_flags: "🎌", musical_score: "🎼",
  loop: "➿", end: "🔚", back: "🔙", on: "🔛", top: "🔝", soon: "🔜", radio_button: "🔘",
  clock: "🕐", hourglass: "⌛", hourglass_flowing: "⏳", alarm_clock: "⏰", stopwatch: "⏱️",
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
