use rand;
use std::sync::{Arc, Mutex};

use super::{BareIPC, BareWorklet};

#[test]
fn ipc() {
    let worklet = BareWorklet::init(0, None);

    worklet.start_utf8(
        "/app.js".into(),
        "BareKit.IPC.on('data', (data) => BareKit.IPC.write(data)).write('Hello, World!')".into(),
        [].into(),
    );

    let ipc = BareIPC::init(worklet.clone());

    ipc.clone().read(move |data| {
        let data = String::from_utf8(data).unwrap();
        assert!(data == "Hello, World!");

        ipc.clone().write(Vec::from("Sveikas, Pasauli!"), move || {
            ipc.clone().read(move |data| {
                let data = String::from_utf8(data).unwrap();
                assert!(data == "Sveikas, Pasauli!");

                ipc.close();
                worklet.terminate();
            });
        });
    });
}

#[test]
fn ipc_large_write() {
    let mut data: Vec<u8> = Vec::with_capacity(4 * 1024 * 1024);

    data.resize_with(4 * 1024 * 1024, || rand::random());

    let worklet = BareWorklet::init(0, None);

    worklet.start_utf8(
        "/app.js".into(),
        "BareKit.IPC.on('data', (data) => BareKit.IPC.write(data))".into(),
        [].into(),
    );

    let ipc = BareIPC::init(worklet.clone());

    ipc.clone().write(data.clone(), move || {
        ipc.clone().read(move |returned| {
            assert!(data == returned);

            ipc.close();
            worklet.terminate();
        });
    });
}

#[test]
fn suspend_resume() {
    let worklet = BareWorklet::init(0, None);

    worklet.start_utf8(
        "/app.js".into(),
        "console.log('Hello, World!')".into(),
        [].into(),
    );

    let suspend_called = Arc::new(Mutex::new(false));
    let resume_called = Arc::new(Mutex::new(false));
    let suspend_cloned = suspend_called.clone();
    let resume_cloned = resume_called.clone();

    worklet.on_suspend(move |_linger| {
        *suspend_cloned.lock().unwrap() = true;
    });
    worklet.on_resume(move || {
        *resume_cloned.lock().unwrap() = true;
    });

    worklet.suspend(10);

    std::thread::sleep(std::time::Duration::from_millis(100));

    worklet.resume();

    std::thread::sleep(std::time::Duration::from_millis(100));

    worklet.terminate();

    assert!(*suspend_called.lock().unwrap());
    assert!(*resume_called.lock().unwrap());
}
