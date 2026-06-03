use std::collections::HashMap;

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime, WebviewWindow};

use crate::runtime::{plugin::worklet::BareKitWorklet, Result};

#[cfg(target_os = "android")]
use crate::runtime::bare_kit::ffi::ALooper_forThread;
#[cfg(target_os = "android")]
use crate::runtime::plugin::worklet::Looper;

pub(crate) mod commands;
pub(crate) mod models;
mod worklet;

pub struct BareKit<R: Runtime> {
    id: u8,
    #[cfg(target_os = "android")]
    looper: Looper,
    worklets: HashMap<u8, BareKitWorklet<R>>,
}

impl<R: Runtime> BareKit<R> {
    pub(crate) fn new<C: DeserializeOwned>(
        _app: &AppHandle<R>,
        _api: PluginApi<R, C>,
    ) -> Result<Self> {
        Ok(Self {
            id: 0,
            #[cfg(target_os = "android")]
            looper: Looper(ALooper_forThread()),
            worklets: HashMap::new(),
        })
    }

    pub(crate) fn optimize_for_memory(&mut self, enabled: bool) -> Result<()> {
        if self.id > 0 {
            Err("`optimize_for_memory` must be called before any worklets are created.".into())
        } else {
            BareKitWorklet::<R>::optimize_for_memory(enabled);

            Ok(())
        }
    }

    pub(crate) fn invalidate(&mut self) -> Result<()> {
        for worklet in self.worklets.values_mut() {
            worklet.terminate();
        }

        self.worklets.clear();

        Ok(())
    }

    pub(crate) fn new_worklet(
        &mut self,
        app: AppHandle<R>,
        window: WebviewWindow<R>,
        memory_limit: usize,
        assets: Option<String>,
        on_poll: u32,
        on_suspend: u32,
        on_wakeup: u32,
        on_idle: u32,
        on_resume: u32,
    ) -> Result<u8> {
        self.id += 1;

        let worklet = BareKitWorklet::new(
            self.id,
            app,
            window,
            memory_limit,
            assets,
            on_poll,
            on_suspend,
            on_wakeup,
            on_idle,
            on_resume,
        );

        self.worklets.insert(self.id, worklet);

        Ok(self.id)
    }

    pub(crate) fn start_file(&mut self, id: u8, filename: String, args: Vec<String>) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            worklet.start(filename, None, args);

            #[cfg(target_os = "android")]
            worklet.set_looper(self.looper);

            Ok(())
        } else {
            Err(format!("No worklet found for ID {id}").into())
        }
    }

    pub(crate) fn start_utf8(
        &mut self,
        id: u8,
        filename: String,
        source: String,
        args: Vec<String>,
    ) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            let source = Vec::from(source);

            worklet.start(filename, Some(source), args);

            #[cfg(target_os = "android")]
            worklet.set_looper(self.looper);

            Ok(())
        } else {
            Err(format!("No worklet found for ID {id}").into())
        }
    }

    pub(crate) fn start_bytes(
        &mut self,
        id: u8,
        filename: String,
        source: Vec<u8>,
        args: Vec<String>,
    ) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            worklet.start(filename, Some(source), args);

            #[cfg(target_os = "android")]
            worklet.set_looper(self.looper);

            Ok(())
        } else {
            Err(format!("No worklet found for ID {id}").into())
        }
    }

    pub(crate) fn read(&mut self, id: u8) -> Result<Option<Vec<u8>>> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            Ok(worklet.read())
        } else {
            Err(format!("No worklet found for ID {id}").into())
        }
    }

    pub(crate) fn write(&mut self, id: u8, data: Option<Vec<u8>>) -> Result<i32> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            Ok(worklet.write(data))
        } else {
            Err(format!("No worklet found for ID {id}").into())
        }
    }

    pub(crate) fn update(&mut self, id: u8, readable: bool, writable: bool) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            worklet.update(readable, writable);

            Ok(())
        } else {
            Err(format!("No worklet found for ID {id}").into())
        }
    }

    pub(crate) fn suspend(&mut self, id: u8, linger: i32) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            worklet.suspend(linger);

            Ok(())
        } else {
            Err(format!("No worklet found for ID {id}").into())
        }
    }

    pub(crate) fn resume(&mut self, id: u8) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            worklet.resume();

            Ok(())
        } else {
            Err(format!("No worklet found for ID {id}").into())
        }
    }

    pub(crate) fn wakeup(&mut self, id: u8, deadline: i32) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            worklet.wakeup(deadline);

            Ok(())
        } else {
            Err(format!("No worklet found for ID {id}").into())
        }
    }

    pub(crate) fn terminate(&mut self, id: u8) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            worklet.terminate();

            Ok(())
        } else {
            Err(format!("No worklet found for ID {id}").into())
        }
    }
}
