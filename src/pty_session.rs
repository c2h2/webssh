/// Open a PTY, fork a child process, and return a PtySession.

use nix::pty::{openpty, Winsize};
use nix::unistd::{fork, ForkResult, setsid, dup2, execve, Pid};
use std::ffi::CString;
use std::os::unix::io::RawFd;
use std::os::fd::IntoRawFd;
use nix::fcntl::{fcntl, FcntlArg, OFlag};

pub struct PtySession {
    pub master_fd: RawFd,
    pub pid: Pid,
}

impl PtySession {
    pub fn spawn(argv: &[&str], cols: u16, rows: u16) -> anyhow::Result<Self> {
        let ws = Winsize { ws_col: cols, ws_row: rows, ws_xpixel: 0, ws_ypixel: 0 };
        let result = openpty(Some(&ws), None)?;
        let master_fd: RawFd = result.master.into_raw_fd();
        let slave_fd:  RawFd = result.slave.into_raw_fd();

        match unsafe { fork()? } {
            ForkResult::Parent { child } => {
                unsafe { libc::close(slave_fd) };
                // Set master non-blocking
                let flags = fcntl(master_fd, FcntlArg::F_GETFL)?;
                fcntl(master_fd, FcntlArg::F_SETFL(OFlag::from_bits_truncate(flags) | OFlag::O_NONBLOCK))?;
                Ok(PtySession { master_fd, pid: child })
            }
            ForkResult::Child => {
                setsid()?;
                unsafe {
                    libc::ioctl(slave_fd, libc::TIOCSCTTY as libc::c_ulong, 0);
                }
                dup2(slave_fd, libc::STDIN_FILENO)?;
                dup2(slave_fd, libc::STDOUT_FILENO)?;
                dup2(slave_fd, libc::STDERR_FILENO)?;
                if slave_fd > 2 { unsafe { libc::close(slave_fd) }; }
                unsafe { libc::close(master_fd) };

                let path_val = std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin:/usr/local/bin".into());
                let home_env = std::env::var("HOME").unwrap_or_default();
                let user_env = std::env::var("USER").unwrap_or_default();
                let lang_env = std::env::var("LANG").unwrap_or_else(|_| "en_US.UTF-8".into());
                let env = vec![
                    CString::new("TERM=xterm-256color").unwrap(),
                    CString::new(format!("PATH={path_val}")).unwrap(),
                    CString::new(format!("HOME={home_env}")).unwrap(),
                    CString::new(format!("USER={user_env}")).unwrap(),
                    CString::new(format!("LANG={lang_env}")).unwrap(),
                    // Prevent ssh from using a GUI askpass dialog — force terminal prompt
                    CString::new("SSH_ASKPASS=").unwrap(),
                    CString::new("DISPLAY=").unwrap(),
                ];

                let args: Vec<CString> = argv.iter()
                    .map(|s| CString::new(*s).unwrap())
                    .collect();
                let _ = execve(&args[0], &args, &env);
                unsafe { libc::_exit(1) };
            }
        }
    }

    pub fn resize(&self, cols: u16, rows: u16) {
        let ws = Winsize { ws_col: cols, ws_row: rows, ws_xpixel: 0, ws_ypixel: 0 };
        unsafe { libc::ioctl(self.master_fd, libc::TIOCSWINSZ as libc::c_ulong, &ws); }
    }

    pub fn write_bytes(&self, data: &[u8]) {
        let mut written = 0;
        while written < data.len() {
            match unsafe {
                libc::write(self.master_fd, data[written..].as_ptr() as *const libc::c_void, data.len() - written)
            } {
                n if n > 0 => written += n as usize,
                _ => break,
            }
        }
    }

    /// Returns:
    ///   Ok(Some(n)) — n bytes read
    ///   Ok(None)    — EAGAIN / would-block, no data yet
    ///   Err(())     — EOF or I/O error (process exited)
    pub fn try_read(&self, buf: &mut [u8]) -> Result<Option<usize>, ()> {
        let n = unsafe {
            libc::read(self.master_fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
        };
        if n > 0 {
            Ok(Some(n as usize))
        } else if n == 0 {
            Err(()) // EOF
        } else {
            let e = std::io::Error::last_os_error();
            let errno = e.raw_os_error().unwrap_or(0);
            let would_block = errno == libc::EAGAIN || errno == libc::EWOULDBLOCK;
            if would_block { Ok(None) } else { Err(()) } // EIO or other error = process gone
        }
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        unsafe {
            libc::kill(self.pid.as_raw(), libc::SIGTERM);
            // Reap to avoid zombies
            let mut status = 0i32;
            libc::waitpid(self.pid.as_raw(), &mut status, libc::WNOHANG);
            libc::close(self.master_fd);
        }
    }
}
