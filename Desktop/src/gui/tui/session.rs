use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};

use super::env;

pub const DEFAULT_ROWS: u16 = 40;
pub const DEFAULT_COLS: u16 = 80;

pub struct TuiSession {
    pub writer: Box<dyn Write + Send>,
    pub parser: Arc<Mutex<vt100::Parser>>,
    pub queue: Arc<Mutex<Vec<Vec<u8>>>>,
    pub done: Arc<AtomicBool>,
    #[allow(dead_code)]
    pub rows: u16,
    #[allow(dead_code)]
    pub cols: u16,
    _master: Box<dyn portable_pty::MasterPty + Send>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
}

fn shell_command_builder(command: &str, cwd: &Path) -> CommandBuilder {
    #[cfg(unix)]
    {
        // Non-login `*-c`: `-lc` runs profiles that may block, prompt, or alter the PTY — breaks
        // embedded one-shot commands and TUIs. PATH comes from `apply_interactive_shell_env`.
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let mut cmd = CommandBuilder::new(shell);
        cmd.args(["-c", command]);
        cmd.cwd(cwd);
        env::apply_interactive_shell_env(&mut cmd);
        cmd
    }
    #[cfg(not(unix))]
    {
        let mut cmd = CommandBuilder::new("sh");
        cmd.cwd(cwd);
        cmd.args(["-c", command]);
        cmd
    }
}

impl TuiSession {
    pub fn spawn(
        command: &str,
        rows: u16,
        cols: u16,
        cwd: &Path,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        let cmd = shell_command_builder(command, cwd);
        let child = pair.slave.spawn_command(cmd)?;
        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 1000)));
        let queue: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
        let done = Arc::new(AtomicBool::new(false));

        {
            let q = Arc::clone(&queue);
            let d = Arc::clone(&done);
            std::thread::spawn(move || {
                let mut reader = reader;
                let mut buf = [0u8; 4096];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) | Err(_) => {
                            d.store(true, Ordering::SeqCst);
                            break;
                        }
                        Ok(n) => {
                            if let Ok(mut locked) = q.lock() {
                                locked.push(buf[..n].to_vec());
                            }
                        }
                    }
                }
            });
        }

        Ok(TuiSession {
            writer,
            parser,
            queue,
            done,
            rows,
            cols,
            _master: pair.master,
            _child: child,
        })
    }

    pub fn write_input(&mut self, bytes: &[u8]) {
        let _ = self.writer.write_all(bytes);
        let _ = self.writer.flush();
    }

    pub fn resize(&self, rows: u16, cols: u16) {
        let _ = self._master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
        if let Ok(mut p) = self.parser.lock() {
            p.set_size(rows, cols);
        }
    }

    /// Current working directory of the foreground process group on this PTY (e.g. interactive shell).
    #[cfg(unix)]
    pub fn foreground_cwd(&self) -> Option<String> {
        let pid = self._master.process_group_leader()? as u32;
        if pid == 0 {
            return None;
        }
        super::cwd::cwd_for_pid(pid)
    }

    #[cfg(not(unix))]
    pub fn foreground_cwd(&self) -> Option<String> {
        None
    }
}
