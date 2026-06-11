use std::sync::{Arc, Mutex};

use crate::bare_kit::{
    ffi::{bare_ipc_poll_t, bare_ipc_readable, bare_ipc_t, bare_ipc_writable, bare_worklet_t},
    ipc::{
        ipc_destroy, ipc_new, ipc_poll_destroy, ipc_poll_new, ipc_poll_start, ipc_poll_stop,
        ipc_read, ipc_write,
    },
    worklet::{
        worklet_destroy, worklet_new, worklet_on_idle, worklet_on_resume, worklet_on_suspend,
        worklet_on_wakeup, worklet_optimize_for_memory, worklet_resume, worklet_start,
        worklet_suspend, worklet_terminate, worklet_wakeup,
    },
};

pub(crate) mod ffi;
pub(crate) mod ipc;
pub(crate) mod worklet;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct BareWorklet {
    worklet: *mut bare_worklet_t,
}

unsafe impl Send for BareWorklet {}

unsafe impl Sync for BareWorklet {}

impl BareWorklet {
    pub fn optimize_for_memory(enabled: bool) {
        worklet_optimize_for_memory(enabled);
    }

    pub fn init(memory_limit: usize, assets: Option<String>) -> Self {
        Self {
            worklet: worklet_new(memory_limit, assets),
        }
    }

    pub fn start_file(&self, filename: String, args: Vec<String>) {
        worklet_start(self.worklet, filename, None, args);
    }

    pub fn start_utf8(&self, filename: String, source: String, args: Vec<String>) {
        worklet_start(self.worklet, filename, Some(Vec::from(source)), args);
    }

    pub fn start_bytes(&self, filename: String, source: Vec<u8>, args: Vec<String>) {
        worklet_start(self.worklet, filename, Some(source), args);
    }

    pub fn suspend(&self, linger: i32) {
        worklet_suspend(self.worklet, linger);
    }

    pub fn resume(&self) {
        worklet_resume(self.worklet);
    }

    pub fn wakeup(&self, deadline: i32) {
        worklet_wakeup(self.worklet, deadline);
    }

    pub fn terminate(&self) {
        worklet_terminate(self.worklet);
        worklet_destroy(self.worklet);
    }

    pub fn on_suspend<F>(&self, callback: F)
    where
        F: FnMut(i32) + 'static,
    {
        worklet_on_suspend(self.worklet, callback);
    }

    pub fn on_wakeup<F>(&self, callback: F)
    where
        F: FnMut(i32) + 'static,
    {
        worklet_on_wakeup(self.worklet, callback);
    }

    pub fn on_idle<F>(&self, callback: F)
    where
        F: FnMut() + 'static,
    {
        worklet_on_idle(self.worklet, callback);
    }

    pub fn on_resume<F>(&self, callback: F)
    where
        F: FnMut() + 'static,
    {
        worklet_on_resume(self.worklet, callback);
    }
}

#[derive(Clone)]
pub struct BareIPC {
    ipc: *mut bare_ipc_t,
    poll: *mut bare_ipc_poll_t,

    readable: Option<Arc<Mutex<dyn FnMut() + 'static>>>,
    writable: Option<Arc<Mutex<dyn FnMut() + 'static>>>,
}

unsafe impl Send for BareIPC {}

unsafe impl Sync for BareIPC {}

impl BareIPC {
    pub fn init(worklet: BareWorklet) -> Self {
        let ipc = ipc_new(worklet.worklet);
        let poll = ipc_poll_new(ipc);

        Self {
            ipc,
            poll,
            readable: None,
            writable: None,
        }
    }

    pub fn read<F>(self, callback: F)
    where
        F: FnOnce(Vec<u8>) + 'static,
    {
        let mut callback = Some(callback);

        if let Some(data) = ipc_read(self.ipc) {
            if let Some(callback) = callback.take() {
                callback(data);
            }
        } else {
            self.clone().set_readable(Some(move || {
                if let Some(data) = ipc_read(self.ipc) {
                    self.clone().set_readable(None::<fn()>);

                    if let Some(callback) = callback.take() {
                        callback(data);
                    }
                }
            }));
        }
    }

    pub fn write<F>(self, data: Vec<u8>, callback: F)
    where
        F: FnOnce() + 'static,
    {
        let mut callback = Some(callback);
        let written = ipc_write(self.ipc, Some(data.clone())) as usize;

        if written == data.len() {
            if let Some(callback) = callback.take() {
                callback();
            }
        } else {
            self.clone().set_writable(Some(move || {
                let mut remaining = data[written..].to_vec();
                let written = ipc_write(self.ipc, Some(remaining.clone())) as usize;

                if written == remaining.len() {
                    self.clone().set_writable(None::<fn()>);

                    if let Some(callback) = callback.take() {
                        callback();
                    }
                } else {
                    remaining = remaining[written..].to_vec();
                }
            }));
        }
    }

    pub fn close(&self) {
        ipc_poll_destroy(self.poll);
        ipc_destroy(self.ipc);
    }

    fn set_readable<F>(mut self, readable: Option<F>)
    where
        F: FnMut() + 'static,
    {
        if let Some(callback) = readable {
            self.readable = Some(Arc::new(Mutex::new(callback)));
        } else {
            self.readable = None;
        }

        self.update();
    }

    fn set_writable<F>(mut self, writable: Option<F>)
    where
        F: FnMut() + 'static,
    {
        if let Some(callback) = writable {
            self.writable = Some(Arc::new(Mutex::new(callback)));
        } else {
            self.writable = None;
        }

        self.update();
    }

    fn update(mut self) {
        let mut events = 0;

        if self.readable.is_some() {
            events |= bare_ipc_readable;
        }

        if self.writable.is_some() {
            events |= bare_ipc_writable;
        }

        if events > 0 {
            ipc_poll_start(self.poll, events as i32, move |readable, writable| {
                if readable {
                    if let Some(callback) = &mut self.readable {
                        callback.lock().unwrap()();
                    }
                }

                if writable {
                    if let Some(callback) = &mut self.writable {
                        callback.lock().unwrap()();
                    }
                }
            });
        } else {
            ipc_poll_stop(self.poll);
        }
    }
}
