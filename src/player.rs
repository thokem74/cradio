use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};

pub struct Player {
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    pub volume: u8,
    backend: VlcBackend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Platform {
    Linux,
    Windows,
}

#[derive(Debug, Clone)]
struct VlcBackend {
    candidates: Vec<PathBuf>,
    install_hint: &'static str,
}

impl Player {
    pub fn new() -> Self {
        Self {
            process: None,
            stdin: None,
            volume: 50,
            backend: VlcBackend::for_current_platform(),
        }
    }

    pub fn play(&mut self, url: &str) -> Option<String> {
        self.stop();

        let vol_arg = self.vlc_volume().to_string();
        for candidate in &self.backend.candidates {
            let mut cmd = build_vlc_command(candidate, &vol_arg, url);

            match cmd.spawn() {
                Ok(mut child) => {
                    self.stdin = child.stdin.take();
                    self.process = Some(child);
                    return None;
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
                Err(err) => {
                    self.process = None;
                    return Some(format!(
                        "Failed to start VLC using {}: {}",
                        candidate.display(),
                        err
                    ));
                }
            }
        }

        self.process = None;
        Some(format!("VLC was not found. {}", self.backend.install_hint))
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

impl VlcBackend {
    fn for_current_platform() -> Self {
        if cfg!(windows) {
            Self::for_platform(Platform::Windows, &env::vars_os().collect::<Vec<_>>())
        } else {
            Self::for_platform(Platform::Linux, &env::vars_os().collect::<Vec<_>>())
        }
    }

    fn for_platform(
        platform: Platform,
        env_vars: &[(std::ffi::OsString, std::ffi::OsString)],
    ) -> Self {
        match platform {
            Platform::Linux => Self {
                candidates: vec![PathBuf::from("cvlc"), PathBuf::from("vlc")],
                install_hint: "Install VLC from your package manager, for example `sudo apt install vlc`.",
            },
            Platform::Windows => Self {
                candidates: windows_vlc_candidates(env_vars),
                install_hint: "Install VLC and make sure `vlc.exe` is on PATH, or install it in `C:\\Program Files\\VideoLAN\\VLC`.",
            },
        }
    }
}

fn build_vlc_command(executable: &Path, volume: &str, url: &str) -> Command {
    let mut cmd = Command::new(executable);
    cmd.args([
        "--no-video",
        "--quiet",
        "--intf",
        "rc",
        "--rc-fake-tty",
        "--volume",
        volume,
        url,
    ])
    .stdin(Stdio::piped())
    .stdout(Stdio::null())
    .stderr(Stdio::null());
    cmd
}

fn windows_vlc_candidates(env_vars: &[(std::ffi::OsString, std::ffi::OsString)]) -> Vec<PathBuf> {
    let mut candidates = vec![PathBuf::from("vlc.exe"), PathBuf::from("vlc")];

    for key in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        if let Some(base) = env_lookup(env_vars, key) {
            candidates.push(windows_path(base, &["VideoLAN", "VLC", "vlc.exe"]));
        }
    }

    dedup_paths(candidates)
}

fn env_lookup<'a>(
    env_vars: &'a [(std::ffi::OsString, std::ffi::OsString)],
    key: &str,
) -> Option<&'a std::ffi::OsStr> {
    env_vars
        .iter()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value.as_os_str())
}

fn dedup_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut deduped = Vec::new();
    for path in paths {
        if !deduped.iter().any(|existing| existing == &path) {
            deduped.push(path);
        }
    }
    deduped
}

fn windows_path(base: &std::ffi::OsStr, segments: &[&str]) -> PathBuf {
    let mut path = base
        .to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .to_string();
    for segment in segments {
        if !path.is_empty() {
            path.push('\\');
        }
        path.push_str(segment);
    }
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::{Platform, VlcBackend, build_vlc_command};
    use std::ffi::OsString;
    use std::path::Path;

    #[test]
    fn linux_backend_prefers_cvlc() {
        let backend = VlcBackend::for_platform(Platform::Linux, &[]);
        assert_eq!(backend.candidates[0], Path::new("cvlc"));
        assert_eq!(backend.candidates[1], Path::new("vlc"));
    }

    #[test]
    fn windows_backend_checks_path_and_common_install_locations() {
        let backend = VlcBackend::for_platform(
            Platform::Windows,
            &[
                (
                    OsString::from("ProgramFiles"),
                    OsString::from(r"C:\Program Files"),
                ),
                (
                    OsString::from("ProgramFiles(x86)"),
                    OsString::from(r"C:\Program Files (x86)"),
                ),
            ],
        );

        assert_eq!(backend.candidates[0], Path::new("vlc.exe"));
        assert_eq!(backend.candidates[1], Path::new("vlc"));
        assert!(
            backend
                .candidates
                .iter()
                .any(|path| path == Path::new(r"C:\Program Files\VideoLAN\VLC\vlc.exe"))
        );
        assert!(
            backend
                .candidates
                .iter()
                .any(|path| path == Path::new(r"C:\Program Files (x86)\VideoLAN\VLC\vlc.exe"))
        );
    }

    #[test]
    fn vlc_command_uses_rc_interface() {
        let cmd = build_vlc_command(Path::new("vlc"), "128", "https://stream");
        let program = cmd.get_program().to_string_lossy().into_owned();
        let args: Vec<String> = cmd
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert_eq!(program, "vlc");
        assert_eq!(
            args,
            vec![
                "--no-video",
                "--quiet",
                "--intf",
                "rc",
                "--rc-fake-tty",
                "--volume",
                "128",
                "https://stream",
            ]
        );
    }
}
