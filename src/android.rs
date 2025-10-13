use std::{collections::HashMap, mem::MaybeUninit, os::raw::c_void, ptr::null_mut, slice};

use tauri::{Runtime, WebviewWindow};

use crate::bindings::*;

pub struct BareKitWorklet<R: Runtime> {
    worklet: *mut bare_worklet_t,
    ipc: *mut bare_ipc_t,
    poll: *mut bare_ipc_poll_t,

    on_poll: i32,
    window: Option<WebviewWindow<R>>,

    started: bool,
    terminated: bool,
}

unsafe impl<R: Runtime> Send for BareKitWorklet<R> {}

unsafe impl<R: Runtime> Sync for BareKitWorklet<R> {}

impl<R: Runtime> BareKitWorklet<R> {
    fn init(memory_limit: i32, assets: Option<String>, on_poll: i32) -> Self {
        let mut worklet: *mut bare_worklet_t = null_mut();
        let err = unsafe { bare_worklet_alloc(&mut worklet) };
        assert!(err == 0);

        let mut ipc: *mut bare_ipc_t = null_mut();
        let err = unsafe { bare_ipc_alloc(&mut ipc) };
        assert!(err == 0);

        let mut poll: *mut bare_ipc_poll_t = null_mut();
        let err = unsafe { bare_ipc_poll_alloc(&mut poll) };
        assert!(err == 0);

        let mut bare_kit_worklet = Self {
            worklet,
            ipc,
            poll,
            on_poll,
            window: None,
            started: false,
            terminated: false,
        };
        unsafe { bare_ipc_poll_set_data(poll, &mut bare_kit_worklet as *mut _ as *mut c_void) };

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

    fn start(&mut self, filename: String, source: Option<Vec<u8>>, args: Vec<String>) {
        if self.started || self.terminated {
            return;
        }

        self.started = true;

        let source = match source {
            Some(mut src) => {
                let mut buffer = uv_buf_t {
                    base: src.as_mut_ptr(),
                    len: src.len(),
                };
                &mut buffer as *mut _
            }
            None => null_mut(),
        };
        let mut argv: Vec<*const u8> = args.iter().map(|s| s.as_ptr()).collect();
        let err = unsafe {
            bare_worklet_start(
                self.worklet,
                filename.as_ptr(),
                source,
                None,
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
    }

    fn read(&mut self) -> Option<Vec<u8>> {
        if !self.started || self.terminated {
            return None;
        }

        let mut len: usize = 0;
        let mut data = MaybeUninit::<u8>::uninit();
        let err = unsafe { bare_ipc_read(self.ipc, data.as_mut_ptr() as *mut _, &mut len) };
        assert!(err == 0 || err == bare_ipc_would_block);

        if err == bare_ipc_would_block {
            return None;
        }

        unsafe { Some(slice::from_raw_parts(data.as_mut_ptr(), len).to_vec()) }
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

    fn update(&mut self, window: WebviewWindow<R>, readable: bool, writable: bool) {
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

        self.window = Some(window);

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

    fn terminate(&mut self) {
        if self.terminated {
            return;
        }

        self.terminated = true;

        if self.started {
            let err = unsafe { bare_worklet_terminate(self.worklet) };
            assert!(err == 0);

            unsafe { bare_ipc_poll_destroy(self.poll) };
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
    let poll = unsafe { *poll };
    let worklet_ptr = poll.data as *mut BareKitWorklet<R>;
    let worklet = unsafe { &mut *worklet_ptr };

    if let Some(window) = &worklet.window {
        window
            .eval(format!(
                "window.__TAURI_INTERNALS__.runCallback({}, {}, {})",
                worklet.on_poll,
                (events as u32 & bare_ipc_readable) != 0,
                (events as u32 & bare_ipc_writable) != 0,
            ))
            .unwrap();
    }
}

pub struct BareKitModule<R: Runtime> {
    id: i32,
    worklets: HashMap<i32, BareKitWorklet<R>>,
}

impl<R: Runtime> BareKitModule<R> {
    pub fn init() -> Self {
        Self {
            id: 0,
            worklets: HashMap::new(),
        }
    }

    pub fn invalidate(&mut self) {
        for (_, worklet) in &mut self.worklets {
            worklet.terminate();
        }
        self.worklets.clear();
    }

    pub fn new(&mut self, memory_limit: i32, assets: Option<String>, on_poll: i32) -> i32 {
        self.id += 1;

        let worklet = BareKitWorklet::init(memory_limit, assets, on_poll);
        self.worklets.insert(self.id, worklet);

        self.id
    }

    pub fn start(&mut self, id: i32, filename: String, source: Option<Vec<u8>>, argv: Vec<String>) {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            worklet.start(filename, source, argv);
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

    pub fn update(&mut self, window: WebviewWindow<R>, id: i32, readable: bool, writable: bool) {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            return worklet.update(window, readable, writable);
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

    pub fn terminate(&mut self, id: i32) {
        if let Some(worklet) = self.worklets.get_mut(&id) {
            return worklet.terminate();
        }
    }
}
