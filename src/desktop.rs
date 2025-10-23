use std::sync::Mutex;

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime, WebviewWindow};

use crate::models::*;

#[cfg(target_vendor = "apple")]
use crate::apple::BareKitModule;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Mutex<BareKit<R>>> {
    Ok(Mutex::new(BareKit {
        _app: app.clone(),
        module: BareKitModule::init(),
    }))
}

/// Access to the bare-kit APIs.
pub struct BareKit<R: Runtime> {
    _app: AppHandle<R>,
    module: BareKitModule,
}

impl<R: Runtime> BareKit<R> {
    pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
        Ok(PingResponse {
            value: payload.value,
        })
    }

    pub fn invalidate(&mut self) -> crate::Result<()> {
        self.module.invalidate::<R>();
        Ok(())
    }

    pub fn new(
        &mut self,
        window: WebviewWindow<R>,
        payload: NewRequest,
    ) -> crate::Result<WorkletResponse> {
        let id = self.module.new(
            window,
            payload.memory_limit,
            payload.assets,
            payload.on_poll,
        );
        Ok(WorkletResponse { data: id })
    }

    pub fn start(&mut self, payload: StartRequest) -> crate::Result<()> {
        self.module
            .start(payload.id, payload.filename, payload.source, payload.argv);
        Ok(())
    }

    pub fn read(&mut self, payload: WorkletRequest) -> crate::Result<ReadResponse> {
        let data = self.module.read(payload.id);
        Ok(ReadResponse { data })
    }

    pub fn write(&mut self, payload: WriteRequest) -> crate::Result<WorkletResponse> {
        let data = self.module.write(payload.id, payload.data);
        Ok(WorkletResponse { data })
    }

    pub fn update(&mut self, payload: UpdateRequest) -> crate::Result<()> {
        self.module
            .update::<R>(payload.id, payload.readable, payload.writable);
        Ok(())
    }

    pub fn suspend(&mut self, payload: SuspendRequest) -> crate::Result<()> {
        self.module.suspend(payload.id, payload.linger);
        Ok(())
    }

    pub fn resume(&mut self, payload: WorkletRequest) -> crate::Result<()> {
        self.module.resume(payload.id);
        Ok(())
    }

    pub fn terminate(&mut self, payload: WorkletRequest) -> crate::Result<()> {
        self.module.terminate::<R>(payload.id);
        Ok(())
    }
}
