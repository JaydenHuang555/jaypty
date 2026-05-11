use ptywin::{child::WinChildWatchdogIO, io::ContpyPseudoTerminalIO};

pub type SystemPseudoTerminalIO = ContpyPseudoTerminalIO;
pub type SystemWatchDogIO = WinChildWatchdogIO;

pub use ptywin::Cin;
pub use ptywin::Cout;

#[cfg(test)]
mod tests {

    #[test]
    fn using_win() {
        #[cfg(any(windows, feature = "win"))]
        {
            assert_eq!(1, 1)
        }
        #[cfg(not(any(windows, feature = "win")))]
        {
            assert_eq!(
                "is this win??",
                "compiled window features when build did not want win"
            )
        }
    }
}
