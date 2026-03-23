//! Cross-platform PTY abstraction layer.

use std::io;
use std::process::ExitStatus;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

/// Cross-platform PTY backend trait.
///
/// Read and write operate on separate underlying handles and are safe
/// to call concurrently (read from one thread, write from another).
pub trait PtyBackend: Send + Sync {
    fn read(&self, buf: &mut [u8]) -> io::Result<usize>;
    fn write(&self, data: &[u8]) -> io::Result<usize>;
    fn resize(&self, cols: u16, rows: u16) -> io::Result<()>;
    fn child_pid(&self) -> u32;
    fn is_alive(&self) -> bool;
    fn wait(&self) -> io::Result<ExitStatus>;
}

/// Configuration for spawning a PTY session.
#[derive(Debug, Clone)]
pub struct PtyConfig {
    pub shell: std::path::PathBuf,
    pub args: Vec<String>,
    pub cwd: std::path::PathBuf,
    pub env: std::collections::HashMap<String, String>,
    pub cols: u16,
    pub rows: u16,
}

impl Default for PtyConfig {
    fn default() -> Self {
        let mut env = std::collections::HashMap::new();
        env.insert("TERM".into(), "xterm-256color".into());
        env.insert("COLORTERM".into(), "truecolor".into());

        Self {
            shell: detect_default_shell(),
            args: vec![],
            cwd: std::env::current_dir().unwrap_or_else(|_| {
                #[cfg(unix)]
                {
                    std::path::PathBuf::from("/")
                }
                #[cfg(windows)]
                {
                    dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("C:\\"))
                }
            }),
            env,
            cols: 80,
            rows: 24,
        }
    }
}

/// Spawn a PTY with the given configuration.
pub fn spawn_pty(config: &PtyConfig) -> io::Result<Box<dyn PtyBackend>> {
    #[cfg(windows)]
    {
        let pty = windows::ConPty::spawn(
            &config.shell,
            &config.args,
            &config.cwd,
            &config.env,
            config.cols,
            config.rows,
        )?;
        Ok(Box::new(pty))
    }

    #[cfg(unix)]
    {
        let pty = unix::UnixPty::spawn(
            &config.shell,
            &config.args,
            &config.cwd,
            &config.env,
            config.cols,
            config.rows,
        )?;
        Ok(Box::new(pty))
    }
}

/// Detect the default shell for the current platform.
fn detect_default_shell() -> std::path::PathBuf {
    #[cfg(unix)]
    {
        std::env::var("SHELL")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("/bin/sh"))
    }

    #[cfg(windows)]
    {
        for shell in &["pwsh.exe", "powershell.exe", "cmd.exe"] {
            if which_exists(shell) {
                return std::path::PathBuf::from(shell);
            }
        }
        std::path::PathBuf::from("cmd.exe")
    }
}

#[cfg(windows)]
fn which_exists(name: &str) -> bool {
    std::process::Command::new("where")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
