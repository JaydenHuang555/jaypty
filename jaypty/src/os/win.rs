use jaypty_core::Options;
use ptywin::{child::WinChildWatchdogIO, io::ContpyPseudoTerminalIO};

pub type SystemPseudoTerminalIO = ContpyPseudoTerminalIO;
pub type SystemWatchDogIO = WinChildWatchdogIO;
