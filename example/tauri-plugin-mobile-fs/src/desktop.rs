use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<MobileFs<R>> {
    Ok(MobileFs(app.clone()))
}

/// Access to the mobile-fs APIs.
pub struct MobileFs<R: Runtime>(AppHandle<R>);

impl<R: Runtime> MobileFs<R> {
    pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
        Ok(PingResponse {
            value: payload.value,
        })
    }

    pub fn get_file_descriptor(
        &self,
        _payload: GetFileDescriptorRequest,
    ) -> crate::Result<GetFileDescriptorResponse> {
        Ok(GetFileDescriptorResponse { fd: None })
    }

    pub fn get_file_name(
        &self,
        _payload: GetFileNameRequest,
    ) -> crate::Result<GetFileNameResponse> {
        Ok(GetFileNameResponse { filename: None })
    }
}
