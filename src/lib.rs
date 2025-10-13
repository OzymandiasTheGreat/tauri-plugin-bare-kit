use std::sync::Mutex;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(target_os = "android")]
mod android;
#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod bindings;
mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::BareKit;
#[cfg(mobile)]
use mobile::BareKit;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the bare-kit APIs.
pub trait BareKitExt<R: Runtime> {
    fn bare_kit(&self) -> &Mutex<BareKit<R>>;
}

impl<R: Runtime, T: Manager<R>> crate::BareKitExt<R> for T {
    fn bare_kit(&self) -> &Mutex<BareKit<R>> {
        self.state::<Mutex<BareKit<R>>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("bare-kit")
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::bare_invalidate,
            commands::bare_new,
            commands::bare_start,
            commands::bare_read,
            commands::bare_write,
            commands::bare_update,
            commands::bare_suspend,
            commands::bare_resume,
            commands::bare_terminate,
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let bare_kit = mobile::init(app, api)?;
            #[cfg(desktop)]
            let bare_kit = desktop::init(app, api)?;
            app.manage(bare_kit);
            Ok(())
        })
        .build()
}
