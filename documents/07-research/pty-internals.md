# PTY Internals Research

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. What is a PTY?

**Pseudo-terminal (PTY)** is a pair of virtual character devices providing a bidirectional communication channel. It "emulates" a physical terminal, allowing terminal emulators to communicate with shell/programs just like a real hardware terminal.

```
┌──────────────────┐         ┌──────────────────┐
│  Terminal         │  read/  │  Shell/Program   │
│  Emulator (Wit)   │<──────>│  (bash, zsh...)  │
│                   │  write  │                  │
│  Opens PTY master │         │  Attached to     │
│  /dev/ptmx        │         │  PTY slave       │
│                   │         │  /dev/pts/N      │
└──────────────────┘         └──────────────────┘
        │                            │
        └─── PTY Master ── PTY Slave ┘
              (fd)           (/dev/pts/N)
```

Key concept: Bytes written to master appear as input on slave (and vice versa). Terminal emulator holds the master side, shell/programs see the slave side as their "terminal."

---

## 2. Unix PTY Architecture

### 2.1 Device Files

**Master multiplexer:**
- `/dev/ptmx` - POSIX standard multiplexer device
- Opening `/dev/ptmx` allocates a new PTY pair and returns master fd
- Each open creates a corresponding `/dev/pts/N` slave device

**Slave devices:**
- `/dev/pts/0`, `/dev/pts/1`, ... - pseudo-terminal slave devices
- Mounted via `devpts` filesystem (Linux)
- Shell/program opens slave device as stdin/stdout/stderr

### 2.2 openpty() System Call

```c
#include <pty.h>

int openpty(int *amaster, int *aslave, char *name,
            const struct termios *termp,
            const struct winsize *winp);
```

**Internally, openpty() performs:**

1. `open("/dev/ptmx", O_RDWR)` - open master, receive master fd
2. `grantpt(master_fd)` - set ownership/permissions for slave device
3. `unlockpt(master_fd)` - unlock slave device so it can be opened
4. `ptsname(master_fd)` - get path to slave device (e.g., `/dev/pts/3`)
5. `open(slave_path, O_RDWR)` - open slave device
6. Optionally set `termios` and `winsize` on slave

**Return:** master fd, slave fd, and (optionally) slave device name

### 2.3 forkpty() System Call

```c
#include <pty.h>

pid_t forkpty(int *amaster, char *name,
              const struct termios *termp,
              const struct winsize *winp);
```

**forkpty() combines multiple operations:**

```
Parent process:
  1. openpty() -> master_fd, slave_fd
  2. fork()
  3. Close slave_fd (parent only needs master)
  4. Return child_pid, master_fd

Child process (after fork):
  1. Close master_fd (child only needs slave)
  2. setsid() -> create new session, become session leader
  3. ioctl(slave_fd, TIOCSCTTY, 0) -> set slave as controlling terminal
  4. dup2(slave_fd, STDIN_FILENO)   -> redirect stdin to slave
  5. dup2(slave_fd, STDOUT_FILENO)  -> redirect stdout to slave
  6. dup2(slave_fd, STDERR_FILENO)  -> redirect stderr to slave
  7. Close original slave_fd (already dup2'd)
  8. Return 0
```

> **This is the main flow Wit will use** (via `portable-pty` crate, abstracting platform differences).

### 2.4 Controlling Terminal (CTTY)

**Controlling terminal** is the terminal device associated with a session:

- Each session has at most 1 controlling terminal
- `setsid()` creates a new session WITHOUT a controlling terminal
- `ioctl(fd, TIOCSCTTY, 0)` sets fd as the controlling terminal for the session
- The controlling terminal enables:
  - Signal delivery (Ctrl+C -> SIGINT to foreground process group)
  - Job control (Ctrl+Z -> SIGTSTP)
  - `SIGHUP` when terminal disconnects

### 2.5 Process Groups and Sessions

