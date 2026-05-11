use jaypty_error::{OsResult, SystemError};
use jaysync::io::waking::{WakingNonBlockingPipeReader, WakingNonBlockingPipeWriter};
use jwinpipe::polling::{PollingWakingNonBlockingPipeReader, PollingWakingNonBlockingPipeWriter};
use miow::pipe::{AnonRead, AnonWrite};

use super::ContpySpawn;
use crate::RegisteredPoll;

pub(crate) type W = PollingWakingNonBlockingPipeWriter<AnonWrite>;
pub(crate) type R = PollingWakingNonBlockingPipeReader<AnonRead>;

const PIPE_CAPTICITY: usize = 1024;

#[inline]
pub(crate) fn cout(spawn: &mut ContpySpawn) -> OsResult<R> {
    Ok(R::new(
        spawn
            .cout
            .take()
            .ok_or(SystemError::UnableToTakeIOFromSpawn)?,
        PIPE_CAPTICITY,
    ))
}

#[inline]
pub(crate) fn cin(spawn: &mut ContpySpawn) -> OsResult<W> {
    Ok(W::new(
        spawn
            .cin
            .take()
            .ok_or(SystemError::UnableToTakeIOFromSpawn)?,
        PIPE_CAPTICITY,
    ))
}
