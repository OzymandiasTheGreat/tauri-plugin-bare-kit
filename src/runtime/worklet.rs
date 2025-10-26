#[cfg(not(target_os = "android"))]
use std::sync::Arc;
use std::{
    ffi::CString,
    marker::PhantomData,
    os::raw::{c_char, c_void},
    ptr::null_mut,
    slice,
};

#[cfg(not(target_os = "android"))]
use tauri::{async_runtime::block_on, Listener, Manager};
use tauri::{Runtime, WebviewWindow};
#[cfg(not(target_os = "android"))]
use tokio::sync::Notify;

use crate::runtime::ffi::*;

#[cfg(target_os = "android")]
pub struct Looper(pub *mut ALooper);
#[cfg(not(target_os = "android"))]
pub struct Looper();

unsafe impl Send for Looper {}

unsafe impl Sync for Looper {}

struct PollData<R: Runtime> {
    #[allow(dead_code)]
    worklet_id: u8,
    window: WebviewWindow<R>,
    callback_id: u32,
}

pub struct BareWorklet<R: Runtime> {
    /** pointers */
    worklet: *mut bare_worklet_t,
    ipc: *mut bare_ipc_t,
    poll: *mut bare_ipc_poll_t,

    /** state */
    started: bool,
    terminated: bool,

    /** typing */
    _runtime: PhantomData<R>,
}

unsafe impl<R: Runtime> Send for BareWorklet<R> {}

unsafe impl<R: Runtime> Sync for BareWorklet<R> {}

impl<R: Runtime> BareWorklet<R> {
    pub fn new(
        worklet_id: u8,
        memory_limit: usize,
        assets: Option<String>,
        window: WebviewWindow<R>,
        callback_id: u32,
    ) -> Self {
        let mut worklet: *mut bare_worklet_t = null_mut();
        let err = unsafe { bare_worklet_alloc(&mut worklet) };
        assert!(err == 0);

        let mut ipc: *mut bare_ipc_t = null_mut();
        let err = unsafe { bare_ipc_alloc(&mut ipc) };
        assert!(err == 0);

        let mut poll: *mut bare_ipc_poll_t = null_mut();
        let err = unsafe { bare_ipc_poll_alloc(&mut poll) };
        assert!(err == 0);

        let data = PollData {
            worklet_id,
            window,
            callback_id,
        };
        let data = Box::<PollData<R>>::into_raw(Box::new(data));
        unsafe { bare_ipc_poll_set_data(poll, data as *mut _) };

        let bare_worklet = Self {
            worklet,
            ipc,
            poll,
            started: false,
            terminated: false,
            _runtime: PhantomData,
        };

        let options = bare_worklet_options_t {
            memory_limit,
            assets: match assets {
                Some(assets) => assets.as_ptr() as *const c_char,
                None => null_mut(),
            },
        };
        let err = unsafe { bare_worklet_init(worklet, &options) };
        assert!(err == 0);

        bare_worklet
    }

    fn _start(
        &mut self,
        filename: String,
        source: Option<uv_buf_t>,
        args: Vec<String>,
        _looper: &Looper,
    ) {
        if self.started || self.terminated {
            return;
        }

        self.started = true;

        let mut argv: Vec<*const c_char> =
            args.iter().map(|a| a.as_ptr() as *const c_char).collect();
        let err = match source {
            Some(source) => unsafe {
                bare_worklet_start(
                    self.worklet,
                    filename.as_ptr() as *const c_char,
                    &source,
                    Some(on_free),
                    null_mut(),
                    argv.len() as i32,
                    argv.as_mut_ptr(),
                )
            },
            None => unsafe {
                bare_worklet_start(
                    self.worklet,
                    filename.as_ptr() as *const c_char,
                    null_mut(),
                    None,
                    null_mut(),
                    argv.len() as i32,
                    argv.as_mut_ptr(),
                )
            },
        };
        assert!(err == 0);

        let err = unsafe { bare_ipc_init(self.ipc, self.worklet) };
        assert!(err == 0);

        let err = unsafe { bare_ipc_poll_init(self.poll, self.ipc) };
        assert!(err == 0);

        #[cfg(target_os = "android")]
        unsafe {
            /* BareKit is expecting Looper for main thread,
             * but since tauri plugins run in their own threads
             * this fails. Replace acquired Looper with prepared Looper
             * from main thread.
             */
            let mut poll = *self.poll;
            ALooper_release(poll.looper);
            poll.looper = _looper.0;
            ALooper_acquire(poll.looper);
        }
    }

    pub fn start_file(&mut self, filename: String, args: Vec<String>, _looper: &Looper) {
        self._start(filename, None, args, _looper);
    }

    pub fn start_utf8(
        &mut self,
        filename: String,
        source: String,
        args: Vec<String>,
        _looper: &Looper,
    ) {
        let source = CString::new(source).unwrap();
        let len = source.count_bytes();
        let source = uv_buf_t {
            base: source.into_raw(),
            len,
        };

        self._start(filename, Some(source), args, _looper);
    }

