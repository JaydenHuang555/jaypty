use super::Error;
use super::Result;
use std::cell::OnceCell;
use std::mem;
use std::sync::OnceLock;

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

pub type ContpyHandle = HPCON;

/// internal window functions as defined by
/// https://devblogs.microsoft.com/commandline/windows-command-line-introducing-the-windows-pseudo-console-conpty/#using-the-conpty-api

/// Creates a "Pseudo Console" (ConPTY).
/// HRESULT WINAPI CreatePseudoConsole(
/// _In_ COORD size,        // ConPty Dimensions
/// _In_ HANDLE hInput,     // ConPty Input
/// _In_ HANDLE hOutput,    // ConPty Output
/// _In_ DWORD dwFlags,     // ConPty Flags
/// _Out_ HPCON* phPC);     // ConPty Reference

type CreatePseudoConsoleFn =
    unsafe extern "system" fn(COORD, HANDLE, HANDLE, u32, *mut HPCON) -> HRESULT;

/// Resizes the given ConPTY to the specified size, in characters.
/// HRESULT WINAPI ResizePseudoConsole(_In_ HPCON hPC, _In_ COORD size);

type ResizePseudoConsoleFn = unsafe extern "system" fn(HPCON, COORD) -> HRESULT;

/// Closes the ConPTY and all associated handles. Client applications attached
/// to the ConPTY will also terminated.
/// VOID WINAPI ClosePseudoConsole(_In_ HPCON hPC);

type ClosePseudoConsoleFn = unsafe extern "system" fn(HPCON);

/// util for api lib symbols
pub struct ContpySymbols {
    create: CreatePseudoConsoleFn,
    resize: ResizePseudoConsoleFn,
    close: ClosePseudoConsoleFn,
}

unsafe impl Send for ContpySymbols {}

pub static INSTANCE: OnceLock<ContpySymbols> = OnceLock::new();

impl ContpySymbols {
    #[inline]
    pub unsafe fn instance() -> &'static Self {
        INSTANCE.get_or_init(|| unsafe { Self::load().unwrap() })
    }

    /// loads symbols from contpy.dll
    /// assuming it is in the same directory
    /// as the exectuable
    ///
    /// TODO: add support for when contpy.dll is not present
    unsafe fn load() -> Result<ContpySymbols> {
        type LoadedFn = unsafe extern "system" fn() -> isize;
        unsafe {
            let hmodule = LoadLibraryW(w!("conpty.dll"));
            if hmodule.is_null() {
                return Error::load_err("unable to load contpy.dll").into();
            }
            let create = GetProcAddress(hmodule, s!("CreatePseudoConsole")).unwrap();
            let resize = GetProcAddress(hmodule, s!("ResizePseudoConsole")).unwrap();
            let close = GetProcAddress(hmodule, s!("ClosePseudoConsole")).unwrap();
            Ok({
                Self {
                    create: std::mem::transmute::<LoadedFn, CreatePseudoConsoleFn>(create),
                    resize: std::mem::transmute::<LoadedFn, ResizePseudoConsoleFn>(resize),
                    close: mem::transmute::<LoadedFn, ClosePseudoConsoleFn>(close),
                }
            })
        }
    }

    pub unsafe fn create(
        &self,
        size: COORD,
        input_handle: HANDLE,
        output_handle: HANDLE,
        flags: u32,
        reference: *mut HPCON,
    ) -> HRESULT {
        unsafe { (self.create)(size, input_handle, output_handle, flags, reference) }
    }

    pub unsafe fn resize(&self, session: ContpyHandle, size: COORD) -> HRESULT {
        unsafe { (self.resize)(session, size) }
    }

    pub unsafe fn close(&self, session: ContpyHandle) {
        unsafe { (self.close)(session) }
    }
}
