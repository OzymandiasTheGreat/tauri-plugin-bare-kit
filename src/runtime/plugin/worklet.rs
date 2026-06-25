use std::ptr::null_mut;
use tauri::{AppHandle, Runtime, WebviewWindow};

#[cfg(not(target_os = "android"))]
use std::sync::Arc;
#[cfg(not(target_os = "android"))]
use tauri::async_runtime::block_on;
#[cfg(not(target_os = "android"))]
use tokio::sync::Notify;

use crate::runtime::bare_kit::ffi::{
    bare_ipc_poll_t, bare_ipc_readable, bare_ipc_t, bare_ipc_writable, bare_worklet_t,
};
use crate::runtime::bare_kit::ipc::*;
use crate::runtime::bare_kit::worklet::*;

#[cfg(target_os = "android")]
use crate::runtime::bare_kit::ffi::{ALooper, ALooper_acquire, ALooper_release};

#[cfg(target_os = "android")]
pub(crate) struct Looper(pub(crate) *mut ALooper);

#[cfg(target_os = "android")]
unsafe impl Send for Looper {}

#[cfg(target_os = "android")]
unsafe impl Sync for Looper {}

pub(crate) struct BareKitWorklet<R: Runtime> {
    id: u8,
    app: AppHandle<R>,
    window: WebviewWindow<R>,

    worklet: *mut bare_worklet_t,
    ipc: *mut bare_ipc_t,
    poll: *mut bare_ipc_poll_t,

    started: bool,
    terminated: bool,

    on_poll: u32,
}

unsafe impl<R: Runtime> Send for BareKitWorklet<R> {}

unsafe impl<R: Runtime> Sync for BareKitWorklet<R> {}

impl<R: Runtime> Clone for BareKitWorklet<R> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            app: self.app.clone(),
            window: self.window.clone(),
            worklet: self.worklet,
            ipc: self.ipc,
            poll: self.poll,
            started: self.started,
            terminated: self.terminated,
            on_poll: self.on_poll,
        }
    }
}

impl<R: Runtime> BareKitWorklet<R> {
    pub(crate) fn new(
        id: u8,
        app: AppHandle<R>,
        window: WebviewWindow<R>,
        memory_limit: usize,
        assets: Option<String>,
        on_poll: u32,
        on_suspend: u32,
        on_wakeup: u32,
        on_idle: u32,
        on_resume: u32,
    ) -> Self {
        let worklet = worklet_new(memory_limit, assets);

        let suspend_window = window.clone();
        worklet_on_suspend(worklet, move |linger| {
            suspend_window
                .eval(format!(
                    "window.__TAURI_INTERNALS__.runCallback({}, {{ linger: {} }})",
                    on_suspend, linger,
                ))
                .unwrap();
        });

        let wakeup_window = window.clone();
        worklet_on_wakeup(worklet, move |deadline| {
            wakeup_window
                .eval(format!(
                    "window.__TAURI_INTERNALS__.runCallback({}, {{ deadline: {} }})",
                    on_wakeup, deadline,
                ))
                .unwrap();
        });

        let idle_window = window.clone();
        worklet_on_idle(worklet, move || {
            idle_window
                .eval(format!(
                    "window.__TAURI_INTERNALS__.runCallback({})",
                    on_idle,
                ))
                .unwrap();
        });

        let resume_window = window.clone();
        worklet_on_resume(worklet, move || {
            resume_window
                .eval(format!(
                    "window.__TAURI_INTERNALS__.runCallback({})",
                    on_resume,
                ))
                .unwrap();
        });

        Self {
            id,
            app,
            window,
            worklet,
            ipc: null_mut(),
            poll: null_mut(),
            started: false,
            terminated: false,
            on_poll,
        }
    }

    pub(crate) fn optimize_for_memory(enabled: bool) {
        worklet_optimize_for_memory(enabled);
    }

    pub(crate) fn start(&mut self, filename: String, source: Option<Vec<u8>>, args: Vec<String>) {
        if self.started || self.terminated {
            return;
        }

        self.started = true;

        worklet_start(self.worklet, filename, source, args);
        self.ipc = ipc_new(self.worklet);
        self.poll = ipc_poll_new(self.ipc);
    }

    #[cfg(target_os = "android")]
    pub(crate) fn set_looper(&mut self, looper: &Looper) {
        unsafe {
            let mut poll = *self.poll;

            ALooper_release(poll.looper);
            poll.looper = looper.0;
            ALooper_acquire(poll.looper);
        }
    }

    pub(crate) fn read(&mut self) -> Option<Vec<u8>> {
        if !self.started || self.terminated {
            return None;
        }

        ipc_read(self.ipc)
    }

    pub(crate) fn write(&mut self, data: Option<Vec<u8>>) -> i32 {
        ipc_write(self.ipc, data.as_ref())
    }

    pub(crate) fn update(&mut self, readable: bool, writable: bool) {
        if self.terminated {
            return;
        }

        let mut events = 0;

        if readable {
            events |= bare_ipc_readable;
        }

        if writable {
            events |= bare_ipc_writable;
        }

        if events > 0 {
            let worklet = self.clone();

            ipc_poll_start(self.poll, events as i32, move |readable, writable| {
                #[cfg(target_os = "android")]
                {
                    worklet.window
                    .eval(format!(
                        "window.__TAURI_INTERNALS__.runCallback({}, {{ readable: {}, writable: {} }})",
                        worklet.on_poll, readable, writable,
                    ))
                    .unwrap();
                }

                #[cfg(not(target_os = "android"))]
                {
                    let notify = Arc::new(Notify::new());
                    let notifier = notify.clone();
                    let notified = notify.notified();

                    worklet.window
                    .eval_with_callback(format!(
                        "window.__TAURI_INTERNALS__.runCallback({}, {{ readable: {}, writable: {} }})",
                        worklet.on_poll, readable, writable,
                    ), move |_| {
                        notifier.notify_waiters();
                    })
                    .unwrap();

                    block_on(async move {
                        notified.await;
                    });
                }
            });
        } else {
            ipc_poll_stop(self.poll);
        }
    }

    pub(crate) fn suspend(&mut self, linger: i32) {
        worklet_suspend(self.worklet, linger);
    }

    pub(crate) fn resume(&mut self) {
        worklet_resume(self.worklet);
    }

    pub(crate) fn wakeup(&mut self, deadline: i32) {
        worklet_wakeup(self.worklet, deadline);
    }

    pub(crate) fn terminate(&mut self) {
        if self.terminated {
            return;
        }

        self.terminated = true;

        if self.started {
            worklet_terminate(self.worklet);
            ipc_poll_destroy(self.poll);
            ipc_destroy(self.ipc);
        }

        worklet_destroy(self.worklet);

        self.worklet = null_mut();
        self.ipc = null_mut();
        self.poll = null_mut();
    }
}
