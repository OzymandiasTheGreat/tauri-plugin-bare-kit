use rand;
use std::sync::{Arc, Mutex, RwLock};
use tauri::async_runtime::block_on;
use tokio::sync::Notify;

use super::{BareIPC, BareWorklet};

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

#[test]
fn ipc() {
    let worklet = BareWorklet::init(0, None);

    worklet.start_utf8(
        "/app.js".into(),
        "BareKit.IPC.on('data', (data) => BareKit.IPC.write(data)).write('Hello, World!')".into(),
        [].into(),
    );

    let ipc = BareIPC::init(&worklet);
    let ipc_clone = ipc.clone();

    let notify = Arc::new(Notify::new());
    let notifier = notify.clone();
    let notified = notify.notified();

    ipc.clone().read(move |data| {
        let ipc = ipc_clone.clone();
        let data = String::from_utf8(data).unwrap();
        assert!(data == "Hello, World!");

        let notifier = notifier.clone();

        ipc.clone().write(&Vec::from("Sveikas, Pasauli!"), move || {
            let notifier = notifier.clone();

            ipc.read(move |data| {
                let data = String::from_utf8(data).unwrap();
                assert!(data == "Sveikas, Pasauli!");

                notifier.notify_waiters();
            });
        });
    });

    block_on(async move {
        notified.await;
    });

    worklet.terminate();
    ipc.close();
}

#[test]
fn ipc_large_write() {
    let data_len = 4 * 1024 * 1024 as usize;
    let mut data: Vec<u8> = Vec::with_capacity(data_len);

    data.resize_with(data_len, || rand::random());

    let worklet = BareWorklet::init(0, None);

    worklet.start_utf8(
        "/app.js".into(),
        "BareKit.IPC.on('data', (data) => BareKit.IPC.write(data))".into(),
        [].into(),
    );

    let ipc = BareIPC::init(&worklet);
    let received = Arc::new(RwLock::new(vec![] as Vec<u8>));
    let buffer = received.clone();
    let notify_write = Arc::new(Notify::new());
    let notifier_write = notify_write.clone();
    let notified_write = notify_write.notified();

    ipc.clone().write(&data, move || {
        notifier_write.notify_waiters();
    });

    block_on(async move {
        notified_write.await;
    });

    let notify_read = Arc::new(Notify::new());
    let notifier_read = notify_read.clone();

    while buffer.read().unwrap().len() < data_len {
        let buffer = buffer.clone();
        let notifier_read = notifier_read.clone();
        let notified_read = notify_read.notified();

        ipc.clone().read(move |chunk| {
            buffer.write().unwrap().extend(&chunk);
            notifier_read.notify_waiters();
        });
        block_on(async move {
            notified_read.await;
        });
    }

    assert!(data == *received.clone().read().unwrap());
    worklet.terminate();
    ipc.close();
}
