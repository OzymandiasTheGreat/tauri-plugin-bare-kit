use std::{collections::HashMap, ffi::CString, os::raw::c_void, ptr::null_mut, slice};

use tauri::{Runtime, WebviewWindow};

use crate::bindings::*;

struct Looper(*mut ALooper);

unsafe impl Send for Looper {}

struct PollData<R: Runtime> {
    window: WebviewWindow<R>,
    callback_id: u32,
}

#[derive(Clone, Debug)]
pub struct BareKitWorklet {
    worklet: *mut bare_worklet_t,
    ipc: *mut bare_ipc_t,
    poll: *mut bare_ipc_poll_t,

    started: bool,
    terminated: bool,
}

unsafe impl Send for BareKitWorklet {}

unsafe impl Sync for BareKitWorklet {}

impl BareKitWorklet {
    fn init<R: Runtime>(
        window: WebviewWindow<R>,
        memory_limit: i32,
        assets: Option<String>,
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
            window,
            callback_id,
        };
        let data = Box::<PollData<R>>::into_raw(Box::new(data));
        unsafe { bare_ipc_poll_set_data(poll, data as *mut _) };

        let bare_kit_worklet = Self {
            worklet,
            ipc,
            poll,
            started: false,
            terminated: false,
        };

        let options: bare_worklet_options_t = bare_worklet_options_s {
            memory_limit: memory_limit as usize,
            assets: match assets {
                Some(assets) => assets.as_ptr(),
                None => null_mut(),
            },
        };

        let err = unsafe { bare_worklet_init(worklet, &options) };
        assert!(err == 0);

        bare_kit_worklet
    }

    fn start(&mut self, looper: &Looper, filename: String, source: String, args: Vec<String>) {
        if self.started || self.terminated {
            return;
        }

        self.started = true;

        let bytes = CString::new(source).unwrap();
        let bytelen = bytes.count_bytes();
        let buffer = uv_buf_t {
            base: bytes.into_raw(),
            len: bytelen,
        };
        let mut argv: Vec<*const u8> = args.iter().map(|s| s.as_ptr()).collect();
        let err = unsafe {
            bare_worklet_start(
                self.worklet,
                filename.as_ptr(),
                &buffer,
                Some(on_free),
                null_mut(),
                argv.len() as i32,
                argv.as_mut_ptr(),
            )
        };
        assert!(err == 0);

        let err = unsafe { bare_ipc_init(self.ipc, self.worklet) };
        assert!(err == 0);

        let err = unsafe { bare_ipc_poll_init(self.poll, self.ipc) };
        assert!(err == 0);

        let mut poll = unsafe { *self.poll };
        unsafe { ALooper_release(poll.looper) };
        poll.looper = looper.0;
    }

    fn read(&mut self) -> Option<Vec<u8>> {
        if !self.started || self.terminated {
            return None;
        }

        let mut len: usize = 0;
        let mut data = null_mut();
        let err = unsafe { bare_ipc_read(self.ipc, &mut data, &mut len) };
        assert!(err == 0 || err == bare_ipc_would_block);

        if err == bare_ipc_would_block {
            return None;
        }

        unsafe { Some(slice::from_raw_parts(data as *mut u8, len).to_vec()) }
    }

    fn write(&mut self, data: Option<Vec<u8>>) -> i32 {
        let err = match data {
            Some(data) => unsafe {
                bare_ipc_write(self.ipc, data.as_ptr() as *const _, data.len())
            },
            None => 0,
        };
        assert!(err >= 0 || err == bare_ipc_would_block);

        if err == bare_ipc_would_block {
            return 0;
        }

        err
    }

    fn update<R: Runtime>(&mut self, readable: bool, writable: bool) {
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
            let err = unsafe { bare_ipc_poll_start(self.poll, events as i32, Some(on_poll::<R>)) };
            assert!(err == 0);
        } else {
            let err = unsafe { bare_ipc_poll_stop(self.poll) };
            assert!(err == 0);
        }
    }

    fn suspend(&mut self, linger: i32) {
        if !self.started || self.terminated {
            return;
        }

        let err = unsafe { bare_worklet_suspend(self.worklet, linger) };
        assert!(err == 0);
    }

    fn resume(&mut self) {
        if !self.started || self.terminated {
            return;
        }

        let err = unsafe { bare_worklet_resume(self.worklet) };
        assert!(err == 0);
    }

    fn terminate<R: Runtime>(&mut self) {
        if self.terminated {
            return;
        }

        self.terminated = true;

        if self.started {
            let err = unsafe { bare_worklet_terminate(self.worklet) };
            assert!(err == 0);

            unsafe {
                let poll = *(self.poll);
                let data = Box::from_raw(poll.data as *mut PollData<R>);
                bare_ipc_poll_destroy(self.poll);
                drop(data);
            };
            unsafe { bare_ipc_destroy(self.ipc) };

            // TODO: think of a safe way to `free()` these pointers
        }

        unsafe { bare_worklet_destroy(self.worklet) };
        // TODO: re:`free()` see above

        self.worklet = null_mut();
        self.ipc = null_mut();
        self.poll = null_mut();
    }
}

