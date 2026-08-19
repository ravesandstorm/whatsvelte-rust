use dioxus::prelude::*;

#[component]
pub fn QrCanvas(code: ReadSignal<String>) -> Element {
    use_effect(use_reactive(&code, move |c| {
        let code_val = c.read().clone();
        let js = format!(
            r#"
            (function() {{
                function tryRender() {{
                    if (window.renderQrCode) {{
                        window.renderQrCode('qr-canvas-holder', {:?});
                    }}
                }}
                tryRender();
                setTimeout(tryRender, 50);
            }})();
            "#,
            code_val
        );
        let _ = document::eval(&js);
    }));

    rsx! {
        div {
            style: "width: 264px; height: 264px; background: #ffffff; padding: 12px; border-radius: 12px; display: flex; flex-direction: column; align-items: center; justify-content: center; box-shadow: 0 4px 12px rgba(0,0,0,0.15); box-sizing: border-box;",
            div {
                id: "qr-canvas-holder",
                style: "width: 240px; height: 240px; display: flex; align-items: center; justify-content: center; overflow: hidden;",
            }
        }
    }
}
