use std::{ffi::c_void, sync::atomic::AtomicPtr};

use crate::{child::WinChildWatchdogIO, factory::ContpySpawn};

pub fn watch(spawn: &mut ContpySpawn) -> AtomicPtr<c_void> {
    let child = spawn.child.take().expect("unable to take child handle");
    let atomic = AtomicPtr::from(child);
    atomic
}