```
Session (setsid)
├── Session Leader (shell, PID == SID)
├── Foreground Process Group
│   └── Currently running command (e.g., vim)
└── Background Process Groups
    ├── Job 1 (e.g., make &)
    └── Job 2 (e.g., sleep 100 &)
```

**Key concepts:**
- **Session:** Collection of process groups, created by `setsid()`
- **Process Group:** Collection of processes that receive signals together
- **Foreground Process Group:** The "active" group - receives keyboard input and signals from the terminal
- **`tcsetpgrp()`:** Set foreground process group for the terminal

**Important for Wit:**
- When user presses Ctrl+C, the kernel sends SIGINT to the foreground process group (not to Wit)
- Wit does not handle Ctrl+C - the PTY layer and kernel take care of it
- Wit only needs to write raw bytes to the master fd

### 2.6 Signal Handling

| Signal | Trigger | Action | Wit concern |
|--------|---------|--------|-------------|
| **SIGCHLD** | Child process exits/stops | Notification to parent | Wit needs to handle: detect shell exit, cleanup PTY |
| **SIGWINCH** | Terminal window size changes | Notification to foreground group | Wit must send when user resizes: `ioctl(master_fd, TIOCSWINSZ, &ws)` |
| **SIGINT** | Ctrl+C | Terminate foreground group | Handled by kernel/PTY, transparent to Wit |
| **SIGTSTP** | Ctrl+Z | Stop foreground group | Handled by kernel/PTY, transparent to Wit |
| **SIGCONT** | `fg` command | Resume stopped process | Handled by shell |
| **SIGHUP** | Terminal disconnect | Terminate session | Wit triggers when closing tab/window |
| **SIGQUIT** | Ctrl+\\ | Core dump foreground group | Handled by kernel/PTY |

**SIGWINCH flow in Wit:**
```
User resizes window
  -> React detects resize
  -> IPC to Rust backend
  -> Rust calls ioctl(master_fd, TIOCSWINSZ, &winsize)
  -> Kernel sends SIGWINCH to foreground process group
  -> Application (vim, etc.) queries new size and redraws
```

### 2.7 Terminal Line Discipline

Line discipline is the layer between PTY master/slave that performs character processing.

**Cooked mode (canonical mode, default):**
- Line buffering: input is only sent to program when user presses Enter
- Echo: typed characters echo back on terminal
- Special characters: Ctrl+C, Ctrl+Z, Ctrl+D are interpreted
- Line editing: backspace, Ctrl+W (delete word), Ctrl+U (delete line)

**Raw mode:**
- No buffering: each keystroke is sent immediately
- No echo (application handles it)
- No special character processing
- Full-screen applications (vim, htop) use raw mode

**Transition:** Shell is normally in cooked mode. When running vim, vim sets raw mode. When vim exits, shell restores cooked mode.

### 2.8 Key termios Settings

```c
struct termios {
    tcflag_t c_iflag;    // Input flags
    tcflag_t c_oflag;    // Output flags
    tcflag_t c_cflag;    // Control flags
    tcflag_t c_lflag;    // Local flags
    cc_t     c_cc[NCCS]; // Special characters
};
```

#### Input Flags (c_iflag)

| Flag | Purpose | Notes |
|------|---------|-------|
| `IGNBRK` | Ignore break condition | |
| `BRKINT` | Signal interrupt on break | Generates SIGINT |
| `ICRNL` | Map CR to NL on input | Important: converts Enter (CR) -> LF |
| `INLCR` | Map NL to CR on input | Rarely used |
| `IGNCR` | Ignore CR | |
| `IXON` | Enable XON/XOFF flow control | Ctrl+S/Ctrl+Q |
| `IXOFF` | Enable input flow control | |
| `IUTF8` | Input is UTF-8 (Linux) | Affects kernel line editing |

#### Output Flags (c_oflag)

