use parking_lot::ReentrantMutex;
use std::{cell::RefCell, sync::Arc};

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

#[derive(Clone, Copy)]
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

    readable: Arc<ReentrantMutex<RefCell<Option<Box<dyn FnMut(Vec<u8>)>>>>>,
    writable: Arc<ReentrantMutex<RefCell<Option<Box<dyn FnMut()>>>>>,

    data: Arc<ReentrantMutex<RefCell<Option<Vec<u8>>>>>,
}

unsafe impl Send for BareIPC {}

unsafe impl Sync for BareIPC {}

impl BareIPC {
    pub fn init(worklet: &BareWorklet) -> Self {
        let ipc = ipc_new(worklet.worklet);
        let poll = ipc_poll_new(ipc);

        Self {
            ipc,
            poll,
            readable: Arc::new(ReentrantMutex::new(RefCell::new(None))),
            writable: Arc::new(ReentrantMutex::new(RefCell::new(None))),
            data: Arc::new(ReentrantMutex::new(RefCell::new(None))),
        }
    }

    pub fn read<F>(&self, mut callback: F)
    where
        F: FnMut(Vec<u8>) + 'static,
    {
        if let Some(data) = ipc_read(self.ipc) {
            callback(data);
        } else {
            *self.readable.lock().borrow_mut() = Some(Box::new(callback));

            self.update();
        }
    }

    pub fn write<F>(&self, data: &Vec<u8>, mut callback: F)
    where
        F: FnMut() + 'static,
    {
        let written = ipc_write(self.ipc, Some(data)) as usize;

        if written == data.len() {
            callback();
        } else {
            *self.data.lock().borrow_mut() = Some(data[written..].to_vec());
            *self.writable.lock().borrow_mut() = Some(Box::new(callback));

            self.update();
        }
    }

    pub fn close(&self) {
        ipc_poll_destroy(self.poll);
        ipc_destroy(self.ipc);
    }

    fn update(&self) {
        let mut events = 0;

        if self.readable.lock().borrow().is_some() {
            events |= bare_ipc_readable;
        }

        if self.writable.lock().borrow().is_some() {
            events |= bare_ipc_writable;
        }

        if events > 0 {
            let this = self.clone();

            ipc_poll_start(this.poll, events as i32, move |readable, writable| {
                if readable {
                    if let Some(data) = ipc_read(this.ipc) {
                        let callback_ref = this.readable.lock();
                        let mut callback_ref = callback_ref.borrow_mut();
                        let mut callback = callback_ref.take();

                        drop(callback_ref);
                        this.update();

                        if let Some(callback) = &mut callback {
                            callback(data);
                        }
                    }
                }

                if writable {
                    let data_lock = this.data.lock();
                    let mut data_ref = data_lock.borrow_mut();

                    if let Some(data) = &*data_ref {
                        let written = ipc_write(this.ipc, Some(data)) as usize;

                        if written == data.len() {
                            let callback_ref = this.writable.lock();
                            let mut callback_ref = callback_ref.borrow_mut();
                            let mut callback = callback_ref.take();

                            drop(callback_ref);
                            this.update();

                            if let Some(callback) = &mut callback {
                                *data_ref = None;

                                callback();
                            }
                        } else {
                            *data_ref = Some(data[written..].to_vec());
                        }
                    }
                }
            });
        } else {
            ipc_poll_stop(self.poll);
        }
    }
}
