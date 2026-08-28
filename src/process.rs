//! Cross-platform process helpers — Windows equivalents for Unix process groups,
//! `/dev/null`, and `ps`/`kill` patterns used by local jobs and git.

use std::ffi::OsStr;
use std::process::{Command, Stdio};

/// Build a [`Command`] with Windows `CREATE_NO_WINDOW` already applied.
pub fn command(program: impl AsRef<OsStr>) -> Command {
    let mut cmd = Command::new(program);
    hide_window(&mut cmd);
    cmd
}

/// Build a [`tokio::process::Command`] with Windows `CREATE_NO_WINDOW` already applied.
pub fn tokio_command(program: impl AsRef<OsStr>) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(program);
    hide_tokio_window(&mut cmd);
    cmd
}

/// Spawn a child without flashing a console window on Windows. No-op elsewhere.
pub fn hide_window(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = &mut *cmd;
}

/// [`hide_window`] for [`tokio::process::Command`].
pub fn hide_tokio_window(cmd: &mut tokio::process::Command) {
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = &mut *cmd;
}

/// The platform null device (`/dev/null` or `NUL`).
pub fn null_device() -> &'static str {
    if cfg!(windows) {
        "NUL"
    } else {
        "/dev/null"
    }
}

/// Apply platform-specific spawn flags for detached background jobs.
pub fn configure_detached(cmd: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
    }
}

/// Whether the process identified by `pid` is still alive (not a zombie).
pub fn pid_alive(pid: &str) -> bool {
    #[cfg(unix)]
    {
        match Command::new("ps")
            .args(["-o", "stat=", "-p", pid])
            .stderr(Stdio::null())
            .output()
        {
            Ok(o) if o.status.success() => {
                let stat = String::from_utf8_lossy(&o.stdout);
                let stat = stat.trim();
                !stat.is_empty() && !stat.starts_with('Z')
            }
            _ => false,
        }
    }
    #[cfg(windows)]
    {
        let mut cmd = Command::new("tasklist");
        cmd.args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .stderr(Stdio::null());
        hide_window(&mut cmd);
        cmd.output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| {
                let out = String::from_utf8_lossy(&o.stdout);
                out.contains(pid) && !out.contains("No tasks")
            })
            .unwrap_or(false)
    }
}

/// Terminate a process tree rooted at `pid`. Returns false when nothing could be
/// killed.
pub fn terminate_tree(pid: &str) -> bool {
    #[cfg(unix)]
    {
        let group = Command::new("kill")
            .args(["-TERM", "--", &format!("-{pid}")])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if group {
            return true;
        }
        Command::new("kill")
            .args(["-TERM", pid])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        let mut cmd = Command::new("taskkill");
        cmd.args(["/PID", pid, "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        hide_window(&mut cmd);
        cmd.status().map(|s| s.success()).unwrap_or(false)
    }
}

/// Kill a child process and its descendants (used when git transport children
/// outlive the parent).
pub fn kill_process_tree(child: &mut std::process::Child) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        unsafe {
            libc::kill(-(child.id() as i32), libc::SIGKILL);
        }
        Ok(())
    }
    #[cfg(windows)]
    {
        let pid = child.id().to_string();
        let _ = terminate_tree(&pid);
        child.kill()
    }
}

/// Resolve `bash` for local experiment runs. On Windows, Git for Windows is the
/// expected provider.
pub fn find_bash() -> Option<std::path::PathBuf> {
    crate::local::shell_env::find_on_path("bash").or_else(git_bash_fallback)
}

fn git_bash_fallback() -> Option<std::path::PathBuf> {
    #[cfg(windows)]
    {
        for base in [
            std::env::var_os("ProgramFiles").map(std::path::PathBuf::from),
            std::env::var_os("ProgramFiles(x86)").map(std::path::PathBuf::from),
        ]
        .into_iter()
        .flatten()
        {
            let candidate = base.join("Git").join("bin").join("bash.exe");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_device_is_platform_specific() {
        if cfg!(windows) {
            assert_eq!(null_device(), "NUL");
        } else {
            assert_eq!(null_device(), "/dev/null");
        }
    }
}