| Flag | Purpose | Notes |
|------|---------|-------|
| `OPOST` | Enable output processing | Master switch for output translation |
| `ONLCR` | Map NL to CR-NL on output | Important: `\n` -> `\r\n` |

#### Local Flags (c_lflag)

| Flag | Purpose | Notes |
|------|---------|-------|
| `ECHO` | Echo input characters | Disabled in password prompts, raw mode |
| `ECHOE` | Echo erase character as BS-SP-BS | Visual backspace |
| `ECHOK` | Echo NL after kill character | |
| `ICANON` | Canonical (line) mode | Core flag: enables line buffering |
| `ISIG` | Enable signals (SIGINT, SIGQUIT, SIGTSTP) | Ctrl+C, Ctrl+\\, Ctrl+Z |
| `IEXTEN` | Enable extended processing | Ctrl+V (literal next) |

#### Special Characters (c_cc)

| Index | Default | Purpose |
|-------|---------|---------|
| `VINTR` | Ctrl+C (0x03) | Interrupt signal |
| `VQUIT` | Ctrl+\\ (0x1C) | Quit signal |
| `VERASE` | Backspace/DEL | Erase character |
| `VKILL` | Ctrl+U (0x15) | Kill line |
| `VEOF` | Ctrl+D (0x04) | End of file |
| `VSUSP` | Ctrl+Z (0x1A) | Suspend signal |
| `VSTART` | Ctrl+Q (0x11) | Resume output (XON) |
| `VSTOP` | Ctrl+S (0x13) | Stop output (XOFF) |

> **Important:** Wit does not need to set termios on the master fd. These settings apply on the slave side. The `portable-pty` crate will handle initial termios setup. But Wit needs to understand these because the behavior affects what bytes Wit receives/sends.

---

## 3. macOS Specifics

### 3.1 Devices

- Master: `/dev/ptmx` (POSIX standard)
- Slaves: `/dev/ttys000`, `/dev/ttys001`, ... (BSD-style naming, not `/dev/pts/N`)
- System call interface is similar to FreeBSD

### 3.2 Differences from Linux

| Aspect | macOS | Linux |
|--------|-------|-------|
| Slave naming | `/dev/ttys*` | `/dev/pts/*` |
| Max PTY | System-limited (usually 999) | Configurable (usually 4096) |
| `posix_openpt()` | Available | Available |
| `openpty()` / `forkpty()` | In `<util.h>` | In `<pty.h>` |
| UTF-8 input flag | No `IUTF8` | Has `IUTF8` |

### 3.3 Sandbox Considerations

- macOS App Sandbox may restrict PTY creation
- Tauri apps typically run un-sandboxed
- If distributing via App Store, entitlements are needed

---

## 4. Linux Specifics

### 4.1 devpts Filesystem

```
mount -t devpts devpts /dev/pts -o gid=5,mode=620
```

- Pseudo-filesystem managing PTY slave devices
- Each mount = separate PTY namespace (important for containers)
- Mount options:
  - `gid=5` - group ownership (tty group)
  - `mode=620` - permissions (owner rw, group w)
  - `ptmxmode=666` - permissions for `/dev/pts/ptmx`
  - `newinstance` - separate PTY namespace

### 4.2 Container Considerations

- Containers mount their own `devpts`
- PTY numbers are per-namespace (container has its own `/dev/pts/0`)
- Wit running inside a container vs. spawning processes in a container - different scenarios

### 4.3 systemd and Login Sessions

- `systemd-logind` manages sessions
- `loginctl` shows active sessions
- Each terminal window creates 1 login session if using login shell
- Relevant for environment variable propagation (e.g., `XDG_SESSION_TYPE`)

---

## 5. Windows ConPTY

### 5.1 History

**Before ConPTY:**
- Windows console subsystem (conhost.exe) was completely different from Unix terminals
- **WinPTY** (2011): Third-party workaround
  - Created hidden console window
  - Scraped screen buffer, translated to VT sequences
  - Used by: VS Code terminal, Git Bash, older Alacritty builds
  - Limitations: screen scraping = slow, imperfect VT translation

