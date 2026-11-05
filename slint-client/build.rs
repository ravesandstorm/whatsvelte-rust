fn main() {
    // Pin the widget style so the app looks the same on every OS (the default
    // would pick the Qt-backed "native" style when Qt happens to be installed).
    let config = slint_build::CompilerConfiguration::new().with_style("fluent".into());
    slint_build::compile_with_config("ui/app.slint", config).expect("failed to compile Slint UI");
}
