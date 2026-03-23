# PTY Handling

> **Status:** draft
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

PTY (Pseudo-Terminal) is the mechanism that allows a terminal emulator to communicate with shell processes. Instead of connecting directly to a hardware serial port (like a real terminal), PTY creates a virtual device pair that simulates a serial connection.

The PTY module is located at `src-tauri/src/pty/` and is the lowest layer in Wit - communicating directly with the OS kernel.

---

## What is a PTY

### Master/Slave Model

```
┌─────────────────┐         ┌──────────────────┐
│  Terminal        │         │  Shell Process    │
│  Emulator (Wit)  │         │  (bash/zsh/pwsh)  │
│                  │         │                   │
│  Reads output  ◄─┤ master ├── Writes stdout    │
│  Writes input  ──┤  pair  ├─► Reads stdin       │
│                  │         │                   │
│                  │  PTY    │  Thinks it's a    │
│                  │ driver  │  real terminal     │
└─────────────────┘         └──────────────────┘
```

**A PTY pair consists of:**
- **Master side (PTY master):** Held by Wit. Reads output from here, writes input to here
- **Slave side (PTY slave):** Used by the shell process. The shell thinks it is talking to a real terminal

**The kernel PTY driver** sits between the two sides, handling:
- Line discipline: echo, line editing, signal generation (Ctrl+C - SIGINT)
- Character processing: CR/LF translation, flow control
- Terminal size tracking

### Why PTY is Needed

Shell and CLI tools rely on `isatty()` to decide behavior:
- **isatty = true:** Interactive mode - colors, prompts, line editing, job control
- **isatty = false:** Pipe mode - no colors, no prompts, raw output

PTY ensures `isatty()` returns true for the shell process, so the shell operates with full interactive features.

---

## Unix PTY

### Relevant APIs

#### `openpty()` - BSD/Linux

```c
#include <pty.h>  // Linux
#include <util.h> // macOS

int openpty(int *amaster, int *aslave, char *name,
            const struct termios *termp,
            const struct winsize *winp);
```

- Creates a PTY pair, returns file descriptors for master and slave
- `termp`: initial terminal attributes (baud rate, echo, etc.)
- `winp`: initial window size (cols x rows)
- More cross-platform than `posix_openpt()` + `grantpt()` + `unlockpt()`

#### `forkpty()` - BSD/Linux

```c
pid_t forkpty(int *amaster, char *name,
              const struct termios *termp,
              const struct winsize *winp);
```

- Combines `openpty()` + `fork()` + `setsid()` + `ioctl(TIOCSCTTY)`
- Child process: slave fd is attached to stdin/stdout/stderr, becomes session leader
- Parent process: receives master fd and child PID

#### POSIX Approach (more portable)

```c
// Step 1: Open master
int master = posix_openpt(O_RDWR | O_NOCTTY);

// Step 2: Grant access & unlock
grantpt(master);
unlockpt(master);

// Step 3: Get slave name
char *slave_name = ptsname(master);

// Step 4: Open slave
int slave = open(slave_name, O_RDWR);

// Step 5: Fork
pid_t pid = fork();
if (pid == 0) {
    // Child: setup as session leader with controlling terminal
    close(master);
    setsid();
    ioctl(slave, TIOCSCTTY, 0);
    dup2(slave, STDIN_FILENO);
    dup2(slave, STDOUT_FILENO);
    dup2(slave, STDERR_FILENO);
    close(slave);
    execvp(shell, args);
}
```

### Rust Implementation

Uses the `nix` crate for POSIX bindings:

