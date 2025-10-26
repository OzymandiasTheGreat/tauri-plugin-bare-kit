#[cfg(not(target_os = "android"))]
use tauri::ipc::{Request, Response};

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
pub(crate) fn bare_invalidate<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    app.bare_kit().lock().unwrap().invalidate()
}

#[command]
pub(crate) fn bare_init<R: Runtime>(
    app: AppHandle<R>,
    window: WebviewWindow<R>,
    payload: InitRequest,
) -> Result<u8> {
    app.bare_kit().lock().unwrap().init(window, payload)
}

#[command]
pub(crate) fn bare_start_file<R: Runtime>(
    app: AppHandle<R>,
    payload: StartFileRequest,
) -> Result<()> {
    app.bare_kit().lock().unwrap().start_file(payload)
}

#[command]
pub(crate) fn bare_start_utf8<R: Runtime>(
    app: AppHandle<R>,
    payload: StartUTF8Request,
) -> Result<()> {
    app.bare_kit().lock().unwrap().start_utf8(payload)
}

#[command]
pub(crate) fn bare_start_bytes<R: Runtime>(
    app: AppHandle<R>,
    payload: StartBytesRequest,
) -> Result<()> {
    app.bare_kit().lock().unwrap().start_bytes(payload)
}

#[cfg(not(target_os = "android"))]
#[command]
pub(crate) fn bare_read<R: Runtime>(app: AppHandle<R>, payload: ReadRequest) -> Result<Response> {
    app.bare_kit().lock().unwrap().read(payload)
}

#[cfg(target_os = "android")]
#[command]
pub(crate) fn bare_read<R: Runtime>(app: AppHandle<R>, payload: ReadRequest) -> Result<String> {
    app.bare_kit().lock().unwrap().read(payload)
}

#[cfg(not(target_os = "android"))]
#[command]
pub(crate) fn bare_write<R: Runtime>(app: AppHandle<R>, payload: Request<'_>) -> Result<i32> {
    app.bare_kit().lock().unwrap().write(payload)
}

#[cfg(target_os = "android")]
#[command]
pub(crate) fn bare_write<R: Runtime>(app: AppHandle<R>, payload: String) -> Result<i32> {
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
pub(crate) fn bare_resume<R: Runtime>(app: AppHandle<R>, payload: ResumeRequest) -> Result<()> {
    app.bare_kit().lock().unwrap().resume(payload)
}

#[command]
pub(crate) fn bare_wakeup<R: Runtime>(app: AppHandle<R>, payload: WakeupRequest) -> Result<()> {
    app.bare_kit().lock().unwrap().wakeup(payload)
}

#[command]
pub(crate) fn bare_terminate<R: Runtime>(
    app: AppHandle<R>,
    payload: TerminateRequest,
) -> Result<()> {
    app.bare_kit().lock().unwrap().terminate(payload)
}