    pub fn start_bytes(
        &mut self,
        filename: String,
        source: Vec<u8>,
        args: Vec<String>,
        _looper: &Looper,
    ) {
        let source = CString::new(source).unwrap();
        let len = source.count_bytes();
        let source = uv_buf_t {
            base: source.into_raw(),
            len,
        };

        self._start(filename, Some(source), args, _looper);
    }

    pub fn read(&mut self) -> Vec<u8> {
        if !self.started || self.terminated {
            return vec![];
        }

        let mut len: usize = 0;
        let mut data = null_mut();
        let err = unsafe { bare_ipc_read(self.ipc, &mut data, &mut len) };
        assert!(err == 0 || err == bare_ipc_would_block);

        if err == bare_ipc_would_block {
            return vec![];
        }

        unsafe { slice::from_raw_parts(data as *mut u8, len).to_vec() }
    }

    pub fn write(&mut self, data: Vec<u8>) -> i32 {
        let err = if data.is_empty() {
            unsafe { bare_ipc_write(self.ipc, null_mut(), 0) }
        } else {
            unsafe { bare_ipc_write(self.ipc, data.as_ptr() as *const _, data.len()) }
        };
        assert!(err >= 0 || err == bare_ipc_would_block);

        if err == bare_ipc_would_block {
            return 0;
        }

        err
    }

    pub fn update(&mut self, readable: bool, writable: bool) {
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

        let err = if events > 0 {
            unsafe { bare_ipc_poll_start(self.poll, events as i32, Some(on_poll::<R>)) }
        } else {
            unsafe { bare_ipc_poll_stop(self.poll) }
        };
        assert!(err == 0);
    }

    pub fn suspend(&mut self, linger: i32) {
        if !self.started || self.terminated {
            return;
        }

        let err = unsafe { bare_worklet_suspend(self.worklet, linger) };
        assert!(err == 0);
    }

    pub fn resume(&mut self) {
        if !self.started || self.terminated {
            return;
        }

        let err = unsafe { bare_worklet_resume(self.worklet) };
        assert!(err == 0);
    }

    pub fn wakeup(&mut self, deadline: i32) {
        if !self.started || self.terminated {
            return;
        }

        let err = unsafe { bare_worklet_wakeup(self.worklet, deadline) };
        assert!(err == 0);
    }

    pub fn terminate(&mut self) {
        if self.terminated {
            return;
        }

        self.terminated = true;

        if self.started {
            let err = unsafe { bare_worklet_terminate(self.worklet) };
            assert!(err == 0);

            unsafe {
                let poll = *self.poll;
                let data = Box::from_raw(poll.data as *mut PollData<R>);
                bare_ipc_poll_destroy(self.poll);
                drop(data);
                bare_ipc_destroy(self.ipc);
            }
        }

        unsafe { bare_worklet_destroy(self.worklet) };

        self.worklet = null_mut();
        self.ipc = null_mut();
        self.poll = null_mut();
    }
}

unsafe extern "C" fn on_free(_: *mut bare_worklet_t, source: *const uv_buf_t, _: *mut c_void) {
    let buffer = *source;
    drop(CString::from_raw(buffer.base));
}

#[cfg(not(target_os = "android"))]
unsafe extern "C" fn on_poll<R: Runtime>(poll: *mut bare_ipc_poll_t, events: i32) {
    let data = unsafe { &mut *(bare_ipc_poll_get_data(poll) as *mut PollData<R>) };
    let readable = (events as u32 & bare_ipc_readable) != 0;
    let writable = (events as u32 & bare_ipc_writable) != 0;

    let notify = Arc::new(Notify::new());
    let notifier = notify.clone();
    let notified = notify.notified();

    let app = data.window.app_handle();

    /* On at least apple platforms the callback is expected to block
     * However since Tauri's IPC is async so is the callback
     * as a workaround block until JS callback emits completion event
     */
    app.listen(format!("bare-kit:worklet:{}", data.worklet_id), move |e| {
        app.unlisten(e.id());
        notifier.notify_waiters();
    });

    data.window
        .eval(format!(
            "window.__TAURI_INTERNALS__.runCallback({}, {{ readable: {}, writable: {} }})",
            data.callback_id, readable, writable,
        ))
        .unwrap();

    block_on(async move {
        notified.await;
    });
}

#[cfg(target_os = "android")]
unsafe extern "C" fn on_poll<R: Runtime>(poll: *mut bare_ipc_poll_t, events: i32) {
    let data = unsafe { &mut *(bare_ipc_poll_get_data(poll) as *mut PollData<R>) };
    let readable = (events as u32 & bare_ipc_readable) != 0;
    let writable = (events as u32 & bare_ipc_writable) != 0;

    data.window
        .eval(format!(
            "window.__TAURI_INTERNALS__.runCallback({}, {{ readable: {}, writable: {} }})",
            data.callback_id, readable, writable,
        ))
        .unwrap();
}
