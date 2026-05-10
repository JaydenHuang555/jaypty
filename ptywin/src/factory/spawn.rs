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

use jaypty_core::{DefinedPseudoTerminalIO, PtySize};
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

use crate::ContpyHandle;
use crate::ContpySymbols;
use crate::child::WinChildWatchdogIO;
use crate::factory;

/// just a helper storage struct
/// of the outputs when creating a
/// contpy instance
pub struct ContpySpawn {
    /// internal windows handle
    pub handle: Option<ContpyHandle>,
    /// writer to cin
    pub cin: Option<AnonWrite>,
    /// reader to cout
    pub cout: Option<AnonRead>,
    /// child process
    pub child: Option<*mut c_void>,
}

unsafe impl Send for ContpySpawn {}

impl ContpySpawn {
    pub fn spawn(options: jaypty_core::Options) -> Self {
        let dimensions = options.dimension;
        let symbols = unsafe { ContpySymbols::instance() };
        let mut pty_handle: ContpyHandle = 0;

        let (cout, pty_output) = miow::pipe::anonymous(0).unwrap();

        let (pty_input, cin) = miow::pipe::anonymous(0).unwrap();
        let size = COORD {
            X: dimensions.columns as i16,
            Y: dimensions.rows as i16,
        };
        unsafe {
            if let result = symbols.create(
                size,
                pty_input.into_raw_handle() as HANDLE,
                pty_output.into_raw_handle() as HANDLE,
                0,
                &mut pty_handle as *mut _,
            ) && result != S_OK
            {
                log::error!("unable to create handle");
                panic!()
            }
        }
        let mut si_ex: STARTUPINFOEXW = unsafe { mem::zeroed() };
        si_ex.StartupInfo.cb = mem::size_of::<STARTUPINFOEXW>() as u32;
        si_ex.StartupInfo.lpTitle = ptr::null_mut() as PWSTR;
        si_ex.StartupInfo.dwFlags |= STARTF_USESTDHANDLES;

        factory::factory_attributes(&pty_handle, &mut si_ex.lpAttributeList);

        let creation_flags = factory::CREATION_FLAGS;
        let cmdline = factory::resolve_cmd(options.cmd);
        let mut pi_client: PROCESS_INFORMATION = unsafe { mem::zeroed() };
        let cwd = factory::resolve_cwd(options.cwd);
        unsafe {
            // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw
            let exit_stat = CreateProcessW(
                ptr::null(),
                cmdline.as_ptr() as PWSTR,
                ptr::null_mut(),
                ptr::null_mut(),
                false as i32,
                creation_flags,
                cwd,
                ptr::null(),
                &mut si_ex.StartupInfo as *mut STARTUPINFOW,
                &mut pi_client as *mut PROCESS_INFORMATION,
            );
            if exit_stat == 0 {
                panic!("unable to create win process")
            }
        }

        ContpySpawn {
            handle: Some(pty_handle as ContpyHandle),
            cin: Some(cin),
            cout: Some(cout),
            child: Some(pi_client.hProcess),
        }
    }
}
