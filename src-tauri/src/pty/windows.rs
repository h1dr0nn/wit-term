//! Windows ConPTY backend implementation.

use std::io;
use std::path::Path;
use std::process::ExitStatus;
use std::ptr;

use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::System::Console::{
    ClosePseudoConsole, CreatePseudoConsole, ResizePseudoConsole, COORD, HPCON,
};
use windows::Win32::System::Pipes::CreatePipe;
use windows::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, GetExitCodeProcess,
    InitializeProcThreadAttributeList, TerminateProcess, UpdateProcThreadAttribute,
    WaitForSingleObject, CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT,
    LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE,
    STARTUPINFOEXW,
};

use super::PtyBackend;

pub struct ConPty {
    hpc: HPCON,
    pty_input: HANDLE,
    pty_output: HANDLE,
    process_info: PROCESS_INFORMATION,
    _attr_list_buf: Vec<u8>,
}

// SAFETY: ConPTY handles can be safely sent across threads.
// The Windows ConPTY API is designed for multi-threaded use.
unsafe impl Send for ConPty {}

impl ConPty {
    pub fn spawn(
        shell: &Path,
        args: &[String],
        cwd: &Path,
        env: &std::collections::HashMap<String, String>,
        cols: u16,
        rows: u16,
    ) -> io::Result<Self> {
        unsafe { Self::spawn_inner(shell, args, cwd, env, cols, rows) }
    }

    unsafe fn spawn_inner(
        shell: &Path,
        args: &[String],
        cwd: &Path,
        env: &std::collections::HashMap<String, String>,
        cols: u16,
        rows: u16,
    ) -> io::Result<Self> {
        // Create pipes for PTY I/O
        let mut pty_input_read = HANDLE::default();
        let mut pty_input_write = HANDLE::default();
        let mut pty_output_read = HANDLE::default();
        let mut pty_output_write = HANDLE::default();

        let sa = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            bInheritHandle: true.into(),
            lpSecurityDescriptor: ptr::null_mut(),
        };

        CreatePipe(&mut pty_input_read, &mut pty_input_write, Some(&sa), 0)
            .map_err(|e| io::Error::other(format!("CreatePipe input: {e}")))?;

        CreatePipe(&mut pty_output_read, &mut pty_output_write, Some(&sa), 0)
            .map_err(|e| io::Error::other(format!("CreatePipe output: {e}")))?;

        // Create pseudo console
        let size = COORD {
            X: cols as i16,
            Y: rows as i16,
        };

        let hpc = CreatePseudoConsole(size, pty_input_read, pty_output_write, 0)
            .map_err(|e| io::Error::other(format!("CreatePseudoConsole: {e}")))?;

        // Close handles that are now owned by the pseudo console
        let _ = CloseHandle(pty_input_read);
        let _ = CloseHandle(pty_output_write);

        // Initialize proc thread attribute list
        let mut attr_list_size: usize = 0;
        let _ = InitializeProcThreadAttributeList(
            LPPROC_THREAD_ATTRIBUTE_LIST(ptr::null_mut()),
            1,
            0,
            &mut attr_list_size,
        );

        let mut attr_list_buf = vec![0u8; attr_list_size];
        let attr_list =
            LPPROC_THREAD_ATTRIBUTE_LIST(attr_list_buf.as_mut_ptr() as *mut _);

        InitializeProcThreadAttributeList(attr_list, 1, 0, &mut attr_list_size)
            .map_err(|e| io::Error::other(format!("InitializeProcThreadAttributeList: {e}")))?;

        UpdateProcThreadAttribute(
            attr_list,
            0,
            PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
            Some(hpc.0 as *const std::ffi::c_void),
            std::mem::size_of::<HPCON>(),
            None,
            None,
        )
        .map_err(|e| io::Error::other(format!("UpdateProcThreadAttribute: {e}")))?;

        // Build command line
        let cmd_line = if args.is_empty() {
            shell.to_string_lossy().to_string()
        } else {
            format!("{} {}", shell.to_string_lossy(), args.join(" "))
        };
        let mut cmd_wide: Vec<u16> = cmd_line.encode_utf16().chain(std::iter::once(0)).collect();