unsafe extern "C" fn on_poll<R: Runtime>(poll: *mut bare_ipc_poll_t, events: i32) {
    let data = unsafe { &mut *(bare_ipc_poll_get_data(poll) as *mut PollData<R>) };

    data.window
        .eval(format!(
            "window.__TAURI_INTERNALS__.runCallback({}, {}, {})",
            data.callback_id,
            (events as u32 & bare_ipc_readable) != 0,
            (events as u32 & bare_ipc_writable) != 0,
        ))
        .unwrap();
}

unsafe extern "C" fn on_free(
    _worklet: *mut bare_worklet_t,
    source: *const uv_buf_t,
    _data: *mut c_void,
) {
    let buffer = *source;
    drop(CString::from_raw(buffer.base));
}

pub struct BareKitModule {
    id: i32,
    looper: Looper,
    worklets: HashMap<i32, BareKitWorklet>,
}

impl BareKitModule {
    pub fn init() -> Self {
        let looper = Looper(unsafe { ALooper_forThread() });

        unsafe { ALooper_acquire(looper.0) };

        Self {
            id: 0,
            looper,
            worklets: HashMap::new(),
        }
    }

    pub fn invalidate<R: Runtime>(&mut self) {
        for (_, worklet) in &mut self.worklets {
            worklet.terminate::<R>();
        }
        self.worklets.clear();
    }

    pub fn new<R: Runtime>(
        &mut self,
        window: WebviewWindow<R>,
        memory_limit: i32,
        assets: Option<String>,
        poll_callback_id: u32,
    ) -> i32 {
        self.id += 1;

        let worklet = BareKitWorklet::init(window, memory_limit, assets, poll_callback_id);
        self.worklets.insert(self.id, worklet);

        self.id
    }

    pub fn start(&mut self, id: i32, filename: String, source: String, argv: Vec<String>) {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            worklet.start(&self.looper, filename, source, argv);
        }
    }

    pub fn read(&mut self, id: i32) -> Option<Vec<u8>> {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            return worklet.read();
        }

        None
    }

    pub fn write(&mut self, id: i32, data: Option<Vec<u8>>) -> i32 {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            return worklet.write(data);
        }

        return bare_ipc_error;
    }

    pub fn update<R: Runtime>(&mut self, id: i32, readable: bool, writable: bool) {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            return worklet.update::<R>(readable, writable);
        }
    }

    pub fn suspend(&mut self, id: i32, linger: i32) {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            return worklet.suspend(linger);
        }
    }

    pub fn resume(&mut self, id: i32) {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            return worklet.resume();
        }
    }

    pub fn terminate<R: Runtime>(&mut self, id: i32) {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            return worklet.terminate::<R>();
        }
    }
}
