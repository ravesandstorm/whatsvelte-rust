fn main() {
    // Klipy GIF API key: KLIPY_API_KEY from the environment, or from a local
    // (gitignored) `.env` file next to Cargo.toml. Baked in at build time,
    // like VITE_KLIPY_API_KEY was for the Svelte build.
    println!("cargo:rerun-if-env-changed=KLIPY_API_KEY");
    println!("cargo:rerun-if-changed=.env");
    let key = std::env::var("KLIPY_API_KEY").ok().or_else(|| {
        std::fs::read_to_string(".env").ok()?.lines().find_map(|l| {
            let (k, v) = l.split_once('=')?;
            matches!(k.trim(), "KLIPY_API_KEY" | "VITE_KLIPY_API_KEY")
                .then(|| v.trim().trim_matches('"').to_string())
        })
    });
    if let Some(key) = key.filter(|k| !k.is_empty()) {
        println!("cargo:rustc-env=KLIPY_API_KEY={key}");
    }

    // Pin the widget style so the app looks the same on every OS (the default
    // would pick the Qt-backed "native" style when Qt happens to be installed).
    let config = slint_build::CompilerConfiguration::new().with_style("fluent".into());
    slint_build::compile_with_config("ui/app.slint", config).expect("failed to compile Slint UI");
}
