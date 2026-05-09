pub mod error;
pub mod factory;
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

use crate::child::{ChildProcess, watchdog::ChildWatchDog};
use crate::contpy::symbols::{ContpyHandle, ContpySymbols};

pub struct ContpyIO {
    handle: ContpyHandle,
    internal: ContpySymbols,
    cin: Option<AnonWrite>,
    cout: Option<AnonRead>,
    child: ChildProcess,
}

unsafe impl Send for ContpyIO {}

impl Drop for ContpyIO {
    fn drop(&mut self) {
        unsafe {
            self.internal.close(self.handle);
        }
    }
}

impl ContpyIO {
    /// transfers the ownership of the cin writer
    pub fn take_cin(&mut self) -> AnonWrite {
        self.cin.take().unwrap()
    }

    /// transfers the ownership of the cout reader
    pub fn take_cout(&mut self) -> AnonRead {
        self.cout.take().unwrap()
    }

    pub fn spawn_child_watchdog(&self) -> ChildWatchDog {
        let watchdog = ChildWatchDog::new(&self.child);
        watchdog
    }
}

impl PseudoTerminalIO for ContpyIO {
    fn new(options: jaypty::Options) -> Self {
        let dimensions = options.dimension;
        let mut cwd = options.cwd;
        let symbols = unsafe { ContpySymbols::load().unwrap() };
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
        let mut size: usize = 0;
        let mut si_ex: STARTUPINFOEXW = unsafe { mem::zeroed() };
        si_ex.StartupInfo.cb = mem::size_of::<STARTUPINFOEXW>() as u32;
        si_ex.StartupInfo.lpTitle = ptr::null_mut() as PWSTR;
        si_ex.StartupInfo.dwFlags |= STARTF_USESTDHANDLES;

        unsafe {
            // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-initializeprocthreadattributelist
            let exit_stat =
                InitializeProcThreadAttributeList(ptr::null_mut(), 1, 0, &mut size as *mut usize);
            if exit_stat > 0 {
                panic!("error, found exit code {}", exit_stat);
            }
        }

        let mut attributes = vec![0u8; size].into_boxed_slice();
        si_ex.lpAttributeList = attributes.as_mut_ptr() as _;

        unsafe {
            // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-initializeprocthreadattributelist
            let exit_stat = InitializeProcThreadAttributeList(
                si_ex.lpAttributeList,
                1,
                0,
                &mut size as *mut usize,
            );
            if exit_stat <= 0 {
                panic!("unable to init proc thread attribute");
            }
        }

        unsafe {
            // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute
            let exit_stat = UpdateProcThreadAttribute(
                si_ex.lpAttributeList,
                0,
                PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
                pty_handle as *mut c_void,
                size_of::<ContpyHandle>(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
            if exit_stat <= 0 {
                panic!(
                    "error, unable to update thread attribute due to {}",
                    exit_stat
                )
            }
        }
        let creation_flags = EXTENDED_STARTUPINFO_PRESENT;
        let cmdline = factory::resolve_cmd(options.cmd);
        let mut pi_client: PROCESS_INFORMATION = unsafe { mem::zeroed() };
        let cwd = factory::resolve_cwd(cwd);
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

        ContpyIO {
            handle: pty_handle as ContpyHandle,
            internal: symbols,
            cin: Some(cin),
            cout: Some(cout),
            child: ChildProcess::new(pi_client.hProcess),
        }
    }
}
