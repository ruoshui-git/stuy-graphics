pub(crate) fn warn_unimpl(cmd: &str, line: usize) {
    eprintln!("unimplemented: {}, line: {}", cmd, line);
}

pub(crate) fn warn_disabled_in_animation(cmd: &str) {
    eprintln!("warning: command `{}` is disabled in animation mode", cmd);
}
