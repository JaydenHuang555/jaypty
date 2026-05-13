use std::{
    io::{Read, Write},
    sync::Arc,
};

use jaypty_core::ErrorFactory;
use jaypty_core::FactoriedErrorKind;
use jaypty_core::PollRegisteringErrorKind;
use jaypty_core::{
    EmptyResult, Options, Result, UnDefinedPseudoTerminalIO, io::PollingIntrestRegisterIO,
};
use polling::{Event, Poller};

use crate::os;
use crate::os::*;

pub struct PseudoTermainalSubsystem {
    io: SystemPseudoTerminalIO,
}

impl PseudoTermainalSubsystem {
    pub fn new(options: Options) -> Result<Self> {
        SystemPseudoTerminalIO::new(options)
            .map(|term| Self { io: term })
            .map_err(|e| ErrorFactory::kind(FactoriedErrorKind::FailedPtyCreation).with_internal(e))
    }

    pub unsafe fn register(
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

    pub fn unregister(&mut self, poller: &Arc<Poller>) -> EmptyResult {
        self.io
            .unregister(poller)
            .map_err(|e| ErrorFactory::kind(PollRegisteringErrorKind::DeRegister).with_internal(e))
    }

    pub fn reregister(
        &mut self,
        poller: &std::sync::Arc<polling::Poller>,
        intrest: Event,
        mode: Option<polling::PollMode>,
    ) -> EmptyResult {
        self.io
            .reregister(poller, intrest, mode)
            .map_err(|e| ErrorFactory::kind(PollRegisteringErrorKind::ReRegister).with_internal(e))
    }

    pub fn resize(&mut self, size: jaypty_core::PtySize) -> EmptyResult {
        self.io
            .resize(size)
            .map_err(|e| ErrorFactory::kind(FactoriedErrorKind::FailedPtyResize).with_internal(e))
    }

    pub fn latch_watchdog(&self) -> Result<SystemWatchDogIO> {
        self.io.latch_watchdog().map_err(|e| {
            ErrorFactory::kind(FactoriedErrorKind::ChildWatchDogLatchFailed).with_internal(e)
        })
    }

    pub fn consume_child(&mut self) -> Option<os::ConsumedChildConsumer> {
        self.io.consume_child()
    }

    pub fn cin(&mut self) -> &mut os::Cin {
        self.io.cin()
    }

    pub fn cout(&mut self) -> &mut os::Cout {
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
