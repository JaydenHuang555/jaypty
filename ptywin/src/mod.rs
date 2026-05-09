pub mod error;
pub mod factory;
pub mod io;
pub mod symbols;

use jwinutil::sanitize_string;
use miow::pipe::{AnonRead, AnonWrite};
use polling::{Event, PollMode, Poller};
use std::io::{PipeReader, Read, Write};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;
use std::time::Duration;
use std::{ffi::c_void, mem, os::windows::io::IntoRawHandle, ptr};
use windows_sys::{
    Win32::{
        Foundation::S_OK,
        System::Threading::{
            CreateProcessW, EXTENDED_STARTUPINFO_PRESENT, InitializeProcThreadAttributeList,
            PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, PROCESS_INFORMATION, STARTF_USESTDHANDLES,
            STARTUPINFOEXW, STARTUPINFOW, UpdateProcThreadAttribute,
        },
    },
    core::PWSTR,
};

use jaypty::{PseudoTerminalIO, PtySize};
use windows_sys::{
    Win32::{
        Foundation::HANDLE,
        System::{
            Console::{COORD, HPCON},
            LibraryLoader::{GetProcAddress, LoadLibraryW},
        },
    },
    core::HRESULT,
    s, w,
};

pub use error::Error;
pub use error::ErrorKind;
pub use error::Result;

pub(crate) use symbols::ContpyHandle;
pub(crate) use symbols::ContpySymbols;
pub(crate) use symbols::loaded_symbols;
