#[cfg(target_os = "android")]
use base64::prelude::{Engine, BASE64_STANDARD};

#[cfg(not(target_os = "android"))]
use tauri::ipc::{InvokeBody, Request, Response};

use tauri::{command, AppHandle, Runtime, WebviewWindow};

use crate::models::*;
use crate::BareKitExt;
use crate::Result;

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
    app.bare_kit().lock().unwrap().init(
        payload.memory_limit,
        payload.assets,
        window,
        payload.on_poll,
    )
}

#[command]
pub(crate) fn bare_start_file<R: Runtime>(
    app: AppHandle<R>,
    payload: StartFileRequest,
) -> Result<()> {
    app.bare_kit()
        .lock()
        .unwrap()
        .start_file(payload.id, payload.filename, payload.args)
}

#[command]
pub(crate) fn bare_start_utf8<R: Runtime>(
    app: AppHandle<R>,
    payload: StartUTF8Request,
) -> Result<()> {
    app.bare_kit().lock().unwrap().start_utf8(
        payload.id,
        payload.filename,
        payload.source,
        payload.args,
    )
}

#[command]
pub(crate) fn bare_start_bytes<R: Runtime>(
    app: AppHandle<R>,
    payload: StartBytesRequest,
) -> Result<()> {
    app.bare_kit().lock().unwrap().start_bytes(
        payload.id,
        payload.filename,
        payload.source,
        payload.args,
    )
}

#[cfg(not(target_os = "android"))]
#[command]
pub(crate) fn bare_read<R: Runtime>(app: AppHandle<R>, payload: ReadRequest) -> Result<Response> {
    Ok(Response::new(
        app.bare_kit().lock().unwrap().read(payload.id)?,
    ))
}

#[cfg(target_os = "android")]
#[command]
pub(crate) fn bare_read<R: Runtime>(app: AppHandle<R>, payload: ReadRequest) -> Result<String> {
    Ok(BASE64_STANDARD.encode(app.bare_kit().lock().unwrap().read(payload.id)?))
}

#[cfg(not(target_os = "android"))]
#[command]
pub(crate) fn bare_write<R: Runtime>(app: AppHandle<R>, payload: Request<'_>) -> Result<i32> {
    let InvokeBody::Raw(payload) = payload.body() else {
        return Err("Invalid payload for write request".into());
    };
    let id = payload[0];
    let data = payload[1..].to_vec();
    app.bare_kit().lock().unwrap().write(id, data)
}

#[cfg(target_os = "android")]
#[command]
pub(crate) fn bare_write<R: Runtime>(app: AppHandle<R>, payload: String) -> Result<i32> {
    let payload = BASE64_STANDARD.decode(payload)?;
    let id = payload[0];
    let data = payload[1..].to_vec();
    app.bare_kit().lock().unwrap().write(id, data)
}

#[command]
pub(crate) fn bare_update<R: Runtime>(app: AppHandle<R>, payload: UpdateRequest) -> Result<()> {
    app.bare_kit()
        .lock()
        .unwrap()
        .update(payload.id, payload.readable, payload.writable)
}

#[command]
pub(crate) fn bare_suspend<R: Runtime>(app: AppHandle<R>, payload: SuspendRequest) -> Result<()> {
    app.bare_kit()
        .lock()
        .unwrap()
        .suspend(payload.id, payload.linger)
}

#[command]
pub(crate) fn bare_resume<R: Runtime>(app: AppHandle<R>, payload: ResumeRequest) -> Result<()> {
    app.bare_kit().lock().unwrap().resume(payload.id)
}

#[command]
pub(crate) fn bare_wakeup<R: Runtime>(app: AppHandle<R>, payload: WakeupRequest) -> Result<()> {
    app.bare_kit()
        .lock()
        .unwrap()
        .wakeup(payload.id, payload.deadline)
}

#[command]
pub(crate) fn bare_terminate<R: Runtime>(
    app: AppHandle<R>,
    payload: TerminateRequest,
) -> Result<()> {
    app.bare_kit().lock().unwrap().terminate(payload.id)
}
