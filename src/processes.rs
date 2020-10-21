//! Utility fn to help working with various processes, especially imagemagick

use std::process::{Child, Command, Stdio};

/// Subprocess (and run) `(magick) convert` with the given `args`
pub fn magick(args: Vec<&str>) -> Child {
    Command::new(if cfg!(windows) { "magick" } else { "convert" })
        .args(args)
        .spawn()
        .expect("Can't spawn imagemagick")
}

/// Subprocess (and run) `(magick) convert` with a piped stdin with the given `args`
pub fn pipe_to_magick(args: Vec<&str>) -> Child {
    Command::new(if cfg!(windows) { "magick" } else { "convert" })
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .expect("Can't spawn imagemagick")
}

/// Wait for `magick` to exit with the appropriate printlns. This is not designed to be composable. It's used usually as the last statement in program.
pub fn wait_for_magick(mut magick: Child) -> std::process::ExitStatus {
    // println!("Waiting for magick to exit...");
    magick.wait().expect("Failed to wait on magick")
    // println!("magick {}", exit_status);
}