**ConPTY (Windows 10 version 1809, Oct 2018):**
- Native pseudo-console API built into Windows
- Proper bidirectional VT sequence support
- Used by: Windows Terminal, modern terminal emulators on Windows
- Part of Windows Console Host (conhost.exe) modernization

### 5.2 CreatePseudoConsole API

```c
#include <windows.h>

HRESULT CreatePseudoConsole(
    COORD      size,         // Initial size (columns, rows)
    HANDLE     hInput,       // Pipe handle for input (terminal -> child)
    HANDLE     hOutput,      // Pipe handle for output (child -> terminal)
    DWORD      dwFlags,      // Flags (0 or PSEUDOCONSOLE_INHERIT_CURSOR)
    HPCON      *phPC         // Output: pseudo-console handle
);
```

### 5.3 ConPTY Setup Flow

```
1. Create pipes:
   CreatePipe(&inputRead, &inputWrite)    // For sending input to child
   CreatePipe(&outputRead, &outputWrite)  // For receiving output from child

2. Create pseudo-console:
   CreatePseudoConsole(size, inputRead, outputWrite, 0, &hPC)

3. Configure startup info:
   STARTUPINFOEX si;
   InitializeProcThreadAttributeList(...)
   UpdateProcThreadAttribute(..., PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, hPC, ...)

4. Create child process:
   CreateProcess(NULL, "cmd.exe", ..., &si, &pi)

5. Communication:
   WriteFile(inputWrite, data, ...)      // Send input to child
   ReadFile(outputRead, buffer, ...)     // Read output from child
```

### 5.4 Key Differences: ConPTY vs Unix PTY

| Aspect | Unix PTY | Windows ConPTY |
|--------|----------|---------------|
| **API** | File descriptors (read/write) | Named pipes (ReadFile/WriteFile) |
| **Device** | `/dev/ptmx` + `/dev/pts/N` | HPCON handle |
| **I/O model** | fd-based, works with select/poll/epoll | HANDLE-based, works with IOCP/WaitForMultipleObjects |
| **VT processing** | Kernel line discipline | ConPTY translates Win32 console API <-> VT sequences |
| **Signals** | SIGWINCH, SIGCHLD, etc. | ResizePseudoConsole, process exit events |
| **Environment** | Inherited via fork/exec | Inherited via CreateProcess, but different defaults |
| **Line endings** | LF (`\n`) | May produce CR+LF (`\r\n`) |
| **Shell** | bash, zsh, fish | cmd.exe, PowerShell, WSL bash |
| **Color support** | Depends on $TERM / terminfo | VT sequences translated by ConPTY |

### 5.5 ResizePseudoConsole

```c
HRESULT ResizePseudoConsole(
    HPCON hPC,        // Pseudo-console handle
    COORD size        // New size (columns, rows)
);
```

- Equivalent of `ioctl(TIOCSWINSZ)` on Unix
- ConPTY internally handles notifying child process
- Child receives `CONSOLE_SCREEN_BUFFER_INFO` change

### 5.6 ClosePseudoConsole

```c
void ClosePseudoConsole(HPCON hPC);
```

**Cleanup sequence:**
1. Close pseudo-console handle
2. Wait for child process to exit
3. Close pipe handles
4. Close process/thread handles

> **Important:** ClosePseudoConsole sends exit signal to child. But child may not exit immediately - need to WaitForSingleObject with timeout.

### 5.7 ConPTY Limitations

1. **VT translation imperfect:** ConPTY translates between Win32 Console API and VT sequences. Some edge cases produce incorrect output
2. **Performance:** Extra translation layer adds latency vs native Unix PTY
3. **Legacy applications:** Old Win32 console apps may not work perfectly
4. **Cursor reporting:** Some cursor position queries don't work correctly
5. **Mouse:** ConPTY mouse support is improving but historically buggy
6. **24-bit color:** Supported since Windows 10 1903, but older versions may have issues

