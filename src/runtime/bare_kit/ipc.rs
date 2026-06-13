use std::{ffi::c_void, ptr::null_mut, slice};

use crate::runtime::bare_kit::ffi::*;

pub(crate) fn ipc_new(worklet: *mut bare_worklet_t) -> *mut bare_ipc_t {
    let mut ipc: *mut bare_ipc_t = null_mut();
    let err = unsafe { bare_ipc_alloc(&mut ipc) };
    assert!(err == 0);

    let err = unsafe { bare_ipc_init(ipc, worklet) };
    assert!(err == 0);

    ipc
}

pub(crate) fn ipc_destroy(ipc: *mut bare_ipc_t) {
    unsafe { bare_ipc_destroy(ipc) };
}

#[allow(dead_code)]
pub(crate) fn ipc_get_incoming(ipc: *mut bare_ipc_t) -> i32 {
    unsafe { bare_ipc_get_incoming(ipc) }
}

#[allow(dead_code)]
pub(crate) fn ipc_get_outgoing(ipc: *mut bare_ipc_t) -> i32 {
    unsafe { bare_ipc_get_outgoing(ipc) }
}

pub(crate) fn ipc_read(ipc: *mut bare_ipc_t) -> Option<Vec<u8>> {
    let mut len: usize = 0;
    let mut data = null_mut();
    let err = unsafe { bare_ipc_read(ipc, &mut data, &mut len) };
    assert!(err == 0 || err == bare_ipc_would_block);

    if err == bare_ipc_would_block {
        return None;
    }

    Some(unsafe { slice::from_raw_parts(data as *mut u8, len).to_vec() })
}

pub(crate) fn ipc_write(ipc: *mut bare_ipc_t, data: Option<&Vec<u8>>) -> i32 {
    let err = match data {
        Some(data) => unsafe { bare_ipc_write(ipc, data.as_ptr() as *const _, data.len()) },
        None => unsafe { bare_ipc_write(ipc, null_mut(), 0) },
    };
    assert!(err >= 0 || err == bare_ipc_would_block);

    if err == bare_ipc_would_block {
        return 0;
    }

    err
}

struct PollCallback(Box<dyn FnMut(bool, bool)>);

pub(crate) fn ipc_poll_new(ipc: *mut bare_ipc_t) -> *mut bare_ipc_poll_t {
    let mut poll: *mut bare_ipc_poll_t = null_mut();
    let err = unsafe { bare_ipc_poll_alloc(&mut poll) };
    assert!(err == 0);

    let err = unsafe { bare_ipc_poll_init(poll, ipc) };
    assert!(err == 0);

    poll
}

pub(crate) fn ipc_poll_destroy(poll: *mut bare_ipc_poll_t) {
    unsafe { bare_ipc_poll_destroy(poll) };
}

#[allow(dead_code)]
pub(crate) fn ipc_poll_get_ipc(poll: *mut bare_ipc_poll_t) -> *mut bare_ipc_t {
    unsafe { bare_ipc_poll_get_ipc(poll) }
}

pub(crate) fn ipc_poll_start<F>(poll: *mut bare_ipc_poll_t, events: i32, callback: F)
where
    F: FnMut(bool, bool) + 'static,
{
    let err = unsafe {
        let data = Box::new(PollCallback(Box::new(callback)));

        bare_ipc_poll_set_data(poll, Box::into_raw(data) as *mut c_void);
        bare_ipc_poll_start(poll, events, Some(on_poll))
    };
    assert!(err == 0);
}

pub(crate) fn ipc_poll_stop(poll: *mut bare_ipc_poll_t) {
    let err = unsafe {
        let data = bare_ipc_poll_get_data(poll) as *mut PollCallback;

        if !data.is_null() {
            drop(Box::from_raw(data));
        }

        bare_ipc_poll_stop(poll)
    };
    assert!(err == 0);
}

unsafe extern "C" fn on_poll(poll: *mut bare_ipc_poll_t, events: i32) {
    let callback_ptr = bare_ipc_poll_get_data(poll) as *mut PollCallback;

    if !callback_ptr.is_null() {
        let callback = &mut *callback_ptr;
        let readable = (events & bare_ipc_readable as i32) != 0;
        let writable = (events & bare_ipc_writable as i32) != 0;

        callback.0(readable, writable);
    }
}
