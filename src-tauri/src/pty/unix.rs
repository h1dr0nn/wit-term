//! Unix PTY backend implementation using nix crate.

use std::io;
use std::path::PathBuf;
use std::process::ExitStatus;

use super::PtyBackend;

pub struct UnixPty {
    master_fd: i32,
    child_pid: u32,
}

impl UnixPty {
    pub fn spawn(
        _shell: &PathBuf,
        _args: &[String],
        _cwd: &PathBuf,
        _env: &std::collections::HashMap<String, String>,
        _cols: u16,
        _rows: u16,
    ) -> io::Result<Self> {
        // TODO: Implement Unix PTY using nix crate
        // - openpty() to create PTY pair
        // - fork() to create child process
        // - In child: setsid(), set controlling terminal, dup2 stdio, execvp shell
        // - In parent: close slave fd, return master fd
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Unix PTY not yet implemented",
        ))
    }
}

impl PtyBackend for UnixPty {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Not implemented"))
    }

    fn write(&mut self, _data: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Not implemented"))
    }

    fn resize(&self, _cols: u16, _rows: u16) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Not implemented"))
    }

    fn child_pid(&self) -> u32 {
        self.child_pid
    }

    fn is_alive(&self) -> bool {
        false
    }

    fn wait(&mut self) -> io::Result<ExitStatus> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Not implemented"))
    }
}

impl Drop for UnixPty {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.master_fd);
        }
    }
}
