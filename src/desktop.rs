use std::sync::Mutex;

use serde::de::DeserializeOwned;
use tauri::{
    ipc::{InvokeBody, Request, Response},
    plugin::PluginApi,
    AppHandle, Runtime, WebviewWindow,
};

use crate::error::Result;
use crate::models::*;
use crate::module::BareModule;

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Mutex<BareKit<R>>> {
    Ok(Mutex::new(BareKit {
        module: BareModule::new(),
    }))
}

/// Access to the bare-kit APIs.
pub struct BareKit<R: Runtime> {
    module: BareModule<R>,
}

impl<R: Runtime> BareKit<R> {
    pub fn ping(&self, payload: PingRequest) -> Result<PingResponse> {
        Ok(PingResponse {
            value: payload.value,
        })
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

    pub fn read(&mut self, payload: ReadRequest) -> Result<Response> {
        Ok(Response::new(self.module.read(payload.id)?))
    }

    pub fn write(&mut self, payload: Request) -> Result<i32> {
        let InvokeBody::Raw(payload) = payload.body() else {
            return Err("Invalid payload for write request".into());
        };
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
