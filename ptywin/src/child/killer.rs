use std::{ffi::c_void, sync::atomic::AtomicPtr, thread};

use jaypty_core::child::{consume::ConsumedChildConsumer, killer::ConsumedChildKiller};
use windows_sys::Win32::System::Threading::TerminateProcess;

use crate::child::ChildHandle;

pub struct ConsumedContpyChildKiller(ChildHandle);

impl ConsumedContpyChildKiller {
    pub(crate) fn consume(self) -> ChildHandle {
        self.0
    }
}

impl ConsumedChildKiller for ConsumedContpyChildKiller {
    fn blocking(self) -> jaypty_core::Result<jaypty_core::child::ChildStatus> {
        let handle = self.consume();
        let _ = unsafe { TerminateProcess(handle.load(std::sync::atomic::Ordering::Relaxed), 1) };
        todo!()
    }

    fn nonblocking(
        self,
    ) -> std::thread::JoinHandle<jaypty_core::Result<jaypty_core::child::ChildStatus>> {
        thread::spawn(move || self.blocking())
    }
}