```rust
use nix::pty::{openpty, OpenptyResult};
use nix::sys::termios::{self, Termios, SetArg};
use nix::unistd::{fork, ForkResult, close, dup2, setsid, execvp};
use nix::libc;

pub struct UnixPty {
    master_fd: RawFd,
    child_pid: Pid,
}

impl UnixPty {
    pub fn new(config: &PtyConfig) -> io::Result<Self> {
        // 1. Setup initial window size
        let winsize = libc::winsize {
            ws_col: config.cols,
            ws_row: config.rows,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        // 2. Open PTY pair
        let OpenptyResult { master, slave } = openpty(
            Some(&winsize),
            None,  // Use default termios
        )?;

        // 3. Fork child process
        match unsafe { fork() }? {
            ForkResult::Parent { child } => {
                close(slave)?;
                Ok(Self {
                    master_fd: master,
                    child_pid: child,
                })
            }
            ForkResult::Child => {
                close(master)?;

                // Become session leader
                setsid()?;

                // Set controlling terminal
                unsafe {
                    libc::ioctl(slave, libc::TIOCSCTTY, 0);
                }

                // Redirect stdio
                dup2(slave, libc::STDIN_FILENO)?;
                dup2(slave, libc::STDOUT_FILENO)?;
                dup2(slave, libc::STDERR_FILENO)?;

                if slave > 2 {
                    close(slave)?;
                }

                // Setup environment
                for (key, value) in &config.env {
                    std::env::set_var(key, value);
                }
                std::env::set_current_dir(&config.cwd)?;

                // Execute shell
                let c_shell = CString::new(config.shell.to_str().unwrap())?;
                let c_args: Vec<CString> = config.args.iter()
                    .map(|a| CString::new(a.as_str()).unwrap())
                    .collect();
                execvp(&c_shell, &c_args)?;

                unreachable!();
            }
        }
    }
}
```

### Signal Handling

#### SIGCHLD - Child Process Exit

```rust
/// When the shell process exits (crash, user exit, killed)
/// SIGCHLD signal is delivered to the parent (Wit)

// Option 1: waitpid polling (simpler)
fn check_child_status(pid: Pid) -> Option<ExitStatus> {
    match waitpid(pid, Some(WaitPidFlag::WNOHANG)) {
        Ok(WaitStatus::Exited(_, code)) => Some(ExitStatus::Code(code)),
        Ok(WaitStatus::Signaled(_, signal, _)) => Some(ExitStatus::Signal(signal)),
        Ok(WaitStatus::StillAlive) => None,
        _ => None,
    }
}

// Option 2: Signal handler (async-signal-safe)
// Register with signal_hook crate
use signal_hook::iterator::Signals;
let mut signals = Signals::new(&[signal_hook::consts::SIGCHLD])?;
for signal in signals.forever() {
    match signal {
        signal_hook::consts::SIGCHLD => {
            // Reap zombie process
            while let Ok(status) = waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
                // Handle each exited child
            }
        }
        _ => {}
    }
}
```

**Note:** Wit should use a combined approach - detect child exit from the read loop (EOF) and confirm with `waitpid()`.

#### SIGWINCH - Window Size Change

`SIGWINCH` does not need a separate handler in Wit because Wit proactively sends size changes via `ioctl(TIOCSWINSZ)`. The OS automatically sends this signal to the shell process when the master side calls TIOCSWINSZ.

---

## Windows ConPTY

### Overview

Windows does not have Unix PTY. Before Windows 10 1809, terminal emulators had to use hacks (inject DLL, scrape console buffer). The ConPTY (Console Pseudo Terminal) API was released in 2018 and provides a PTY-like interface.

### API Overview

```
┌──────────────┐     ┌──────────┐     ┌──────────────┐
│  Wit         │     │  ConPTY  │     │  Shell       │
│  (Terminal)  │     │  (OS)    │     │  (pwsh/cmd)  │
│              │     │          │     │              │
│  ReadFile ◄──┤─pipe┤──────────┤─────┤── stdout     │
│  WriteFile ──┤─pipe┤──────────┤─────┤── stdin      │
│              │     │          │     │              │
└──────────────┘     └──────────┘     └──────────────┘
```

### `CreatePseudoConsole`

```c
HRESULT CreatePseudoConsole(
    COORD size,                     // Initial size {cols, rows}
    HANDLE hInput,                  // Read end of input pipe (shell reads from here)
    HANDLE hOutput,                 // Write end of output pipe (shell writes to here)
    DWORD dwFlags,                  // 0 or PSEUDOCONSOLE_INHERIT_CURSOR
    HPCON *phPC                     // Output: pseudo console handle
);
```

### Rust Implementation

