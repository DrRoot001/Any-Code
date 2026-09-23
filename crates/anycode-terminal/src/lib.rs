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

/// The `PATH` the user's own shell would have, resolved once per process.
///
/// An app opened from Finder or the Dock inherits launchd's `PATH` —
/// `/usr/bin:/bin:/usr/sbin:/sbin` — so an agent's `npm test` fails with "command not
/// found" even though it works in the user's terminal. This asks the user's shell,
/// started as an interactive login shell so it reads the same profile and rc files
/// their terminal does, and caches the answer.
///
/// `None` when that fails or takes too long (a profile that blocks, a shell that
/// rejects the flags); callers then inherit this process's `PATH`. The first call can
/// take seconds — a heavy rc file is sourced — so call it off the async runtime.
pub fn login_shell_path() -> Option<String> {
    static PATH: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    PATH.get_or_init(resolve_login_shell_path).clone()
}

fn resolve_login_shell_path() -> Option<String> {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    if cfg!(windows) {
        return None;
    }
    // rc files print banners and prompts; markers find the value among them.
    const MARK: &str = "__ANYCODE_PATH__";
    let mut child = Command::new(default_shell())
        .args([
            "-l",
            "-i",
            "-c",
            &format!("printf '{MARK}%s{MARK}' \"$PATH\""),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    // Read on another thread: a chatty rc file could otherwise fill the pipe and stall
    // the shell until the deadline.
    let mut stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut out = String::new();
        let _ = stdout.read_to_string(&mut out);
        out
    });

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(50)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }

    let out = reader.join().ok()?;
    let start = out.find(MARK)? + MARK.len();
    let end = start + out[start..].find(MARK)?;
    let path = &out[start..end];
    (!path.is_empty()).then(|| path.to_string())
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

    /// The resolved PATH must be the shell's own, not an empty or garbled capture: it
    /// contains the system directories every login shell has, and is a single line.
    #[test]
    fn login_shell_path_is_the_shells_own() {
        let path = login_shell_path().expect("the login shell should report a PATH");
        assert!(path.split(':').any(|dir| dir == "/usr/bin"), "{path}");
        assert!(!path.contains('\n'), "captured more than PATH: {path:?}");
        // Cached: a second call must not start another shell.
        assert_eq!(login_shell_path().as_deref(), Some(path.as_str()));
    }

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
