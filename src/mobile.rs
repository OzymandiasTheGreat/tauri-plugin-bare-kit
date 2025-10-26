use std::sync::Mutex;

#[cfg(not(target_os = "android"))]
use tauri::ipc::{InvokeBody, Request, Response};

#[cfg(target_os = "android")]
use base64::{prelude::BASE64_STANDARD, Engine};

use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime, WebviewWindow,
};

use crate::error::Result;
use crate::models::*;
use crate::module::BareModule;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_bare_kit);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<Mutex<BareKit<R>>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin("sh.quince.bare_kit", "ExamplePlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_bare_kit)?;
    let module = BareModule::new();
    Ok(Mutex::new(BareKit { handle, module }))
}

/// Access to the bare-kit APIs.
pub struct BareKit<R: Runtime> {
    handle: PluginHandle<R>,
    module: BareModule<R>,
}

impl<R: Runtime> BareKit<R> {
    pub fn ping(&self, payload: PingRequest) -> Result<PingResponse> {
        self.handle
            .run_mobile_plugin("ping", payload)
            .map_err(Into::into)
    }

    pub fn invalidate(&mut self) -> Result<()> {
        self.module.invalidate();
        Ok(())
    }

    pub fn init(&mut self, window: WebviewWindow<R>, payload: InitRequest) -> Result<u8> {
        Ok(self.module.init(
            payload.memory_limit,
            payload.assets,
            window,
            payload.on_poll,
        ))
    }

    pub fn start_file(&mut self, payload: StartFileRequest) -> Result<()> {
        self.module
            .start_file(payload.id, payload.filename, payload.args)
    }

    pub fn start_utf8(&mut self, payload: StartUTF8Request) -> Result<()> {
        self.module
            .start_utf8(payload.id, payload.filename, payload.source, payload.args)
    }

    pub fn start_bytes(&mut self, payload: StartBytesRequest) -> Result<()> {
        self.module
            .start_bytes(payload.id, payload.filename, payload.source, payload.args)
    }

    #[cfg(not(target_os = "android"))]
    pub fn read(&mut self, payload: ReadRequest) -> Result<Response> {
        Ok(Response::new(self.module.read(payload.id)?))
    }

    #[cfg(target_os = "android")]
    pub fn read(&mut self, payload: ReadRequest) -> Result<String> {
        Ok(BASE64_STANDARD.encode(self.module.read(payload.id)?))
    }

    #[cfg(not(target_os = "android"))]
    pub fn write(&mut self, payload: Request) -> Result<i32> {
        let InvokeBody::Raw(payload) = payload.body() else {
            return Err("Invalid payload for write request".into());
        };
        let id = payload[0];
        let data = payload[1..].to_vec();
        self.module.write(id, data)
    }

    #[cfg(target_os = "android")]
    pub fn write(&mut self, payload: String) -> Result<i32> {
        let payload = BASE64_STANDARD.decode(payload).unwrap();
        let id = payload[0];
        let data = payload[1..].to_vec();
        self.module.write(id, data)
    }

    pub fn update(&mut self, payload: UpdateRequest) -> Result<()> {
        self.module
            .update(payload.id, payload.readable, payload.writable)
    }

    pub fn suspend(&mut self, payload: SuspendRequest) -> Result<()> {
        self.module.suspend(payload.id, payload.linger)
    }

    pub fn resume(&mut self, payload: ResumeRequest) -> Result<()> {
        self.module.resume(payload.id)
    }

    pub fn wakeup(&mut self, payload: WakeupRequest) -> Result<()> {
        self.module.wakeup(payload.id, payload.deadline)
    }

    pub fn terminate(&mut self, payload: TerminateRequest) -> Result<()> {
        self.module.terminate(payload.id)
    }
}
