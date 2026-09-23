//! Native PTY sessions (PRD §82). This crate owns spawning and I/O only; the Tauri
//! layer owns the session table and turns bytes into frontend events, because "how a
//! session's lifecycle is tracked" is app-shell policy, not terminal mechanics.

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum TerminalError {
    #[error(transparent)]
    Pty(#[from] anyhow::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// The user's login shell, falling back to something that exists on every platform.
fn default_shell() -> String {
    if cfg!(windows) {
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }
}

pub struct PtySession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
}

impl PtySession {
    /// Spawns the user's shell in `cwd`, returning the session handle plus a reader
    /// the caller owns directly — reading blocks, so it belongs on its own thread.
    pub fn spawn(
        cwd: &Path,
        cols: u16,
        rows: u16,
    ) -> Result<(Self, Box<dyn Read + Send>), TerminalError> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(default_shell());
        cmd.cwd(cwd);

        // A GUI process inherits no TERM, which leaves the shell with no terminal
        // capabilities: no line editor, no colour, and `clear`/`less`/`vim` failing
        // outright. The frontend is xterm.js, so name what it actually emulates.
        cmd.env("TERM", "xterm-256color");

        // A GUI process also inherits the launcher's PATH, not the user's — an app
        // opened from Finder sees only /usr/bin:/bin:/usr/sbin:/sbin, so nothing the
        // user installed is on it. A login shell reads the profile that sets PATH,
        // which is what their own terminal emulator does too.
        if !cfg!(windows) {
            cmd.arg("-l");
        }

        let child = pair.slave.spawn_command(cmd)?;
        drop(pair.slave);

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        Ok((
            Self {
                master: pair.master,
                writer,
                child,
            },
            reader,
        ))
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), TerminalError> {
        self.writer.write_all(data)?;
        Ok(())
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), TerminalError> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    pub fn kill(&mut self) -> Result<(), TerminalError> {
        self.child.kill()?;
        Ok(())
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    /// The shell must come up able to drive an xterm-256color emulator. Without TERM a
    /// GUI-spawned shell has no line editor and no colour, which looks to the user like
    /// a dead terminal — so assert the value the shell itself reports, not our input.
    #[test]
    fn spawned_shell_reports_an_xterm_term() {
        let (mut session, mut reader) =
            PtySession::spawn(Path::new("/"), 80, 24).expect("shell should spawn");

        // A login shell can take seconds to finish sourcing the user's profile, and it
        // would swallow input typed before then, so send on a delay and read patiently.
        let writer = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(2));
            session.write(b"printf 'TERM<%s>\\n' \"$TERM\"\n").unwrap();
            // Held open: dropping the session closes the PTY and ends the read below.
            std::thread::sleep(Duration::from_secs(13));
        });

        let deadline = Instant::now() + Duration::from_secs(15);
        let mut seen = String::new();
        let mut buf = [0u8; 4096];
        while Instant::now() < deadline {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => seen.push_str(&String::from_utf8_lossy(&buf[..n])),
                Err(_) => break,
            }
            if seen.contains("TERM<xterm-256color>") {
                return;
            }
        }
        writer.join().ok();
        panic!("shell never reported TERM=xterm-256color; saw: {seen:?}");
    }
}
