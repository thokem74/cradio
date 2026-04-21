use std::io::Write;
use std::process::{Child, ChildStdin, Command, Stdio};

pub struct Player {
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    pub volume: u8,
}

impl Player {
    pub fn new() -> Self {
        Self {
            process: None,
            stdin: None,
            volume: 50,
        }
    }

    pub fn play(&mut self, url: &str) -> Option<String> {
        self.stop();
        let vol_arg = vlc_volume_from_percent(self.volume).to_string();
        let mut cmd = Command::new("cvlc");
        cmd.args([
            "--no-video",
            "--quiet",
            "--intf",
            "rc",
            "--rc-fake-tty",
            "--volume",
            &vol_arg,
            url,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

        match cmd.spawn() {
            Ok(mut child) => {
                self.stdin = child.stdin.take();
                self.process = Some(child);
                None
            }
            Err(e) => {
                self.process = None;
                let msg = if e.kind() == std::io::ErrorKind::NotFound {
                    "cvlc not found. Please install VLC: sudo apt install vlc".to_string()
                } else {
                    format!("Failed to start cvlc: {}", e)
                };
                Some(msg)
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
            self.stdin = None;
        }
    }

    pub fn volume_up(&mut self) {
        if self.volume < 100 {
            self.volume = (self.volume + 5).min(100);
            let _ = self.send_vlc_command(&vlc_volume_command(self.volume));
        }
    }

    pub fn volume_down(&mut self) {
        if self.volume > 0 {
            self.volume = self.volume.saturating_sub(5);
            let _ = self.send_vlc_command(&vlc_volume_command(self.volume));
        }
    }

    fn send_vlc_command(&mut self, cmd: &str) -> std::io::Result<()> {
        if let Some(stdin) = &mut self.stdin {
            stdin.write_all(cmd.as_bytes())?;
            stdin.flush()?;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "cvlc stdin not available",
            ))
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}

fn vlc_volume_from_percent(volume: u8) -> u32 {
    // VLC volume: 0-256 maps from our 0-100
    (volume as u32 * 256) / 100
}

fn vlc_volume_command(volume: u8) -> String {
    format!("volume {}\n", vlc_volume_from_percent(volume))
}

#[cfg(test)]
mod tests {
    use super::{Player, vlc_volume_command, vlc_volume_from_percent};
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn vlc_volume_mapping_matches_expected_bounds() {
        assert_eq!(vlc_volume_from_percent(0), 0);
        assert_eq!(vlc_volume_from_percent(50), 128);
        assert_eq!(vlc_volume_from_percent(100), 256);
    }

    #[test]
    fn vlc_volume_command_formats_rc_input() {
        assert_eq!(vlc_volume_command(25), "volume 64\n");
    }

    #[test]
    fn play_returns_install_hint_when_cvlc_is_not_on_path() {
        let _guard = env_lock().lock().expect("env lock");
        let original_path = std::env::var_os("PATH");

        unsafe {
            std::env::set_var("PATH", "");
        }

        let mut player = Player::new();
        let result = player.play("https://example.com/stream");

        match original_path {
            Some(path) => unsafe { std::env::set_var("PATH", path) },
            None => unsafe { std::env::remove_var("PATH") },
        }

        assert_eq!(
            result,
            Some("cvlc not found. Please install VLC: sudo apt install vlc".to_string())
        );
    }
}
