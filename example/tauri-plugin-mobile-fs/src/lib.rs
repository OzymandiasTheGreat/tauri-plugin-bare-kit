use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::MobileFs;
#[cfg(mobile)]
use mobile::MobileFs;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the mobile-fs APIs.
pub trait MobileFsExt<R: Runtime> {
    fn mobile_fs(&self) -> &MobileFs<R>;
}

impl<R: Runtime, T: Manager<R>> crate::MobileFsExt<R> for T {
    fn mobile_fs(&self) -> &MobileFs<R> {
        self.state::<MobileFs<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("mobile-fs")
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::get_file_descriptor,
            commands::get_file_name
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let mobile_fs = mobile::init(app, api)?;
            #[cfg(desktop)]
            let mobile_fs = desktop::init(app, api)?;
            app.manage(mobile_fs);
            Ok(())
        })
        .build()
}
