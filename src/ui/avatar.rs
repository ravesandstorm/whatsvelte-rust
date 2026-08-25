use dioxus::prelude::*;

const AVATAR_COLORS: &[&str] = &[
    "#00a884",
    "#008069",
    "#1f8a70",
    "#0284c7",
    "#2563eb",
    "#7c3aed",
    "#9333ea",
    "#c026d3",
    "#db2777",
    "#d97706",
    "#059669",
    "#0d9488",
];

fn get_avatar_color(jid: &str) -> &'static str {
    let hash: usize = jid.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize));
    AVATAR_COLORS[hash % AVATAR_COLORS.len()]
}

fn extract_initials(name: &str) -> Option<String> {
    let parts: Vec<&str> = name.split_whitespace().filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return None;
    }

    let mut initials = String::new();
    for part in parts.iter().take(2) {
        if let Some(ch) = part.chars().find(|c| c.is_alphabetic()) {
            initials.push(ch.to_ascii_uppercase());
        }
    }

    if initials.is_empty() {
        None
    } else {
        Some(initials)
    }
}

#[component]
pub fn Avatar(
    jid: String,
    name: Option<String>,
    #[props(default = None)] picture_url: Option<String>,
    #[props(default = 48)] size: u32,
) -> Element {
    let bg_color = get_avatar_color(&jid);
    let is_group = jid.ends_with("@g.us");
    let is_status = jid == "status@broadcast";
    let font_size = (size as f32 * 0.42).round() as u32;

    let initials = name.as_deref().and_then(extract_initials);

    rsx! {
        div {
            style: "width: {size}px; height: {size}px; border-radius: 50%; background-color: {bg_color}; display: flex; align-items: center; justify-content: center; font-weight: 600; color: #ffffff; font-size: {font_size}px; flex-shrink: 0; position: relative; overflow: hidden; user-select: none; box-shadow: 0 1px 3px rgba(0,0,0,0.2);",
            if let Some(url) = picture_url.as_ref().filter(|u| !u.is_empty()) {
                img {
                    src: "{url}",
                    alt: "Profile photo",
                    style: "width: 100%; height: 100%; object-fit: cover; border-radius: 50%; display: block;",
                }
            } else if is_group {
                // Group Icon
                svg {
                    view_box: "0 0 24 24",
                    width: "{size * 6 / 10}px",
                    height: "{size * 6 / 10}px",
                    fill: "#ffffff",
                    path {
                        d: "M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"
                    }
                }
            } else if is_status {
                // Status Broadcast Icon
                svg {
                    view_box: "0 0 24 24",
                    width: "{size * 6 / 10}px",
                    height: "{size * 6 / 10}px",
                    fill: "#ffffff",
                    path {
                        d: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"
                    }
                }
            } else if let Some(inits) = initials {
                "{inits}"
            } else {
                // Default User Silhouette Icon
                svg {
                    view_box: "0 0 24 24",
                    width: "{size * 6 / 10}px",
                    height: "{size * 6 / 10}px",
                    fill: "#ffffff",
                    path {
                        d: "M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"
                    }
                }
            }
        }
    }
}
