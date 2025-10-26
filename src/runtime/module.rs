use std::collections::HashMap;

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime, WebviewWindow};

use crate::error::Result;
#[cfg(target_os = "android")]
use crate::runtime::ffi::ALooper_forThread;
use crate::runtime::worklet::{BareWorklet, Looper};

pub struct BareKit<R: Runtime> {
    id: u8,
    worklets: HashMap<u8, BareWorklet<R>>,
    looper: Looper,
}

impl<R: Runtime> BareKit<R> {
    pub fn new<C: DeserializeOwned>(_app: &AppHandle<R>, _api: PluginApi<R, C>) -> Result<Self> {
        #[cfg(not(target_os = "android"))]
        let looper = Looper();
        #[cfg(target_os = "android")]
        let looper = unsafe {
            let looper = Looper(ALooper_forThread());
            looper
        };

        Ok(Self {
            id: 0,
            worklets: HashMap::new(),
            looper,
        })
    }

    pub fn invalidate(&mut self) -> Result<()> {
        for (_, worklet) in &mut self.worklets {
            worklet.terminate();
        }
        self.worklets.clear();

        Ok(())
    }

    pub fn init(
        &mut self,
        memory_limit: usize,
        assets: Option<String>,
        window: WebviewWindow<R>,
        callback_id: u32,
    ) -> Result<u8> {
        self.id += 1;

        let worklet = BareWorklet::new(self.id, memory_limit, assets, window, callback_id);
        self.worklets.insert(self.id, worklet);

        Ok(self.id)
    }

    pub fn start_file(&mut self, id: u8, filename: String, args: Vec<String>) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            Ok(worklet.start_file(filename, args, &self.looper))
        } else {
            Err(format!("No worklet with ID {id}").into())
        }
    }

    pub fn start_utf8(
        &mut self,
        id: u8,
        filename: String,
        source: String,
        args: Vec<String>,
    ) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            Ok(worklet.start_utf8(filename, source, args, &self.looper))
        } else {
            Err(format!("No worklet with ID {id}").into())
        }
    }

    pub fn start_bytes(
        &mut self,
        id: u8,
        filename: String,
        source: Vec<u8>,
        args: Vec<String>,
    ) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            Ok(worklet.start_bytes(filename, source, args, &self.looper))
        } else {
            Err(format!("No worklet with ID {id}").into())
        }
    }

    pub fn read(&mut self, id: u8) -> Result<Vec<u8>> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            Ok(worklet.read())
        } else {
            Err(format!("No worklet with ID {id}").into())
        }
    }

    pub fn write(&mut self, id: u8, data: Vec<u8>) -> Result<i32> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            Ok(worklet.write(data))
        } else {
            Err(format!("No worklet with ID {id}").into())
        }
    }

    pub fn update(&mut self, id: u8, readable: bool, writable: bool) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            Ok(worklet.update(readable, writable))
        } else {
            Err(format!("No worklet with ID {id}").into())
        }
    }

    pub fn suspend(&mut self, id: u8, linger: i32) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            Ok(worklet.suspend(linger))
        } else {
            Err(format!("No worklet with ID {id}").into())
        }
    }

    pub fn resume(&mut self, id: u8) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            Ok(worklet.resume())
        } else {
            Err(format!("No worklet with ID {id}").into())
        }
    }

    pub fn wakeup(&mut self, id: u8, deadline: i32) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            Ok(worklet.wakeup(deadline))
        } else {
            Err(format!("No worklet with ID {id}").into())
        }
    }

    pub fn terminate(&mut self, id: u8) -> Result<()> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            Ok(worklet.terminate())
        } else {
            Err(format!("No worklet with ID {id}").into())
        }
    }
}
