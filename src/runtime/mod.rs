use std::{env::consts, sync::Mutex};

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, RunEvent, Runtime,
};

pub use error::{Error, Result};
pub use plugin::models::*;

use plugin::{commands, BareKit};

pub mod bare_kit;
pub mod error;

mod plugin;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the bare-kit APIs.
pub trait BareKitExt<R: Runtime> {
    fn bare_kit(&self) -> &Mutex<BareKit<R>>;
}

impl<R: Runtime, T: Manager<R>> BareKitExt<R> for T {
    fn bare_kit(&self) -> &Mutex<BareKit<R>> {
        self.state::<Mutex<BareKit<R>>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let init_js: String = format!("Object.defineProperty(window, '__TAURI_BARE_KIT_PLUGIN_INTERNALS__', {{ value: {{ platform: '{}' }} }})", consts::OS);

    Builder::new("bare-kit")
        .js_init_script(init_js)
        .invoke_handler(tauri::generate_handler![
            commands::bare_optimize_for_memory,
            commands::bare_new_worklet,
            commands::bare_start_file,
            commands::bare_start_utf8,
            commands::bare_start_bytes,
            commands::bare_read,
            commands::bare_write,
            commands::bare_update,
            commands::bare_suspend,
            commands::bare_resume,
            commands::bare_wakeup,
            commands::bare_terminate,
        ])
        .setup(|app, api| {
            let bare_kit = BareKit::new(app, api)?;
            app.manage(Mutex::new(bare_kit));
            Ok(())
        })
        .on_event(|app, event| match event {
            RunEvent::ExitRequested { .. } => app.bare_kit().lock().unwrap().invalidate().unwrap(),
            _ => (),
        })
        .build()
}
