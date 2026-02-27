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
        let vol_arg = format!("{}", self.vlc_volume());
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

    #[allow(dead_code)]
    pub fn is_playing(&self) -> bool {
        self.process.is_some()
    }

    pub fn volume_up(&mut self) {
        if self.volume < 100 {
            self.volume = (self.volume + 5).min(100);
            let _ = self.send_vlc_command(&format!("volume {}\n", self.vlc_volume()));
        }
    }

    pub fn volume_down(&mut self) {
        if self.volume > 0 {
            self.volume = self.volume.saturating_sub(5);
            let _ = self.send_vlc_command(&format!("volume {}\n", self.vlc_volume()));
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

    fn vlc_volume(&self) -> u32 {
        // VLC volume: 0-256 maps from our 0-100
        (self.volume as u32 * 256) / 100
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}
