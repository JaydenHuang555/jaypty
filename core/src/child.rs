use crate::io::SafePseudoTerminalRegisterIO;

pub trait ChildWatchDogIO: SafePseudoTerminalRegisterIO + Future {
    fn status(&mut self) -> Option<crate::Result<u32>>;

    fn is_dead(&self) -> bool;

    fn wait(&mut self);
}
