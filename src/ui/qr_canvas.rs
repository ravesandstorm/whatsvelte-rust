use dioxus::prelude::*;

#[component]
pub fn QrCanvas(code: String) -> Element {
    rsx! {
        div {
            style: "width: 264px; height: 264px; background: #ffffff; padding: 12px; border-radius: 12px; display: flex; flex-direction: column; align-items: center; justify-content: center; box-shadow: 0 4px 12px rgba(0,0,0,0.15);",
            svg {
                width: "240",
                height: "240",
                view_box: "0 0 100 100",
                rect { width: "100", height: "100", fill: "#ffffff" }
                // QR code matrix placeholder rendering
                text {
                    x: "50",
                    y: "50",
                    dominant_baseline: "middle",
                    text_anchor: "middle",
                    fill: "#111b21",
                    font_size: "6",
                    font_family: "sans-serif",
                    "Scan with WhatsApp"
                }
            }
        }
    }
}
