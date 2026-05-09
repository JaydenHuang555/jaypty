

use jaysync::io::waking::{WakingNonBlockingPipeReader, WakingNonBlockingPipeWriter};

use super::ContpySpawn;
use crate::RegisteredPoll;

const PIPE_CAPTICITY: usize = 1024;

#[inline]
pub(crate) fn cout(spawn: &mut ContpySpawn) -> WakingNonBlockingPipeReader<RegisteredPoll> {
    WakingNonBlockingPipeReader::new(
        spawn.cout.take().unwrap(),
        PIPE_CAPTICITY,
        RegisteredPoll::default(),
    )
}

#[inline]
pub(crate) fn cin(spawn: &mut ContpySpawn) -> WakingNonBlockingPipeWriter<RegisteredPoll> {
    WakingNonBlockingPipeWriter::new(
        spawn.cin.take().unwrap(),
        PIPE_CAPTICITY,
        RegisteredPoll::default(),
    )
}