        // Build environment block - inherit current env and add custom vars
        let env_block = {
            let mut block = String::new();
            for (key, value) in std::env::vars() {
                block.push_str(&format!("{key}={value}\0"));
            }
            for (key, value) in env {
                block.push_str(&format!("{key}={value}\0"));
            }
            block.push('\0');
            block.encode_utf16().collect::<Vec<u16>>()
        };

        let cwd_wide: Vec<u16> = cwd
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut startup_info = STARTUPINFOEXW::default();
        startup_info.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
        startup_info.lpAttributeList = attr_list;

        let mut process_info = PROCESS_INFORMATION::default();

        CreateProcessW(
            None,
            windows::core::PWSTR(cmd_wide.as_mut_ptr()),
            None,
            None,
            false,
            EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,
            Some(env_block.as_ptr() as *const std::ffi::c_void),
            windows::core::PCWSTR(cwd_wide.as_ptr()),
            &startup_info.StartupInfo,
            &mut process_info,
        )
        .map_err(|e| io::Error::other(format!("CreateProcessW: {e}")))?;

        Ok(ConPty {
            hpc,
            pty_input: pty_input_write,
            pty_output: pty_output_read,
            process_info,
            _attr_list_buf: attr_list_buf,
        })
    }
}

impl PtyBackend for ConPty {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut bytes_read: u32 = 0;
        unsafe {
            pty_read_file(self.pty_output, buf, &mut bytes_read)?;
        }
        Ok(bytes_read as usize)
    }

    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let mut bytes_written: u32 = 0;
        unsafe {
            pty_write_file(self.pty_input, data, &mut bytes_written)?;
        }
        Ok(bytes_written as usize)
    }

    fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        let size = COORD {
            X: cols as i16,
            Y: rows as i16,
        };
        unsafe {
            ResizePseudoConsole(self.hpc, size)
                .map_err(|e| io::Error::other(format!("ResizePseudoConsole: {e}")))?;
        }
        Ok(())
    }

    fn child_pid(&self) -> u32 {
        self.process_info.dwProcessId
    }

    fn is_alive(&self) -> bool {
        unsafe { WaitForSingleObject(self.process_info.hProcess, 0) != WAIT_OBJECT_0 }
    }

    fn wait(&mut self) -> io::Result<ExitStatus> {
        unsafe {
            WaitForSingleObject(self.process_info.hProcess, u32::MAX);
            let mut exit_code: u32 = 0;
            GetExitCodeProcess(self.process_info.hProcess, &mut exit_code)
                .map_err(|e| io::Error::other(format!("GetExitCodeProcess: {e}")))?;
            std::process::Command::new("cmd")
                .arg("/c")
                .arg(format!("exit {exit_code}"))
                .status()
        }
    }
}

impl Drop for ConPty {
    fn drop(&mut self) {
        unsafe {
            ClosePseudoConsole(self.hpc);

            if self.is_alive() {
                let _ = TerminateProcess(self.process_info.hProcess, 1);
            }

            let _ = CloseHandle(self.process_info.hProcess);
            let _ = CloseHandle(self.process_info.hThread);
            let _ = CloseHandle(self.pty_input);
            let _ = CloseHandle(self.pty_output);

            if !self._attr_list_buf.is_empty() {
                DeleteProcThreadAttributeList(LPPROC_THREAD_ATTRIBUTE_LIST(
                    self._attr_list_buf.as_mut_ptr() as *mut _,
                ));
            }
        }
    }
}

/// Read from a Windows HANDLE into a buffer.
unsafe fn pty_read_file(
    handle: HANDLE,
    buf: &mut [u8],
    bytes_read: &mut u32,
) -> io::Result<()> {
    windows::Win32::Storage::FileSystem::ReadFile(handle, Some(buf), Some(bytes_read), None)
        .map_err(|e| io::Error::other(format!("ReadFile: {e}")))
}

/// Write to a Windows HANDLE from a buffer.
unsafe fn pty_write_file(
    handle: HANDLE,
    buf: &[u8],
    bytes_written: &mut u32,
) -> io::Result<()> {
    windows::Win32::Storage::FileSystem::WriteFile(handle, Some(buf), Some(bytes_written), None)
        .map_err(|e| io::Error::other(format!("WriteFile: {e}")))
}