```rust
use windows::Win32::System::Console::*;
use windows::Win32::System::Threading::*;
use windows::Win32::System::Pipes::*;
use windows::Win32::Security::*;

pub struct ConPty {
    console: HPCON,
    input_write: HANDLE,   // Wit writes user input here
    output_read: HANDLE,   // Wit reads shell output here
    process: HANDLE,
    thread: HANDLE,
}

impl ConPty {
    pub fn new(config: &PtyConfig) -> io::Result<Self> {
        // 1. Create pipes
        let mut input_read = HANDLE::default();
        let mut input_write = HANDLE::default();
        let mut output_read = HANDLE::default();
        let mut output_write = HANDLE::default();

        unsafe {
            CreatePipe(&mut input_read, &mut input_write, None, 0)?;
            CreatePipe(&mut output_read, &mut output_write, None, 0)?;
        }

        // 2. Create pseudo console
        let size = COORD {
            X: config.cols as i16,
            Y: config.rows as i16,
        };
        let mut console = HPCON::default();
        unsafe {
            CreatePseudoConsole(size, input_read, output_write, 0, &mut console)?;
        }

        // 3. Setup startup info with pseudo console
        let mut startup_info_ex = STARTUPINFOEXW::default();
        startup_info_ex.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;

        // Initialize proc thread attribute list
        let mut attr_list_size: usize = 0;
        unsafe {
            InitializeProcThreadAttributeList(
                LPPROC_THREAD_ATTRIBUTE_LIST::default(),
                1, 0, &mut attr_list_size,
            );
        }
        let mut attr_list_buf = vec![0u8; attr_list_size];
        let attr_list = LPPROC_THREAD_ATTRIBUTE_LIST(attr_list_buf.as_mut_ptr() as _);
        unsafe {
            InitializeProcThreadAttributeList(attr_list, 1, 0, &mut attr_list_size)?;
            UpdateProcThreadAttribute(
                attr_list,
                0,
                PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
                Some(console.0 as *const _ as *const std::ffi::c_void),
                std::mem::size_of::<HPCON>(),
                None, None,
            )?;
        }
        startup_info_ex.lpAttributeList = attr_list;

        // 4. Create process
        let mut process_info = PROCESS_INFORMATION::default();
        let command_line = format!(
            "\"{}\" {}",
            config.shell.display(),
            config.args.join(" ")
        );
        unsafe {
            CreateProcessW(
                None,
                &mut command_line.encode_utf16().collect::<Vec<_>>(),
                None, None,
                false,
                EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,
                None,
                config.cwd.to_str().map(|s| s.encode_utf16().collect::<Vec<_>>()),
                &startup_info_ex.StartupInfo,
                &mut process_info,
            )?;
        }

        // 5. Close pipe ends that belong to child
        unsafe {
            CloseHandle(input_read)?;
            CloseHandle(output_write)?;
        }

        Ok(Self {
            console,
            input_write,
            output_read,
            process: process_info.hProcess,
            thread: process_info.hThread,
        })
    }
}
```

### ConPTY Limitations vs Unix PTY

| Feature | Unix PTY | Windows ConPTY |
|---|---|---|
| API maturity | Decades, very stable | Since 2018, still has quirks |
| Signal handling | SIGCHLD, SIGWINCH | WaitForSingleObject, ResizePseudoConsole |
| Raw mode | `tcsetattr(TCSANOW)` | ConPTY handles this itself |
| Output processing | Configurable via termios | ConPTY processes ANSI internally then re-emits |
| Performance | Direct fd I/O | Pipe-based, additional overhead |
| Terminal size | `ioctl(TIOCSWINSZ)` | `ResizePseudoConsole()` |
| Process groups | Full support | Limited |
| Job control | Full (fg, bg, Ctrl+Z) | Limited |
| UTF-8 handling | Via locale/termios | ConPTY converts automatically |
| Double-width chars | Terminal handles | ConPTY may miscount columns |

**Known ConPTY Issues:**
- ConPTY sometimes sends spurious redraws (entire screen content)
- Resize can cause visual artifacts
- Some VT sequences are intercepted and re-interpreted incorrectly by ConPTY
- Performance overhead due to pipe + console rendering pipeline

---

## PTY Lifecycle

### Sequence Diagram

