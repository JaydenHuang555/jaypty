use std::{
    ffi::c_void,
    path::{Path, PathBuf},
    ptr,
};

use crate::util::sanitize_string;
use jaypty_core::command::Command;
use windows_sys::{Win32::System::Threading::EXTENDED_STARTUPINFO_PRESENT, core::PWSTR};

pub const DEFAULT_SHELL: &'static str = "cmd.exe";

pub const CREATION_FLAGS: u32 = EXTENDED_STARTUPINFO_PRESENT;

#[inline]
pub(crate) fn resolve_cwd(cwd_input: Option<PathBuf>) -> *const c_void {
    cwd_input
        .map(|cwd| sanitize_string(&cwd.as_os_str()))
        .map_or_else(ptr::null, |santized| santized.as_ptr()) as *const c_void
}

#[inline]
pub(crate) fn resolve_cmd(cmd: Option<Command>) -> Vec<u16> {
    cmd.map(|c| {
        let pname = c.process_name().clone();

        let mut builder = String::new();
        builder.push_str(pname.as_str());

        for arg in c.args().iter() {
            push_escaped_arg(&mut builder, arg);
        }

        sanitize_string(&builder)
    })
    .unwrap_or(sanitize_string(&DEFAULT_SHELL))
}

fn push_escaped_arg(cmd: &mut String, arg: &str) {
    let arg_bytes = arg.as_bytes();
    let quote = arg_bytes.iter().any(|c| *c == b' ' || *c == b'\t') || arg_bytes.is_empty();
    if quote {
        cmd.push('"');
    }

    let mut backslashes: usize = 0;
    for x in arg.chars() {
        if x == '\\' {
            backslashes += 1;
        } else {
            if x == '"' {
                // Add n+1 backslashes to total 2n+1 before internal '"'.
                cmd.extend((0..=backslashes).map(|_| '\\'));
            }
            backslashes = 0;
        }
        cmd.push(x);
    }

    if quote {
        // Add n backslashes to total 2n before ending '"'.
        cmd.extend((0..backslashes).map(|_| '\\'));
        cmd.push('"');
    }
}

#[cfg(test)]
mod tests {
    use crate::factory::options::push_escaped_arg;

    /// modified code from
    /// https://github.com/alacritty/alacritty/blob/master/alacritty_terminal/src/tty/windows/mod.rs
    #[test]
    fn test_escape() {
        let test_set = vec![
            ("abc", "abc"),
            ("", "\"\""),
            (" ", "\" \""),
            ("ab c", "\"ab c\""),
            ("ab\tc", "\"ab\tc\""),
            ("ab\\c", "ab\\c"),
            ("ab\"c", "ab\\\"c"),
            ("\"", "\\\""),
            ("a\"b\"c", "a\\\"b\\\"c"),
            ("ab \"c", "\"ab \\\"c\""),
            ("a \"b\" c", "\"a \\\"b\\\" c\""),
            ("C:\\Program Files\\", "\"C:\\Program Files\\\\\""),
            ("C:\\Program Files\\a.txt", "\"C:\\Program Files\\a.txt\""),
            (
                r#"sh -c "cd /home/user; ARG='abc' \""'${SHELL:-sh}" -i -c '"'echo hello'""#,
                r#""sh -c \"cd /home/user; ARG='abc' \\\"\"'${SHELL:-sh}\" -i -c '\"'echo hello'\"""#,
            ),
        ];

        for (input, expected) in test_set {
            let mut escaped_arg = String::new();
            push_escaped_arg(&mut escaped_arg, input);
            assert_eq!(escaped_arg, expected, "Failed for input: {}", input);
        }
    }
}
