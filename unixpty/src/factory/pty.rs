use std::ffi::CString;
use std::os::fd::{AsRawFd, OwnedFd};

use rustix::fs::{CWD, Mode};
use rustix::io;
use rustix::pty::{self, OpenptFlags};

use crate::factory::err::{FactoryError, Result};

const FLAGS: OpenptFlags = OpenptFlags::CLOEXEC
    .union(OpenptFlags::RDWR)
    .union(OpenptFlags::NOCTTY);

pub fn master_fd() -> io::Result<OwnedFd> {
    let master_fd = pty::openpt(FLAGS)?;
    pty::grantpt(&master_fd)?;
    pty::unlockpt(&master_fd)?;
    Ok(master_fd)
}

pub fn slave_fd(master: &OwnedFd) -> io::Result<OwnedFd> {
    let fname = pty::ptsname(master, Vec::new())?;
    let slave = rustix::fs::openat(CWD, fname, FLAGS.into(), Mode::empty())?;

    Ok(slave)
}

pub struct Pty {
    pub master: OwnedFd,
    pub slave: OwnedFd,
}

impl Pty {
    pub fn spawn() -> io::Result<Self> {
        let master_fd = master_fd()?;
        let slave_fd = slave_fd(&master_fd)?;

        Ok(Self {
            master: master_fd,
            slave: slave_fd,
        })
    }

    pub fn set_master_nonblocking(&mut self) -> i32 {
        unsafe { super::set_to_nonblocking(self.master.as_raw_fd()) }
    }
}
