use std::sync::Mutex;

use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime, WebviewWindow,
};

use crate::models::*;

#[cfg(target_os = "android")]
use crate::android::BareKitModule;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_bare_kit);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Mutex<BareKit<R>>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin("sh.quince.bare_kit", "ExamplePlugin")?;
    let module = BareKitModule::init();
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_bare_kit)?;
    Ok(Mutex::new(BareKit { handle, module }))
}

/// Access to the bare-kit APIs.
pub struct BareKit<R: Runtime> {
    handle: PluginHandle<R>,
    module: BareKitModule,
}

impl<R: Runtime> BareKit<R> {
    pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
        self.handle
            .run_mobile_plugin("ping", payload)
            .map_err(Into::into)
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
