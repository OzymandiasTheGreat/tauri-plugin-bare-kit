use crate::bindings::*;

pub struct BareKitWorklet {
    worklet: BareWorklet,
    ipc: BareIPC,
}

pub struct BareKitModule {}

impl BareKitModule {
    pub fn init() -> Self {
        Self {}
    }
}
