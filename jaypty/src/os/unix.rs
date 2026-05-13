use std::process::Child;

pub type SystemPseudoTerminalIO = unixpty::UnixPseudoTerminalIO;
pub type SystemWatchDogIO = unixpty::child::SignalWatchDogIO;
pub type Cin = unixpty::Cin;
pub type Cout = unixpty::Cout;
pub type ChildHandle = Child;

pub type ConsumedChildConsumer = unixpty::child::ConsumedPosixChildConsumer;
pub type ConsumedChildKiller = unixpty::child::killer::ConsumedPosixChildKiller;
