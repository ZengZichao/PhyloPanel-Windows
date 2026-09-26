pub mod bundle;
pub mod commands;
pub mod runner;
pub mod schema;
pub mod toolpack;

/// Error strings cross the IPC boundary as `E#<key>|<detail>` rather than as prose, so the
/// panel can word them in whatever language the UI is currently showing.
pub fn error(key: &str, detail: impl std::fmt::Display) -> String {
    format!("E#{}|{}", key, detail)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::reflect,
            commands::preview_command,
            commands::run_pipeline,
            commands::bundled_tool,
            commands::write_text,
            commands::write_base64
        ])
        .run(tauri::generate_context!())
        .expect("PhyloPanel failed to start");
}
