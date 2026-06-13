use std::{
    ffi::{c_char, c_void, CString},
    ptr::null_mut,
};

use crate::runtime::bare_kit::ffi::*;

struct WorkletCallbacks {
    suspend: Option<Box<dyn FnMut(i32)>>,
    wakeup: Option<Box<dyn FnMut(i32)>>,
    idle: Option<Box<dyn FnMut()>>,
    resume: Option<Box<dyn FnMut()>>,
}

pub(crate) fn worklet_optimize_for_memory(enabled: bool) {
    let err = unsafe { bare_worklet_optimize_for_memory(enabled) };
    assert!(err == 0);
}

pub(crate) fn worklet_new(memory_limit: usize, assets: Option<String>) -> *mut bare_worklet_t {
    let mut worklet: *mut bare_worklet_t = null_mut();
    let err = unsafe { bare_worklet_alloc(&mut worklet) };
    assert!(err == 0);

    let options = bare_worklet_options_t {
        memory_limit,
        assets: match assets {
            Some(assets) => assets.as_ptr() as *const c_char,
            None => null_mut(),
        },
    };

    let err = unsafe { bare_worklet_init(worklet, &options) };
    assert!(err == 0);

    let callbacks = Box::new(WorkletCallbacks {
        suspend: None,
        wakeup: None,
        idle: None,
        resume: None,
    });
    unsafe { bare_worklet_set_data(worklet, Box::into_raw(callbacks) as *mut c_void) };

    worklet
}

pub(crate) fn worklet_destroy(worklet: *mut bare_worklet_t) {
    unsafe {
        drop(Box::from_raw(
            bare_worklet_get_data(worklet) as *mut WorkletCallbacks
        ));
        bare_worklet_destroy(worklet);
    }
}

pub(crate) fn worklet_on_suspend<F>(worklet: *mut bare_worklet_t, callback: F)
where
    F: FnMut(i32) + 'static,
{
    let err = unsafe {
        let callbacks = bare_worklet_get_data(worklet) as *mut WorkletCallbacks;
        let callbacks = &mut *callbacks;

        callbacks.suspend = Some(Box::new(callback));

        bare_worklet_on_suspend(worklet, Some(on_suspend), worklet as *mut c_void)
    };
    assert!(err == 0);
}

pub(crate) fn worklet_on_wakeup<F>(worklet: *mut bare_worklet_t, callback: F)
where
    F: FnMut(i32) + 'static,
{
    let err = unsafe {
        let callbacks = bare_worklet_get_data(worklet) as *mut WorkletCallbacks;
        let callbacks = &mut *callbacks;

        callbacks.wakeup = Some(Box::new(callback));

        bare_worklet_on_wakeup(worklet, Some(on_wakeup), worklet as *mut c_void)
    };
    assert!(err == 0);
}

pub(crate) fn worklet_on_idle<F>(worklet: *mut bare_worklet_t, callback: F)
where
    F: FnMut() + 'static,
{
    let err = unsafe {
        let callbacks = bare_worklet_get_data(worklet) as *mut WorkletCallbacks;
        let callbacks = &mut *callbacks;

        callbacks.idle = Some(Box::new(callback));

        bare_worklet_on_idle(worklet, Some(on_idle), worklet as *mut c_void)
    };
    assert!(err == 0);
}

pub(crate) fn worklet_on_resume<F>(worklet: *mut bare_worklet_t, callback: F)
where
    F: FnMut() + 'static,
{
    let err = unsafe {
        let callbacks = bare_worklet_get_data(worklet) as *mut WorkletCallbacks;
        let callbacks = &mut *callbacks;

        callbacks.resume = Some(Box::new(callback));

        bare_worklet_on_resume(worklet, Some(on_resume), worklet as *mut c_void)
    };
    assert!(err == 0);
}

pub(crate) fn worklet_start(
    worklet: *mut bare_worklet_t,
    filename: String,
    source: Option<Vec<u8>>,
    args: Vec<String>,
) {
    let argc = args.len() as i32;
    let argv: Vec<CString> = args
        .iter()
        .map(|arg| CString::new(arg.clone()).unwrap())
        .collect();
    let mut argv: Vec<*const c_char> = argv.iter().map(|arg| arg.as_ptr()).collect();
    let err = match source {
        Some(source) => unsafe {
            let source = uv_buf_init(source.as_ptr() as *mut _, source.len() as u32);
            bare_worklet_start(
                worklet,
                CString::new(filename).unwrap().as_ptr(),
                &source,
                argc,
                argv.as_mut_ptr(),
            )
        },
        None => unsafe {
            bare_worklet_start(
                worklet,
                CString::new(filename).unwrap().as_ptr(),
                null_mut(),
                argc,
                argv.as_mut_ptr(),
            )
        },
    };
    assert!(err == 0);
}

pub(crate) fn worklet_suspend(worklet: *mut bare_worklet_t, linger: i32) {
    let err = unsafe { bare_worklet_suspend(worklet, linger) };
    assert!(err == 0);
}

pub(crate) fn worklet_resume(worklet: *mut bare_worklet_t) {
    let err = unsafe { bare_worklet_resume(worklet) };
    assert!(err == 0);
}

pub(crate) fn worklet_wakeup(worklet: *mut bare_worklet_t, deadline: i32) {
    let err = unsafe { bare_worklet_wakeup(worklet, deadline) };
    assert!(err == 0);
}

pub(crate) fn worklet_terminate(worklet: *mut bare_worklet_t) {
    let err = unsafe { bare_worklet_terminate(worklet) };
    assert!(err == 0);
}

unsafe extern "C" fn on_suspend(_: *mut bare_t, linger: i32, data: *mut c_void) {
    let callbacks = bare_worklet_get_data(data as *mut bare_worklet_t) as *mut WorkletCallbacks;
    let callbacks = &mut *callbacks;

    if let Some(ref mut callback) = &mut callbacks.suspend {
        callback(linger);
    }
}

unsafe extern "C" fn on_wakeup(_: *mut bare_t, deadline: i32, data: *mut c_void) {
    let callbacks = bare_worklet_get_data(data as *mut bare_worklet_t) as *mut WorkletCallbacks;
    let callbacks = &mut *callbacks;

    if let Some(ref mut callback) = &mut callbacks.wakeup {
        callback(deadline);
    }
}

unsafe extern "C" fn on_idle(_: *mut bare_t, data: *mut c_void) {
    let callbacks = bare_worklet_get_data(data as *mut bare_worklet_t) as *mut WorkletCallbacks;
    let callbacks = &mut *callbacks;

    if let Some(ref mut callback) = &mut callbacks.idle {
        callback();
    }
}

unsafe extern "C" fn on_resume(_: *mut bare_t, data: *mut c_void) {
    let callbacks = bare_worklet_get_data(data as *mut bare_worklet_t) as *mut WorkletCallbacks;
    let callbacks = &mut *callbacks;

    if let Some(ref mut callback) = &mut callbacks.resume {
        callback();
    }
}