```
Wit                    OS/Kernel              Shell
 │                        │                     │
 │  1. Create PTY pair    │                     │
 ├───────────────────────►│                     │
 │  ◄── master_fd, slave  │                     │
 │                        │                     │
 │  2. Configure termios  │                     │
 ├───────────────────────►│                     │
 │                        │                     │
 │  3. Fork + exec shell  │                     │
 ├───────────────────────►│────────────────────►│
 │                        │   slave → stdio     │
 │                        │                     │
 │  4. Set TERM env       │                     │
 │  5. Set initial size   │                     │
 │                        │                     │
 │  ◄─── Shell prompt ────┤◄────────────────────┤
 │  6. Read from master   │                     │
 │                        │                     │
 │  7. User types ────────┤────────────────────►│
 │     Write to master    │                     │
 │                        │                     │
 │  ◄─── Shell output ────┤◄────────────────────┤
 │  8. Read from master   │                     │
 │                        │                     │
 │  9. Window resize      │                     │
 ├── TIOCSWINSZ ─────────►│── SIGWINCH ────────►│
 │                        │                     │
 │  ◄─── EOF (read = 0) ──┤◄── Shell exit ──────┤
 │  10. Cleanup           │                     │
 ├── waitpid ────────────►│                     │
 ├── close(master_fd) ───►│                     │
 │                        │                     │
```

### Phase Details

#### Phase 1: Create

```rust
fn create_pty(config: &PtyConfig) -> Result<PtyHandle> {
    #[cfg(unix)]
    {
        let pty = UnixPty::new(config)?;
        Ok(PtyHandle::Unix(pty))
    }

    #[cfg(windows)]
    {
        let pty = ConPty::new(config)?;
        Ok(PtyHandle::Windows(pty))
    }
}
```

#### Phase 2: Configure

Unix terminal attributes (termios) configuration:

```rust
fn configure_termios(fd: RawFd) -> Result<()> {
    let mut termios = termios::tcgetattr(fd)?;

    // Raw mode - disable kernel line editing
    termios::cfmakeraw(&mut termios);

    // But keep some settings:
    // - UTF-8 support (CS8)
    termios.control_flags |= ControlFlags::CS8;

    // Apply immediately
    termios::tcsetattr(fd, SetArg::TCSANOW, &termios)?;

    Ok(())
}
```

**Note:** Wit needs raw mode because the terminal emulator handles echo, line editing, and signal generation itself. If the kernel handles these, double-processing would occur.

#### Phase 3: Spawn Shell

See details in the "Shell Spawning" section below.

#### Phase 4: I/O Loop

```rust
/// Main I/O read loop - runs on a dedicated thread
fn pty_io_loop(
    pty: &mut dyn PtyBackend,
    output_tx: Sender<Vec<u8>>,
    shutdown_rx: Receiver<()>,
) -> Result<()> {
    let mut buf = [0u8; 8192];

    loop {
        // Check shutdown signal (non-blocking)
        if shutdown_rx.try_recv().is_ok() {
            break;
        }

        match pty.read(&mut buf) {
            Ok(0) => {
                // EOF - shell exited
                break;
            }
            Ok(n) => {
                // Send bytes to parser/terminal
                let data = buf[..n].to_vec();
                if output_tx.send(data).is_err() {
                    break; // Receiver dropped
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
                // EINTR - retry
                continue;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // Non-blocking mode, no data available
                // Sleep briefly to avoid busy-wait
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }
            Err(e) => {
                // Real error - PTY is broken
                return Err(e.into());
            }
        }
    }

    Ok(())
}
```

#### Phase 5: Resize

```rust
impl UnixPty {
    pub fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        let winsize = libc::winsize {
            ws_col: cols,
            ws_row: rows,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        unsafe {
            if libc::ioctl(self.master_fd, libc::TIOCSWINSZ, &winsize) == -1 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }
}

impl ConPty {
    pub fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        let size = COORD {
            X: cols as i16,
            Y: rows as i16,
        };
        unsafe {
            ResizePseudoConsole(self.console, size)?;
        }
        Ok(())
    }
}
```

#### Phase 6: Cleanup

