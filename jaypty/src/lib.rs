mod os;

pub use jaypty_core::command::Command;
pub use jaypty_core::io::*;
pub use jaypty_core::tokens::Token;
pub use jaypty_core::{Options, PtySize};

pub use os::{SystemPseudoTerminalIO, SystemWatchDogIO};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
