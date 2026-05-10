use crate::error::LoadErrorKind;

use super::Error;
use super::Result;
use std::mem;
use std::sync::OnceLock;

use jaypty::error::ErrorKind;
use windows_sys::Win32::System::Console::ClosePseudoConsole;
use windows_sys::Win32::System::Console::CreatePseudoConsole;
use windows_sys::Win32::System::Console::ResizePseudoConsole;
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

pub(crate) type ContpyHandle = HPCON;

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

#[inline]
pub unsafe fn loaded_symbols() -> &'static ContpySymbols {
    INSTANCE.get_or_init(|| unsafe { ContpySymbols::load() })
}

unsafe impl Send for ContpySymbols {}

pub static INSTANCE: OnceLock<ContpySymbols> = OnceLock::new();

impl ContpySymbols {
    #[inline]
    pub unsafe fn instance() -> &'static Self {
        unsafe { loaded_symbols() }
    }

    /// first trys to load the symbols from contpy.dll
    /// if it can't load from contpy.dll
    /// load the win api symbols
    unsafe fn load() -> ContpySymbols {
        unsafe {
            Self::load_dll().unwrap_or(ContpySymbols {
                create: CreatePseudoConsole,
                resize: ResizePseudoConsole,
                close: ClosePseudoConsole,
            })
        }
    }

    /// loads symbols from contpy.dll
    /// assuming it is in the same directory
    /// as the executable binary
    unsafe fn load_dll() -> Result<ContpySymbols> {
        type LoadedFn = unsafe extern "system" fn() -> isize;
        unsafe {
            let hmodule = LoadLibraryW(w!("conpty.dll"));
            if hmodule.is_null() {
                return Error::from(Error::failed_loading_pty(
                    LoadErrorKind::UnableToFindContpyDll,
                ))
                .context("unable to load contpy.dll")
                .into();
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

    /// Creates a "Pseudo Console" (ConPTY).
    /// HRESULT WINAPI CreatePseudoConsole(
    /// _In_ COORD size,        // ConPty Dimensions
    /// _In_ HANDLE hInput,     // ConPty Input
    /// _In_ HANDLE hOutput,    // ConPty Output
    /// _In_ DWORD dwFlags,     // ConPty Flags
    /// _Out_ HPCON* phPC);     // ConPty Reference
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

    /// Resizes the given ConPTY to the specified size, in characters.
    /// HRESULT WINAPI ResizePseudoConsole(_In_ HPCON hPC, _In_ COORD size);
    pub unsafe fn resize(&self, handle: ContpyHandle, size: COORD) -> HRESULT {
        unsafe { (self.resize)(handle, size) }
    }

    /// Closes the ConPTY and all associated handles. Client applications attached
    /// to the ConPTY will also terminated.
    /// VOID WINAPI ClosePseudoConsole(_In_ HPCON hPC);
    pub unsafe fn close(&self, handle: ContpyHandle) {
        unsafe { (self.close)(handle) }
    }
}
