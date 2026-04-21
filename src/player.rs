use rodio::{Decoder, OutputStream, Sink};
use stream_download::source::DecodeError;
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload};
use tokio::runtime::Handle;
use tokio::task;

pub struct Player {
    backend: Box<dyn PlaybackBackend>,
}

trait PlaybackBackend {
    fn play(&mut self, url: &str) -> Option<String>;
    fn stop(&mut self);
    fn set_volume(&mut self, percent: u8);
    fn volume(&self) -> u8;
    fn is_playing(&self) -> bool;
}

pub struct NativePlayer {
    stream: Option<OutputStream>,
    sink: Option<Sink>,
    volume: u8,
}

impl Player {
    pub fn new() -> Self {
        Self {
            backend: Box::new(NativePlayer::new()),
        }
    }

    pub fn play(&mut self, url: &str) -> Option<String> {
        self.backend.play(url)
    }

    pub fn stop(&mut self) {
        self.backend.stop();
    }

    #[allow(dead_code)]
    pub fn is_playing(&self) -> bool {
        self.backend.is_playing()
    }

    pub fn volume_up(&mut self) {
        let next = self.volume().saturating_add(5).min(100);
        self.backend.set_volume(next);
    }

    pub fn volume_down(&mut self) {
        let next = self.volume().saturating_sub(5);
        self.backend.set_volume(next);
    }

    pub fn volume(&self) -> u8 {
        self.backend.volume()
    }
}

impl NativePlayer {
    fn new() -> Self {
        Self {
            stream: None,
            sink: None,
            volume: 50,
        }
    }

    fn build_stream(url: &str) -> Result<Decoder<StreamDownload<TempStorageProvider>>, String> {
        let url = url
            .parse()
            .map_err(|e| format!("Invalid stream URL: {}", e))?;
        let reader = task::block_in_place(|| {
            Handle::current().block_on(async move {
                match StreamDownload::new_http(url, TempStorageProvider::new(), Settings::default())
                    .await
                {
                    Ok(reader) => Ok(reader),
                    Err(err) => Err(err.decode_error().await),
                }
            })
        })
        .map_err(|msg| format!("Network stream open failure: {}", msg))?;

        Decoder::new(reader).map_err(|e| format!("Unsupported or undecodable stream format: {}", e))
    }

    fn set_sink_volume(&self) {
        if let Some(sink) = &self.sink {
            sink.set_volume(self.volume as f32 / 100.0);
        }
    }
}

impl PlaybackBackend for NativePlayer {
    fn play(&mut self, url: &str) -> Option<String> {
        self.stop();

        let (stream, stream_handle) = match OutputStream::try_default() {
            Ok(stream) => stream,
            Err(err) => return Some(format!("Audio output device unavailable: {}", err)),
        };

        let sink = match Sink::try_new(&stream_handle) {
            Ok(sink) => sink,
            Err(err) => return Some(format!("Audio output device unavailable: {}", err)),
        };
        sink.set_volume(self.volume as f32 / 100.0);

        let source = match Self::build_stream(url) {
            Ok(source) => source,
            Err(err) => return Some(err),
        };

        sink.append(source);
        self.stream = Some(stream);
        self.sink = Some(sink);
        None
    }

    fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.stream = None;
    }

    fn set_volume(&mut self, percent: u8) {
        self.volume = percent.min(100);
        self.set_sink_volume();
    }

    fn volume(&self) -> u8 {
        self.volume
    }

    fn is_playing(&self) -> bool {
        self.sink
            .as_ref()
            .map(|sink| !sink.empty())
            .unwrap_or(false)
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::{NativePlayer, PlaybackBackend, Player};

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
    fn play_failure_returns_error() {
        let mut player = NativePlayer::new();
        let err = player.play("not a url").expect("invalid url must fail");
        assert!(err.contains("Invalid stream URL"));
    }

    #[test]
    fn stop_clears_playback_state() {
        let mut player = Player::new();
        player.stop();
        assert!(!player.is_playing());
    }
}
