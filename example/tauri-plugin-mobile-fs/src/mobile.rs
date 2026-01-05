use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_mobile_fs);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<MobileFs<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin("sh.quince.mobilefs", "MobileFSPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_mobile_fs)?;
    Ok(MobileFs(handle))
}

/// Access to the mobile-fs APIs.
pub struct MobileFs<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> MobileFs<R> {
    pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
        self.0
            .run_mobile_plugin("ping", payload)
            .map_err(Into::into)
    }

    pub fn get_file_descriptor(
        &self,
        payload: GetFileDescriptorRequest,
    ) -> crate::Result<GetFileDescriptorResponse> {
        self.0
            .run_mobile_plugin("getFileDescriptor", payload)
            .map_err(Into::into)
    }

    pub fn get_file_name(&self, payload: GetFileNameRequest) -> crate::Result<GetFileNameResponse> {
        self.0
            .run_mobile_plugin("getFileName", payload)
            .map_err(Into::into)
    }
}
