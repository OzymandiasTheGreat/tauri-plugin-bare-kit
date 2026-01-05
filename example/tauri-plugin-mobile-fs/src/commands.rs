use tauri::{command, AppHandle, Runtime};

use crate::models::*;
use crate::MobileFsExt;
use crate::Result;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.mobile_fs().ping(payload)
}

#[command]
pub(crate) async fn get_file_descriptor<R: Runtime>(
    app: AppHandle<R>,
    payload: GetFileDescriptorRequest,
) -> Result<GetFileDescriptorResponse> {
    app.mobile_fs().get_file_descriptor(payload)
}

#[command]
pub(crate) async fn get_file_name<R: Runtime>(
    app: AppHandle<R>,
    payload: GetFileNameRequest,
) -> Result<GetFileNameResponse> {
    app.mobile_fs().get_file_name(payload)
}