---

## 6. Common Pitfalls

### 6.1 Blocking Reads and Buffer Sizes

**Problem:** `read(master_fd)` blocks if there is no data. If Wit's read thread blocks, the UI thread can also be affected.

**Solutions:**
- Non-blocking I/O: `fcntl(fd, F_SETFL, O_NONBLOCK)` + `poll()/epoll()`
- Async I/O: `tokio::io::AsyncReadExt` (Tokio wraps fd in async)
- Dedicated thread: Read on a separate thread, send via channel

**Buffer size:**
- Typical PTY buffer: 4096 bytes (Linux default)
- Read buffer for Wit: recommend 4096-8192 bytes
- Too small -> many system calls, overhead
- Too large -> latency (waiting to fill buffer before processing)

### 6.2 Partial UTF-8 Sequences

**Problem:** UTF-8 characters can be 1-4 bytes. Read boundary can cut in the middle of a multi-byte character.

```
Read 1: [0x48, 0x65, 0x6C, 0x6C, 0x6F, 0xC3]     <- 0xC3 is first byte of 2-byte char
Read 2: [0xA9, 0x21]                                <- 0xA9 is second byte -> "e!"
```

**Solution:**
```rust
// Keep incomplete bytes between reads
struct ReadBuffer {
    incomplete: Vec<u8>,  // Carry-over bytes from last read
}

impl ReadBuffer {
    fn process(&mut self, new_data: &[u8]) -> String {
        self.incomplete.extend_from_slice(new_data);
        // Find last valid UTF-8 boundary
        let valid_end = find_utf8_boundary(&self.incomplete);
        let valid = String::from_utf8_lossy(&self.incomplete[..valid_end]).to_string();
        self.incomplete = self.incomplete[valid_end..].to_vec();
        valid
    }
}
```

> **Recommendation:** Use the `vte` crate - it handles partial sequences internally.

### 6.3 Shell Startup Time

**Problem:** Shell startup can be slow (especially zsh with plugins):
- `.zshrc` loading
- Plugin managers (oh-my-zsh, zinit, etc.)
- conda/nvm/rbenv initialization
- Custom prompt (starship, powerlevel10k) initialization

**Timing:**
| Shell | Bare | With plugins |
|-------|------|--------------|
| bash | ~50ms | ~200ms |
| zsh | ~100ms | ~500ms-2s |
| fish | ~200ms | ~300ms-1s |
| PowerShell | ~500ms | ~1s-3s |

**Mitigation for Wit:**
- Show loading indicator during shell startup
- Pre-warm PTY in background when application starts
- Don't assume shell is ready after fork - wait for first prompt

### 6.4 Environment Variable Propagation

**Unix:**
- Child inherits parent's environment
- Wit needs to set:
  - `TERM=xterm-256color` (or custom)
  - `COLORTERM=truecolor` (indicate 24-bit color support)
  - `TERM_PROGRAM=wit` (identify terminal emulator)
  - `TERM_PROGRAM_VERSION=x.y.z`
  - `WIT_TERMINAL=1` (custom variable for scripts to detect Wit)
  - `LANG` / `LC_*` - ensure UTF-8 locale

**Windows:**
- CreateProcess inherits environment block
- Need to set:
  - `TERM=xterm-256color` (for WSL and Git Bash)
  - `WT_SESSION` - some tools check this (Windows Terminal sets it)
  - May need to modify `PATH` for proper tool discovery

### 6.5 Login Shell vs Interactive Shell

| Type | How to invoke | Config files loaded | When to use |
|------|---------------|--------------------|----|
| **Login shell** | `bash --login` or `bash -l` | `.bash_profile` -> `.bashrc` | New terminal window/tab |
| **Interactive (non-login)** | `bash` | `.bashrc` only | Subshell, nested shell |
| **Non-interactive** | `bash -c "cmd"` | None (or `$BASH_ENV`) | Script execution |

