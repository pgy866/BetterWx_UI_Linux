/// Check if running with root/sudo privileges on Linux
pub fn is_running_as_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}
