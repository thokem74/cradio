#[cfg(target_os = "linux")]
mod imp {
    use std::io::Write;
    use std::process::{Child, ChildStdin, Command, Stdio};

    pub struct Player {
        process: Option<Child>,
        stdin: Option<ChildStdin>,
        volume: u8,
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

            let mut cmd = Command::new("cvlc");
            cmd.args([
                "--no-video",
                "--quiet",
                "--intf",
                "rc",
                "--rc-fake-tty",
                "--volume",
                &self.vlc_volume().to_string(),
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
                Err(err) => {
                    self.process = None;
                    self.stdin = None;
                    Some(match err.kind() {
                        std::io::ErrorKind::NotFound => {
                            "VLC command-line player not found; install VLC and ensure `cvlc` is on PATH.".to_string()
                        }
                        _ => format!("Failed to start VLC playback: {}", err),
                    })
                }
            }
        }

        pub fn stop(&mut self) {
            if let Some(mut child) = self.process.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
            self.stdin = None;
        }

        #[allow(dead_code)]
        pub fn is_playing(&self) -> bool {
            self.process.is_some()
        }

        pub fn volume_up(&mut self) {
            let next = self.volume.saturating_add(5).min(100);
            self.set_volume(next);
        }

        pub fn volume_down(&mut self) {
            let next = self.volume.saturating_sub(5);
            self.set_volume(next);
        }

        pub fn volume(&self) -> u8 {
            self.volume
        }

        fn set_volume(&mut self, percent: u8) {
            self.volume = percent.min(100);
            let _ = self.send_vlc_command(&format!("volume {}\n", self.vlc_volume()));
        }

        fn send_vlc_command(&mut self, cmd: &str) -> std::io::Result<()> {
            if let Some(stdin) = &mut self.stdin {
                stdin.write_all(cmd.as_bytes())?;
                stdin.flush()?;
                Ok(())
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "VLC stdin not available",
                ))
            }
        }

        fn vlc_volume(&self) -> u32 {
            (self.volume as u32 * 256) / 100
        }
    }

    impl Drop for Player {
        fn drop(&mut self) {
            self.stop();
        }
    }

    #[cfg(test)]
    mod tests {
        use super::Player;

        #[test]
        fn player_starts_at_expected_volume() {
            let player = Player::new();
            assert_eq!(player.volume(), 50);
            assert!(!player.is_playing());
        }

        #[test]
        fn volume_changes_are_clamped() {
            let mut player = Player::new();
            for _ in 0..20 {
                player.volume_up();
            }
            assert_eq!(player.volume(), 100);

            for _ in 0..30 {
                player.volume_down();
            }
            assert_eq!(player.volume(), 0);
        }

        #[test]
        fn stop_is_safe_without_running_process() {
            let mut player = Player::new();
            player.stop();
            assert!(!player.is_playing());
        }

        #[test]
        fn missing_cvlc_returns_clear_error() {
            let mut player = Player::new();
            let original_path = std::env::var_os("PATH");
            unsafe {
                std::env::set_var("PATH", "");
            }

            let err = player
                .play("http://example.invalid/stream.mp3")
                .expect("missing cvlc should return an error");

            match original_path {
                Some(path) => unsafe { std::env::set_var("PATH", path) },
                None => unsafe { std::env::remove_var("PATH") },
            }

            assert!(err.contains("cvlc"));
        }
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use windows::{
        Foundation::Uri,
        Media::{Core::MediaSource, Playback::MediaPlayer},
        core::HSTRING,
    };

    pub struct Player {
        player: Option<MediaPlayer>,
        volume: u8,
        is_playing: bool,
    }

    impl Player {
        pub fn new() -> Self {
            Self {
                player: None,
                volume: 50,
                is_playing: false,
            }
        }

        pub fn play(&mut self, url: &str) -> Option<String> {
            let uri = match Uri::CreateUri(&HSTRING::from(url)) {
                Ok(uri) => uri,
                Err(err) => return Some(format!("Invalid stream URL: {}", err)),
            };

            let source = match MediaSource::CreateFromUri(&uri) {
                Ok(source) => source,
                Err(err) => return Some(format!("Failed to create Windows media source: {}", err)),
            };

            let player = match MediaPlayer::new() {
                Ok(player) => player,
                Err(err) => return Some(format!("Windows media backend unavailable: {}", err)),
            };

            let _ = player.SetVolume(self.volume as f64 / 100.0);
            if let Err(err) = player.SetSource(&source) {
                return Some(format!(
                    "Failed to configure Windows playback source: {}",
                    err
                ));
            }
            if let Err(err) = player.Play() {
                return Some(format!("Failed to start Windows playback: {}", err));
            }

            self.player = Some(player);
            self.is_playing = true;
            None
        }

        pub fn stop(&mut self) {
            if let Some(player) = &self.player {
                let _ = player.Pause();
            }
            self.player = None;
            self.is_playing = false;
        }

        #[allow(dead_code)]
        pub fn is_playing(&self) -> bool {
            self.is_playing
        }

        pub fn volume_up(&mut self) {
            let next = self.volume.saturating_add(5).min(100);
            self.set_volume(next);
        }

        pub fn volume_down(&mut self) {
            let next = self.volume.saturating_sub(5);
            self.set_volume(next);
        }

        pub fn volume(&self) -> u8 {
            self.volume
        }

        fn set_volume(&mut self, percent: u8) {
            self.volume = percent.min(100);
            if let Some(player) = &self.player {
                let _ = player.SetVolume(self.volume as f64 / 100.0);
            }
        }
    }

    impl Drop for Player {
        fn drop(&mut self) {
            self.stop();
        }
    }

    #[cfg(test)]
    mod tests {
        use super::Player;

        #[test]
        fn player_starts_at_expected_volume() {
            let player = Player::new();
            assert_eq!(player.volume(), 50);
            assert!(!player.is_playing());
        }

        #[test]
        fn volume_changes_are_clamped() {
            let mut player = Player::new();
            for _ in 0..20 {
                player.volume_up();
            }
            assert_eq!(player.volume(), 100);

            for _ in 0..30 {
                player.volume_down();
            }
            assert_eq!(player.volume(), 0);
        }

        #[test]
        fn invalid_url_returns_clear_error() {
            let mut player = Player::new();
            let err = player.play("not a url").expect("invalid url should fail");
            assert!(err.contains("Invalid stream URL"));
        }

        #[test]
        fn stop_is_safe_before_play() {
            let mut player = Player::new();
            player.stop();
            assert!(!player.is_playing());
        }
    }
}

pub use imp::Player;
