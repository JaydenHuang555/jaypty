use std::{
    io::{Read, Write},
    sync::Arc,
};

use jaypty_core::{
    EmptyResult, Error, ErrorFactory, FactoriedError, FactoriedErrorKind, Options,
    PollRegisteringErrorKind, Result, UnDefinedPseudoTerminalIO, io::PollingIntrestRegisterIO,
};
use polling::{Event, Poller};

use crate::{SystemPseudoTerminalIO, SystemWatchDogIO, os};

pub struct PseudoTermainalSubsystem {
    io: SystemPseudoTerminalIO,
}

impl PseudoTermainalSubsystem {
    pub fn new(options: Options) -> Result<Self> {
        SystemPseudoTerminalIO::new(options)
            .map(|term| Self { io: term })
            .map_err(|e| ErrorFactory::kind(FactoriedErrorKind::FailedPtyCreation).with_internal(e))
    }

    unsafe fn register(
        &mut self,
        poller: &std::sync::Arc<polling::Poller>,
        intrest: Event,
        mode: Option<polling::PollMode>,
    ) -> EmptyResult {
        unsafe {
            self.io.register(poller, intrest, mode).map_err(|e| {
                ErrorFactory::kind(PollRegisteringErrorKind::Register).with_internal(e)
            })
        }
    }

    fn unregister(&mut self, poller: &Arc<Poller>) -> EmptyResult {
        self.io
            .unregister(poller)
            .map_err(|e| ErrorFactory::kind(PollRegisteringErrorKind::DeRegister).with_internal(e))
    }

    fn reregister(
        &mut self,
        poller: &std::sync::Arc<polling::Poller>,
        intrest: Event,
        mode: Option<polling::PollMode>,
    ) -> EmptyResult {
        self.io
            .reregister(poller, intrest, mode)
            .map_err(|e| ErrorFactory::kind(PollRegisteringErrorKind::ReRegister).with_internal(e))
    }

    fn resize(&mut self, size: jaypty_core::PtySize) -> EmptyResult {
        self.io
            .resize(size)
            .map_err(|e| ErrorFactory::kind(FactoriedErrorKind::FailedPtyResize).with_internal(e))
    }

    fn latch_watchdog(&self) -> Result<SystemWatchDogIO> {
        self.io.latch_watchdog().map_err(|e| {
            ErrorFactory::kind(FactoriedErrorKind::ChildWatchDogLatchFailed).with_internal(e)
        })
    }

    fn kill_child(&mut self) -> EmptyResult {
        self.io
            .kill_child()
            .map_err(|e| ErrorFactory::kind(FactoriedErrorKind::KillChildFailed).with_internal(e))
    }

    fn cin(&mut self) -> &mut os::Cin {
        self.io.cin()
    }

    fn cout(&mut self) -> &mut os::Cout {
        self.io.cout()
    }
}

impl Write for PseudoTermainalSubsystem {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.io.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.io.flush()
    }
}

impl Read for PseudoTermainalSubsystem {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.io.read(buf)
    }
}
