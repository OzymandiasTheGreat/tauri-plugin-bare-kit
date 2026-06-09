#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_webdriver::init())
        .plugin(tauri_plugin_bare_kit::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
