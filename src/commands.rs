use tauri::{command, AppHandle, Runtime, WebviewWindow};

use crate::models::*;
use crate::BareKitExt;
use crate::Result;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.bare_kit().lock().unwrap().ping(payload)
}

#[command]
pub(crate) fn bare_invalidate<R: Runtime>(app: AppHandle<R>, _payload: ()) -> Result<()> {
    app.bare_kit().lock().unwrap().invalidate()
}

#[command]
pub(crate) fn bare_new<R: Runtime>(
    app: AppHandle<R>,
    window: WebviewWindow<R>,
    payload: NewRequest,
) -> Result<WorkletResponse> {
    app.bare_kit().lock().unwrap().new(window, payload)
}

#[command]
pub(crate) fn bare_start<R: Runtime>(app: AppHandle<R>, payload: StartRequest) -> Result<()> {
    app.bare_kit().lock().unwrap().start(payload)
}

#[command]
pub(crate) fn bare_read<R: Runtime>(
    app: AppHandle<R>,
    payload: WorkletRequest,
) -> Result<ReadResponse> {
    app.bare_kit().lock().unwrap().read(payload)
}

#[command]
pub(crate) fn bare_write<R: Runtime>(
    app: AppHandle<R>,
    payload: WriteRequest,
) -> Result<WorkletResponse> {
    app.bare_kit().lock().unwrap().write(payload)
}

#[command]
pub(crate) fn bare_update<R: Runtime>(app: AppHandle<R>, payload: UpdateRequest) -> Result<()> {
    app.bare_kit().lock().unwrap().update(payload)
}

#[command]
pub(crate) fn bare_suspend<R: Runtime>(app: AppHandle<R>, payload: SuspendRequest) -> Result<()> {
    app.bare_kit().lock().unwrap().suspend(payload)
}

#[command]
pub(crate) fn bare_resume<R: Runtime>(app: AppHandle<R>, payload: WorkletRequest) -> Result<()> {
    app.bare_kit().lock().unwrap().resume(payload)
}

#[command]
pub(crate) fn bare_terminate<R: Runtime>(app: AppHandle<R>, payload: WorkletRequest) -> Result<()> {
    app.bare_kit().lock().unwrap().terminate(payload)
}
