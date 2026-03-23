//! Unix PTY backend implementation using nix crate.

use std::ffi::CString;
use std::io;
use std::os::unix::io::RawFd;
use std::path::Path;
use std::process::ExitStatus;

use nix::fcntl::{open, OFlag};
use nix::libc;
use nix::pty::openpty;
use nix::sys::signal::{kill, Signal};
use nix::sys::stat::Mode;
use nix::sys::termios::{self, SetArg, Termios};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{close, dup2, execvp, fork, read, setsid, write, ForkResult, Pid};

use super::PtyBackend;

pub struct UnixPty {
    master_fd: RawFd,
    child_pid: Pid,
}

impl UnixPty {
    pub fn spawn(
        shell: &Path,
        args: &[String],
        cwd: &Path,
        env: &std::collections::HashMap<String, String>,
        cols: u16,
        rows: u16,
    ) -> io::Result<Self> {
        // Create PTY pair
        let pty = openpty(None, None)
            .map_err(|e| io::Error::other(format!("openpty: {e}")))?;

        let master_fd = pty.master;
        let slave_fd = pty.slave;

        // Set initial terminal size
        let winsize = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        unsafe {
            libc::ioctl(master_fd, libc::TIOCSWINSZ, &winsize);
        }

        // Fork
        match unsafe { fork() } {
            Ok(ForkResult::Child) => {
                // Child process
                // Close master fd
                let _ = close(master_fd);

                // Create new session
                setsid().map_err(|e| io::Error::other(format!("setsid: {e}")))?;

                // Set controlling terminal
                unsafe {
                    libc::ioctl(slave_fd, libc::TIOCSCTTY as _, 0);
                }

                // Redirect stdio to slave PTY
                dup2(slave_fd, 0).map_err(|e| io::Error::other(format!("dup2 stdin: {e}")))?;
                dup2(slave_fd, 1).map_err(|e| io::Error::other(format!("dup2 stdout: {e}")))?;
                dup2(slave_fd, 2).map_err(|e| io::Error::other(format!("dup2 stderr: {e}")))?;

                // Close original slave fd if it's not one of stdio
                if slave_fd > 2 {
                    let _ = close(slave_fd);
                }

                // Change directory
                std::env::set_current_dir(cwd).ok();

                // Set environment
                std::env::set_var("TERM", "xterm-256color");
                std::env::set_var("COLORTERM", "truecolor");
                std::env::set_var("WIT_TERM", "1");
                for (key, value) in env {
                    std::env::set_var(key, value);
                }

                // Build args for execvp
                let shell_str = shell.to_string_lossy();
                let c_shell = CString::new(shell_str.as_ref())
                    .map_err(|e| io::Error::other(format!("CString shell: {e}")))?;

                let mut c_args: Vec<CString> = vec![c_shell.clone()];
                for arg in args {
                    c_args.push(
                        CString::new(arg.as_str())
                            .map_err(|e| io::Error::other(format!("CString arg: {e}")))?,
                    );
                }

                // Execute shell
                execvp(&c_shell, &c_args)
                    .map_err(|e| io::Error::other(format!("execvp: {e}")))?;

                unreachable!()
            }
            Ok(ForkResult::Parent { child }) => {
                // Parent process
                // Close slave fd
                let _ = close(slave_fd);

                Ok(UnixPty {
                    master_fd,
                    child_pid: child,
                })
            }
            Err(e) => Err(io::Error::other(format!("fork: {e}"))),
        }
    }
}

unsafe impl Sync for UnixPty {}

impl PtyBackend for UnixPty {
    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        match read(self.master_fd, buf) {
            Ok(n) => Ok(n),
            Err(nix::errno::Errno::EIO) => Ok(0), // EOF - child exited
            Err(nix::errno::Errno::EINTR) => Ok(0),
            Err(e) => Err(io::Error::other(format!("read: {e}"))),
        }
    }

    fn write(&self, data: &[u8]) -> io::Result<usize> {
        write(self.master_fd, data)
            .map_err(|e| io::Error::other(format!("write: {e}")))
    }

    fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        let winsize = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let ret = unsafe { libc::ioctl(self.master_fd, libc::TIOCSWINSZ, &winsize) };
        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    fn child_pid(&self) -> u32 {
        self.child_pid.as_raw() as u32
    }

    fn is_alive(&self) -> bool {
        match waitpid(self.child_pid, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::StillAlive) => true,
            _ => false,
        }
    }

    fn wait(&self) -> io::Result<ExitStatus> {
        loop {
            match waitpid(self.child_pid, None) {
                Ok(WaitStatus::Exited(_, code)) => {
                    return std::process::Command::new("sh")
                        .arg("-c")
                        .arg(format!("exit {code}"))
                        .status();
                }
                Ok(WaitStatus::Signaled(_, _, _)) => {
                    return std::process::Command::new("sh")
                        .arg("-c")
                        .arg("exit 128")
                        .status();
                }
                Ok(_) => continue,
                Err(e) => return Err(io::Error::other(format!("waitpid: {e}"))),
            }
        }
    }
}

impl Drop for UnixPty {
    fn drop(&mut self) {
        // Send SIGHUP to child process
        let _ = kill(self.child_pid, Signal::SIGHUP);
        // Close master fd
        let _ = close(self.master_fd);
        // Reap zombie
        let _ = waitpid(self.child_pid, Some(WaitPidFlag::WNOHANG));
    }
}