```rust
impl Drop for UnixPty {
    fn drop(&mut self) {
        // 1. Send SIGHUP to child process group
        //    (terminal hangup - conventional signal when terminal closes)
        unsafe {
            libc::kill(-self.child_pid.as_raw(), libc::SIGHUP);
        }

        // 2. Give child a moment to exit gracefully
        std::thread::sleep(Duration::from_millis(100));

        // 3. Force kill if still alive
        if let Ok(WaitStatus::StillAlive) = waitpid(
            self.child_pid,
            Some(WaitPidFlag::WNOHANG)
        ) {
            unsafe {
                libc::kill(-self.child_pid.as_raw(), libc::SIGKILL);
            }
        }

        // 4. Reap zombie
        let _ = waitpid(self.child_pid, None);

        // 5. Close master fd
        let _ = close(self.master_fd);
    }
}

impl Drop for ConPty {
    fn drop(&mut self) {
        unsafe {
            // Close console first - this will signal child to exit
            ClosePseudoConsole(self.console);

            // Wait briefly for process to exit
            WaitForSingleObject(self.process, 1000);

            // Terminate if still running
            TerminateProcess(self.process, 1);

            // Close handles
            CloseHandle(self.process);
            CloseHandle(self.thread);
            CloseHandle(self.input_write);
            CloseHandle(self.output_read);
        }
    }
}
```

---

## Shell Spawning

### Default Shell Detection

#### Unix

```rust
fn detect_default_shell_unix() -> PathBuf {
    // Priority order:
    // 1. $SHELL environment variable
    if let Ok(shell) = std::env::var("SHELL") {
        return PathBuf::from(shell);
    }

    // 2. User's login shell from passwd database
    if let Some(shell) = get_passwd_shell() {
        return PathBuf::from(shell);
    }

    // 3. Fallback
    PathBuf::from("/bin/sh")
}

fn get_passwd_shell() -> Option<String> {
    let uid = unsafe { libc::getuid() };
    let passwd = unsafe { libc::getpwuid(uid) };
    if !passwd.is_null() {
        let shell = unsafe { CStr::from_ptr((*passwd).pw_shell) };
        return Some(shell.to_string_lossy().to_string());
    }
    None
}
```

#### Windows

```rust
fn detect_default_shell_windows() -> PathBuf {
    // Priority order:
    // 1. PowerShell 7+ (pwsh.exe) - modern, cross-platform
    if let Ok(output) = Command::new("where").arg("pwsh.exe").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout);
            return PathBuf::from(path.trim());
        }
    }

    // 2. Windows PowerShell (powershell.exe) - always available
    if let Ok(system_root) = std::env::var("SystemRoot") {
        let ps_path = PathBuf::from(&system_root)
            .join("System32")
            .join("WindowsPowerShell")
            .join("v1.0")
            .join("powershell.exe");
        if ps_path.exists() {
            return ps_path;
        }
    }

    // 3. cmd.exe - last resort fallback
    if let Ok(comspec) = std::env::var("COMSPEC") {
        return PathBuf::from(comspec);
    }

    PathBuf::from("cmd.exe")
}
```

### Environment Setup

Variables to set for the shell process:

```rust
fn build_shell_environment(config: &PtyConfig) -> HashMap<String, String> {
    let mut env: HashMap<String, String> = std::env::vars().collect();

    // Terminal identification
    env.insert("TERM".into(), "xterm-256color".into());
    env.insert("COLORTERM".into(), "truecolor".into());

    // Wit-specific
    env.insert("WIT_TERM".into(), "1".into());
    env.insert("WIT_VERSION".into(), env!("CARGO_PKG_VERSION").into());

    // Locale (ensure UTF-8)
    if !env.contains_key("LANG") {
        env.insert("LANG".into(), "en_US.UTF-8".into());
    }

    // Override with user-specified env vars
    env.extend(config.env.clone());

    env
}
```

### Shell Arguments

| Shell | Login | Interactive | Recommended |
|---|---|---|---|
| bash | `--login` | `-i` | `bash --login` |
| zsh | `--login` | `-i` | `zsh --login` |
| fish | `--login` | `-i` | `fish --login` |
| pwsh | `-Login` | `-NoExit` | `pwsh -Login -NoExit` |
| cmd | - | - | `cmd.exe` |

Wit spawns the shell in login mode by default to load the user profile (.bashrc, .zshrc, etc.).

---

## I/O Patterns

### Buffer Sizes

| Scenario | Buffer Size | Reason |
|---|---|---|
| PTY read buffer | 8,192 bytes | Balance between syscall overhead and latency |
| PTY write buffer | 4,096 bytes | User input has less data than output |
| Pipe buffer (OS) | ~64 KB (Linux), ~4 KB (macOS) | OS kernel pipe buffer |

