use crate::Options;

pub trait PseudoTerminalIO {
    fn new(_options: Options) -> Self;
}
