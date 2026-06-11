#[cfg(target_os = "android")]
use base64::prelude::{Engine, BASE64_STANDARD};

#[cfg(not(target_os = "android"))]
use tauri::ipc::{InvokeBody, Request, Response};

use tauri::{command, AppHandle, Runtime, WebviewWindow};

use crate::runtime::plugin::models::*;
use crate::runtime::BareKitExt;
use crate::runtime::Result;

#[command]
pub fn bare_optimize_for_memory<R: Runtime>(
    app: AppHandle<R>,
    payload: OptimizeForMemoryRequest,
) -> Result<()> {
    app.bare_kit()
        .lock()
        .unwrap()
        .optimize_for_memory(payload.enabled)
}

#[command]
pub fn bare_new_worklet<R: Runtime>(
    app: AppHandle<R>,
    window: WebviewWindow<R>,
    payload: NewWorkletRequest,
) -> Result<u8> {
    app.clone().bare_kit().lock().unwrap().new_worklet(
        app,
        window,
        payload.memory_limit,
        payload.assets,
        payload.on_poll,
        payload.on_suspend,
        payload.on_wakeup,
        payload.on_idle,
        payload.on_resume,
    )
}

#[command]
pub fn bare_start_file<R: Runtime>(app: AppHandle<R>, payload: StartFileRequest) -> Result<()> {
    app.bare_kit()
        .lock()
        .unwrap()
        .start_file(payload.id, payload.filename, payload.args)
}

#[command]
pub fn bare_start_utf8<R: Runtime>(app: AppHandle<R>, payload: StartUTF8Request) -> Result<()> {
    app.bare_kit().lock().unwrap().start_utf8(
        payload.id,
        payload.filename,
        payload.source,
        payload.args,
    )
}

#[command]
pub fn bare_start_bytes<R: Runtime>(app: AppHandle<R>, payload: StartBytesRequest) -> Result<()> {
    app.bare_kit().lock().unwrap().start_bytes(
        payload.id,
        payload.filename,
        payload.source,
        payload.args,
    )
}

#[cfg(not(target_os = "android"))]
#[command]
pub fn bare_read<R: Runtime>(app: AppHandle<R>, payload: ReadRequest) -> Result<Response> {
    let data = app.bare_kit().lock().unwrap().read(payload.id).unwrap();

    if let Some(data) = data {
        Ok(Response::new(data))
    } else {
        Ok(Response::new(vec![]))
    }
}

#[cfg(target_os = "android")]
#[command]
pub fn bare_read<R: Runtime>(app: AppHandle<R>, payload: ReadRequest) -> Result<Option<String>> {
    let data = app.bare_kit().lock().unwrap().read(payload.id).unwrap();

    if let Some(data) = data {
        Ok(BASE64_STANDARD.encode(data))
    } else {
        Ok(None)
    }
}

#[cfg(not(target_os = "android"))]
#[command]
pub fn bare_write<R: Runtime>(app: AppHandle<R>, payload: Request<'_>) -> Result<i32> {
    let InvokeBody::Raw(payload) = payload.body() else {
        return Err("Invalid payload for write request".into());
    };

    let id = payload[0];

    if payload.len() == 1 {
        app.bare_kit().lock().unwrap().write(id, None)
    } else {
        let data = payload[1..].to_vec();
        app.bare_kit().lock().unwrap().write(id, Some(data))
    }
}

#[cfg(target_os = "android")]
#[command]
pub fn bare_write<R: Runtime>(app: AppHandle<R>, payload: String) -> Result<i32> {
    let payload = BASE64_STANDARD.decode(payload)?;
    let id = payload[0];

    if payload.len() == 1 {
        app.bare_kit().lock().unwrap().write(id, None)
    } else {
        let data = payload[1..].to_vec();
        app.bare_kit().lock().unwrap().write(id, Some(data))
    }
}

#[command]
pub fn bare_update<R: Runtime>(app: AppHandle<R>, payload: UpdateRequest) -> Result<()> {
    app.bare_kit()
        .lock()
        .unwrap()
        .update(payload.id, payload.readable, payload.writable)
}

#[command]
pub fn bare_suspend<R: Runtime>(app: AppHandle<R>, payload: SuspendRequest) -> Result<()> {
    app.bare_kit()
        .lock()
        .unwrap()
        .suspend(payload.id, payload.linger)
}

#[command]
pub fn bare_resume<R: Runtime>(app: AppHandle<R>, payload: ResumeRequest) -> Result<()> {
    app.bare_kit().lock().unwrap().resume(payload.id)
}

#[command]
pub fn bare_wakeup<R: Runtime>(app: AppHandle<R>, payload: WakeupRequest) -> Result<()> {
    app.bare_kit()
        .lock()
        .unwrap()
        .wakeup(payload.id, payload.deadline)
}

#[command]
pub fn bare_terminate<R: Runtime>(app: AppHandle<R>, payload: TerminateRequest) -> Result<()> {
    app.bare_kit().lock().unwrap().terminate(payload.id)
}
