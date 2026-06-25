#[cfg(debug_assertions)]
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    #[cfg(not(target_os = "android"))]
    let builder = builder.plugin(tauri_plugin_webdriver::init());

    let builder = builder.plugin(tauri_plugin_bare_kit::init()).setup(|_app| {
        #[cfg(debug_assertions)]
        if let Some(window) = _app.get_webview_window("main") {
            window.open_devtools();
        }

        Ok(())
    });

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