**Zsh:**
| Type | Config files |
|------|-------------|
| Login | `.zshenv` -> `.zprofile` -> `.zshrc` -> `.zlogin` |
| Interactive | `.zshenv` -> `.zshrc` |
| Non-interactive | `.zshenv` |

> **Recommendation for Wit:** Default to login shell for new tabs (`bash --login`, `zsh --login`). Subshells/splits can use interactive non-login.

### 6.6 Graceful Shutdown

**Unix:**
```
1. Send SIGHUP to child process group
2. Wait briefly (100ms)
3. If still alive, send SIGTERM
4. Wait briefly (100ms)
5. If still alive, send SIGKILL
6. close(master_fd)
7. waitpid(child_pid) - reap zombie
```

**Windows:**
```
1. ClosePseudoConsole(hPC) - signals child
2. WaitForSingleObject(hProcess, timeout)
3. If timeout, TerminateProcess(hProcess)
4. CloseHandle(hProcess)
5. CloseHandle(hThread)
6. Close pipe handles
```

### 6.7 PTY Size Reporting

Wit must set the initial size AND update on resize:

```rust
// Unix
use libc::{ioctl, winsize, TIOCSWINSZ};

let ws = winsize {
    ws_row: rows as u16,
    ws_col: cols as u16,
    ws_xpixel: pixel_width as u16,
    ws_ypixel: pixel_height as u16,
};
unsafe { ioctl(master_fd, TIOCSWINSZ, &ws) };

// Windows
ResizePseudoConsole(hPC, COORD { X: cols, Y: rows });
```

`ws_xpixel` and `ws_ypixel` are important for Sixel graphics - must report accurate pixel size.

---

## 7. Architecture Recommendations for Wit

### 7.1 Thread/Task Model

```
┌─────────────────────────────────────────────────┐
│ Tauri Main Thread (UI)                          │
│                                                 │
│  React <--- IPC events ---> Tauri Commands      │
│                                    │             │
│                                    v             │
│                            ┌──────────────┐     │
│                            │ PTY Manager  │     │
│                            │ (Rust)       │     │
│                            └──────┬───────┘     │
│                                   │              │
│                    ┌──────────────┼───────────┐  │
│                    v              v            v  │
│             ┌──────────┐  ┌──────────┐ ┌────────┐│
│             │ Reader   │  │ Writer   │ │ Resize ││
│             │ Task     │  │ Task     │ │ Handler││
│             │ (async)  │  │ (async)  │ │        ││
│             └────┬─────┘  └──────────┘ └────────┘│
│                  │                                │
│                  v                                │
│           ┌──────────┐                           │
│           │ Parser   │                           │
│           │ (vte)    │                           │
│           └────┬─────┘                           │
│                │                                 │
│                v                                 │
│         ┌──────────────┐                        │
│         │ Terminal     │                        │
│         │ Grid/State   │                        │
│         └──────────────┘                        │
└─────────────────────────────────────────────────┘
```

### 7.2 Crate Usage

| Component | Crate | Notes |
|-----------|-------|-------|
| PTY creation | `portable-pty` | Cross-platform (Unix PTY + ConPTY) |
| VT parsing | `vte` | Alacritty's parser, proven |
| Async I/O | `tokio` | Tauri already uses Tokio |
| IPC | Tauri commands + events | Built-in |

---

## References

1. `pty(7)` man page: `man 7 pty`
2. `termios(3)` man page: `man 3 termios`
3. Stevens & Rago, "Advanced Programming in the UNIX Environment" - Chapter 19 (Pseudo Terminals)
4. Microsoft ConPTY docs: https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session
5. WinPTY: https://github.com/rprichard/winpty
6. portable-pty: https://github.com/nickelpack/portable-pty (originally wez/wezterm)
7. Linux devpts: https://www.kernel.org/doc/html/latest/filesystems/devpts.html