### Blocking vs Non-blocking I/O

**Read side (PTY output):**

```rust
// Option A: Blocking read on a dedicated thread (recommended)
// Simple, reliable, low CPU usage
fn blocking_read_loop(master_fd: RawFd) {
    let mut buf = [0u8; 8192];
    loop {
        // Blocks until data available or EOF
        let n = unsafe { libc::read(master_fd, buf.as_mut_ptr() as _, buf.len()) };
        if n <= 0 { break; }
        // Process buf[..n]
    }
}

// Option B: Non-blocking with poll/select
// More control, can combine with shutdown signal
fn nonblocking_read_loop(master_fd: RawFd, shutdown_fd: RawFd) {
    use nix::poll::{poll, PollFd, PollFlags};

    let mut fds = [
        PollFd::new(master_fd, PollFlags::POLLIN),
        PollFd::new(shutdown_fd, PollFlags::POLLIN),
    ];

    loop {
        match poll(&mut fds, 100 /* timeout ms */) {
            Ok(0) => continue, // timeout
            Ok(_) => {
                if fds[1].revents().unwrap().contains(PollFlags::POLLIN) {
                    break; // shutdown signal
                }
                if fds[0].revents().unwrap().contains(PollFlags::POLLIN) {
                    // Read available data
                }
                if fds[0].revents().unwrap().contains(PollFlags::POLLHUP) {
                    break; // PTY closed
                }
            }
            Err(nix::errno::Errno::EINTR) => continue,
            Err(_) => break,
        }
    }
}
```

**Wit recommendation:** Use blocking read on a dedicated thread + shutdown via closing the master fd or an eventfd/pipe signal.

**Write side (user input):**

Write is usually fast with little data. Blocking write is sufficient. However, the following must be handled:
- `EAGAIN`/`EWOULDBLOCK` if the pipe buffer is full (rarely happens with user input)
- Partial write - retry remaining bytes

### Handling Partial Reads

PTY read may return any number of bytes, including in the middle of a multi-byte UTF-8 sequence or in the middle of an ANSI escape sequence:

```
Read 1: b"\x1b[38;2;25"   <- Incomplete CSI sequence
Read 2: b"5;128;0mHello"  <- Continuation + text
```

**Handling:**
1. The ANSI parser must maintain state between reads
2. The UTF-8 decoder must buffer incomplete multi-byte sequences
3. Never assume a read boundary = logical boundary

---

## Error Handling

### Shell Process Crash

```rust
enum PtyEvent {
    /// Shell exited normally
    ShellExited { exit_code: i32 },

    /// Shell killed by signal
    ShellKilled { signal: i32 },

    /// PTY I/O error
    IoError { error: io::Error },
}

fn handle_shell_exit(session: &mut Session, event: PtyEvent) {
    match event {
        PtyEvent::ShellExited { exit_code } => {
            // Show exit status in terminal
            session.write_to_grid(&format!(
                "\r\n[Process exited with code {}]\r\n", exit_code
            ));
            // Optionally auto-close session after delay
        }
        PtyEvent::ShellKilled { signal } => {
            session.write_to_grid(&format!(
                "\r\n[Process killed by signal {}]\r\n", signal
            ));
        }
        PtyEvent::IoError { error } => {
            session.write_to_grid(&format!(
                "\r\n[PTY error: {}]\r\n", error
            ));
        }
    }

    // Cleanup PTY resources
    session.mark_dead();
    // Notify frontend
    emit_event("session_ended", session.id);
}
```

### PTY Errors

| Error | Cause | Handling |
|---|---|---|
| `ENOENT` on exec | Shell path does not exist | Notify user, suggest alternatives |
| `EACCES` on exec | No execute permission | Notify user |
| `EIO` on read | PTY slave closed (shell exit) | Detect EOF, cleanup session |
| `EAGAIN` on read | Non-blocking, no data | Retry (poll/select) |
| `EINTR` on read/write | Signal interrupted | Retry immediately |
| `EPERM` on openpty | No permission to create PTY | Fatal - cannot create session |
| `ENOMEM` | Out of memory | Fatal - log and notify |

### Cleanup on Abnormal Exit

