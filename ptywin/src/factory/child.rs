use std::{ffi::c_void, sync::atomic::AtomicPtr};

use jaypty_error::{OsResult, SystemError};

use crate::{child::WinChildWatchdogIO, factory::ContpySpawn};

pub fn watch(spawn: &mut ContpySpawn) -> OsResult<AtomicPtr<c_void>> {
    let child = spawn
        .child
        .take()
        .ok_or(SystemError::UnableToTakeChildHandleFromSpawn)?;
    let atomic = AtomicPtr::from(child);
    Ok(atomic)
}
