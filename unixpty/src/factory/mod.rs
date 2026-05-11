pub mod account;
pub mod err;
pub(crate) mod pty;

use std::{
    ffi::{CString, c_int},
    os::{
        fd::{AsRawFd, OwnedFd},
        unix::{ffi::OsStrExt, process::CommandExt},
    },
    path::PathBuf,
    process::Command,
};

pub(crate) use account::Account;
use jaypty_core::Options;
use libc::{F_GETFL, F_SETFL, O_NONBLOCK, TIOCSCTTY, fcntl};

use crate::{Error, Pty, Result};

pub fn cmdlify(options: &Options) -> Result<Command> {
    let account = Account::env_account()?;

    Ok(options
        .cmd
        .as_ref()
        .map(|input| {
            let mut cmd = Command::new(input.process_name());
            for arg in input.args() {
                cmd.arg(arg);
            }
            cmd
        })
        .unwrap_or(Command::new(account.shell)))
}

pub fn build_cmd(options: &Options, pty: &Pty) -> Result<Command> {
    let master = pty.master.as_raw_fd();
    let slave = pty.slave.as_raw_fd();

    let mut process_builder = cmdlify(&options)?;
    process_builder.stdin(
        pty.slave
            .try_clone()
            .map_err(|e| Error::UnableToCloneHandle(e))?,
    );
    process_builder.stdout(
        pty.slave
            .try_clone()
            .map_err(|e| Error::UnableToCloneHandle(e))?,
    );
    process_builder.stderr(
        pty.slave
            .try_clone()
            .map_err(|e| Error::UnableToCloneHandle(e))?,
    );

    let mut cwd = options.cwd.clone();

    unsafe {
        process_builder.pre_exec(move || {
            cwd.take().map(|cwd| {
                CString::new(cwd.as_os_str().as_bytes()).ok().map(|str| {
                    libc::chdir(str.as_ptr());
                });
            });

            libc::ioctl(slave, TIOCSCTTY);

            libc::close(master);
            libc::close(slave);

            libc::signal(libc::SIGCHLD, libc::SIG_DFL);
            libc::signal(libc::SIGHUP, libc::SIG_DFL);
            libc::signal(libc::SIGINT, libc::SIG_DFL);
            libc::signal(libc::SIGQUIT, libc::SIG_DFL);
            libc::signal(libc::SIGTERM, libc::SIG_DFL);
            libc::signal(libc::SIGALRM, libc::SIG_DFL);

            Ok(())
        });
    }

    Ok(process_builder)
}

pub unsafe fn set_to_nonblocking(fd: c_int) -> i32 {
    unsafe { fcntl(fd, F_SETFL | fcntl(fd, F_GETFL) | O_NONBLOCK) }
}
