use std::{ffi::c_void, ptr};

use windows_sys::Win32::System::Threading::{
    InitializeProcThreadAttributeList, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, UpdateProcThreadAttribute,
};

use crate::contpy::symbols::ContpyHandle;

type Attributes = LPPROC_THREAD_ATTRIBUTE_LIST;

#[inline]
fn init_attributes(attributes: &mut Attributes) -> usize {
    let mut size: usize = 0;

    unsafe {
        // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-initializeprocthreadattributelist
        let exit_stat =
            InitializeProcThreadAttributeList(ptr::null_mut(), 1, 0, &mut size as *mut usize);
        if exit_stat > 0 {
            panic!("error, found exit code {}", exit_stat);
        }
    }

    let mut null_attributes = vec![0u8; size].into_boxed_slice();
    *attributes = null_attributes.as_mut_ptr() as _;

    unsafe {
        // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-initializeprocthreadattributelist
        let exit_stat =
            InitializeProcThreadAttributeList(*attributes, 1, 0, &mut size as *mut usize);
        if exit_stat <= 0 {
            panic!("unable to init proc thread attribute");
        }
    }
    size
}

#[inline]
fn update_attributes(handle: &ContpyHandle, attributes: &mut Attributes) {
    unsafe {
        // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute
        let exit_stat = UpdateProcThreadAttribute(
            *attributes,
            0,
            PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
            *handle as *mut c_void,
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
}

#[inline]
pub fn factory_attributes(handle: &ContpyHandle, attributes: &mut Attributes) {
    let _ = init_attributes(attributes);
    update_attributes(handle, attributes);
}
