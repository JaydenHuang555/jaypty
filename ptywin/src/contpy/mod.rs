pub mod error;
pub mod internal;

use jwinutil::sanitize_string;
use polling::{Event, PollMode, Poller};
use std::io::{PipeReader, Read, Write};
use std::sync::{Arc, mpsc};
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

use jaypty::PtySize;
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

use crate::pipe::RegisteredTask;
use crate::{
    child::ChildExitWatchDog,
    contpy::internal::{ContpyHandle, ContpyInternal},
    pipe::{input::NonBlockingPipeWriter, output::NonBlockingPipeReader},
};

pub struct ContpyIO {
    handle: ContpyHandle,
    internal: ContpyInternal,
    cin: NonBlockingPipeWriter,
    cout: NonBlockingPipeReader,
    child_exit_watchdog: ChildExitWatchDog,
}

unsafe impl Send for ContpyIO {}

impl Drop for ContpyIO {
    fn drop(&mut self) {
        self.internal.close(self.handle);
    }
}

impl ContpyIO {
    pub fn new(dimensions: PtySize) -> ContpyIO {
        let internal = ContpyInternal::load().unwrap();
        let mut pty_handle: ContpyHandle = 0;

        // open a pipe for writing
        // the connect_output is our reader for the terminal
        // pty_output is the actual writer writing to pty
        let (cout, pty_output) = miow::pipe::anonymous(0).unwrap();

        // open a pipe for reading
        // the pty input is the reader of the output
        // that is provided by the
        // connected input
        let (pty_input, cin) = miow::pipe::anonymous(0).unwrap();
        let size = COORD {
            X: dimensions.columns as i16,
            Y: dimensions.rows as i16,
        };
        if let result = internal.create(
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
        let mut size: usize = 0;
        let mut si_ex: STARTUPINFOEXW = unsafe { mem::zeroed() };
        si_ex.StartupInfo.cb = mem::size_of::<STARTUPINFOEXW>() as u32;
        si_ex.StartupInfo.lpTitle = ptr::null_mut() as PWSTR;
        si_ex.StartupInfo.dwFlags |= STARTF_USESTDHANDLES;

        unsafe {
            let exit_stat =
                InitializeProcThreadAttributeList(ptr::null_mut(), 1, 0, &mut size as *mut usize);
            if exit_stat > 0 {
                panic!("error, found exit code {}", exit_stat);
            }
        }

        let mut attributes = vec![0u8; size].into_boxed_slice();
        si_ex.lpAttributeList = attributes.as_mut_ptr() as _;

        unsafe {
            let exit_stat = UpdateProcThreadAttribute(
                si_ex.lpAttributeList,
                0,
                PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
                pty_handle as *mut c_void,
                size_of::<ContpyHandle>(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
            if exit_stat > 0 {
                panic!(
                    "error, unable to update thread attribute due to {}",
                    exit_stat
                )
            }
        }
        let creation_flags = EXTENDED_STARTUPINFO_PRESENT;
        let cmdline = sanitize_string(&"powershell -Command dir");
        let mut pi_client: PROCESS_INFORMATION = unsafe { mem::zeroed() };
        let cwd = sanitize_string(&"C:\\Users\\Jshizzle");
        unsafe {
            let exit_stat = CreateProcessW(
                ptr::null(),
                cmdline.as_ptr() as PWSTR,
                ptr::null_mut(),
                ptr::null_mut(),
                false as i32,
                creation_flags,
                ptr::null_mut() as *const c_void,
                ptr::null(),
                &mut si_ex.StartupInfo as *mut STARTUPINFOW,
                &mut pi_client as *mut PROCESS_INFORMATION,
            );
            if exit_stat == 0 {
                panic!("unable to create win process")
            }
        }

        let c_out = NonBlockingPipeReader::new(cout, 1024);
        let c_in = NonBlockingPipeWriter::new(cin, 1024);

        ContpyIO {
            handle: pty_handle as ContpyHandle,
            internal,
            cin: c_in,
            cout: c_out,
            child_exit_watchdog: ChildExitWatchDog::new(pi_client.hProcess),
        }
    }

    pub fn update_child_alive(&mut self) -> bool {
        self.child_exit_watchdog.update_child_running()
    }

    pub fn wait(&mut self) {
        while self.child_exit_watchdog.update_child_running() {
            thread::sleep(Duration::from_millis(500));
        }
    }

    pub fn register_all(&mut self, poller: &Arc<Poller>, event: Event, mode: PollMode) {
        self.cin.register(poller, event, mode);
        self.cout.register(poller, event, mode);
    }

    pub fn register_cout(&mut self, poller: &Arc<Poller>, event: Event, mode: PollMode) {
        self.cout.register(poller, event, mode);
    }

    pub fn register_cin(&mut self, poller: &Arc<Poller>, event: Event, mode: PollMode) {
        self.cin.register(poller, event, mode);
    }

    pub fn reader(&mut self) -> &mut NonBlockingPipeReader {
        &mut self.cout
    }

    pub fn writer(&mut self) -> &mut NonBlockingPipeWriter {
        &mut self.cin
    }
}

impl Write for ContpyIO {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.cin.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.cin.flush()
    }
}

impl Read for ContpyIO {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.cout.read(buf)
    }
}
