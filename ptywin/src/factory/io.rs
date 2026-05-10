use jaysync::io::waking::{WakingNonBlockingPipeReader, WakingNonBlockingPipeWriter};
use jwinpipe::polling::{PollingWakingNonBlockingPipeReader, PollingWakingNonBlockingPipeWriter};
use miow::pipe::{AnonRead, AnonWrite};

use super::ContpySpawn;
use crate::RegisteredPoll;

pub(crate) type W = PollingWakingNonBlockingPipeWriter<AnonWrite>;
pub(crate) type R = PollingWakingNonBlockingPipeReader<AnonRead>;

const PIPE_CAPTICITY: usize = 1024;

#[inline]
pub(crate) fn cout(spawn: &mut ContpySpawn) -> R {
    R::new(spawn.cout.take().unwrap(), PIPE_CAPTICITY)
}

#[inline]
pub(crate) fn cin(spawn: &mut ContpySpawn) -> W {
    W::new(spawn.cin.take().unwrap(), PIPE_CAPTICITY)
}
