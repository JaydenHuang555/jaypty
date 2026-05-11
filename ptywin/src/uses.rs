pub(crate) use crate::symbols::ContpyHandle;
pub(crate) use crate::symbols::ContpySymbols;
pub(crate) use crate::symbols::loaded_symbols;

pub use crate::poll::RegisteredPoll;

pub type Cin = PollingWakingNonBlockingPipeWriter<AnonWrite>;
pub type Cout = PollingWakingNonBlockingPipeReader<AnonRead>;
use jwinpipe::polling::PollingWakingNonBlockingPipeReader;
use jwinpipe::polling::PollingWakingNonBlockingPipeWriter;
use miow::pipe::AnonRead;
use miow::pipe::AnonWrite;