When Wit crashes or is force-killed:
- OS automatically closes file descriptors - master fd closed - shell receives SIGHUP - shell exits
- Zombie processes: OS automatically reaps when parent exits
- ConPTY: Windows automatically cleans up when process handle is closed

When Wit exits gracefully (user closes app):
- Iterate all sessions - send SIGHUP - waitpid - close fds
- Timeout: force kill after 2 seconds

---

## Cross-Platform Abstraction

### Trait Design

```rust
/// Core PTY operations - platform-agnostic
pub trait PtyBackend: Send {
    /// Read bytes from PTY (shell output)
    /// Returns 0 on EOF (shell exited)
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;

    /// Write bytes to PTY (user input)
    fn write(&mut self, data: &[u8]) -> io::Result<usize>;

    /// Resize the terminal
    fn resize(&self, cols: u16, rows: u16) -> io::Result<()>;

    /// Get the PID/process ID of the child
    fn child_pid(&self) -> u32;

    /// Check if child is still alive (non-blocking)
    fn is_alive(&self) -> bool;

    /// Wait for child to exit (blocking)
    fn wait(&mut self) -> io::Result<ExitStatus>;

    /// Get the name of the running process
    /// (may read /proc on Linux, ProcessInfo on macOS/Windows)
    fn foreground_process_name(&self) -> Option<String>;
}

/// Factory
pub fn create_pty(config: &PtyConfig) -> io::Result<Box<dyn PtyBackend>> {
    #[cfg(unix)]
    { Ok(Box::new(unix::UnixPty::new(config)?)) }

    #[cfg(windows)]
    { Ok(Box::new(windows::ConPty::new(config)?)) }
}
```

### Platform-Specific Modules

```
src-tauri/src/pty/
├── mod.rs          # PtyBackend trait, PtyConfig, create_pty()
├── unix.rs         # UnixPty implementation (cfg(unix))
├── windows.rs      # ConPty implementation (cfg(windows))
└── common.rs       # Shared utilities (shell detection, env setup)
```

### Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Test basic PTY creation and I/O
    #[test]
    fn test_pty_echo() {
        let config = PtyConfig {
            shell: PathBuf::from(if cfg!(windows) { "cmd.exe" } else { "/bin/sh" }),
            args: vec![],
            cwd: std::env::current_dir().unwrap(),
            env: HashMap::new(),
            cols: 80,
            rows: 24,
        };

        let mut pty = create_pty(&config).unwrap();

        // Write a command
        pty.write(b"echo hello\n").unwrap();

        // Read output (with timeout)
        let mut buf = [0u8; 1024];
        let mut output = String::new();
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            match pty.read(&mut buf) {
                Ok(n) if n > 0 => {
                    output.push_str(&String::from_utf8_lossy(&buf[..n]));
                    if output.contains("hello") {
                        break;
                    }
                }
                _ => std::thread::sleep(Duration::from_millis(10)),
            }
        }

        assert!(output.contains("hello"));
    }

    /// Test resize
    #[test]
    fn test_pty_resize() {
        let config = PtyConfig { cols: 80, rows: 24, .. };
        let pty = create_pty(&config).unwrap();

        // Should not error
        pty.resize(120, 40).unwrap();
        pty.resize(40, 10).unwrap();
    }

    /// Test child exit detection
    #[test]
    fn test_pty_child_exit() {
        let config = PtyConfig {
            shell: PathBuf::from(if cfg!(windows) { "cmd.exe" } else { "/bin/sh" }),
            args: vec!["-c".into(), "exit 42".into()],
            ..
        };

        let mut pty = create_pty(&config).unwrap();
        let status = pty.wait().unwrap();
        assert_eq!(status.code(), Some(42));
    }
}
```

---

## References

- [The TTY Demystified](https://www.linusakesson.net/programming/tty/) - Linus Akesson
- [termios(3)](https://man7.org/linux/man-pages/man3/termios.3.html) - POSIX terminal interface
- [pty(7)](https://man7.org/linux/man-pages/man7/pty.7.html) - Linux PTY overview
- [ConPTY API](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session) - Microsoft docs
- [Alacritty tty module](https://github.com/alacritty/alacritty/tree/master/alacritty_terminal/src/tty) - Reference implementation
- [nix crate](https://docs.rs/nix) - Rust POSIX bindings
