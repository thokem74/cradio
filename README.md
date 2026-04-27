# cradio
Interactive terminal app for listening to internet radio on Linux and Windows 10/11.

## Features

- Browse and search radio stations from [radio-browser.info](https://www.radio-browser.info/)
- Filter by station name, tags, country code (ISO 3166-1), language (ISO 639), and bitrate
- Linux playback through VLC for broad station compatibility
- Windows 10/11 playback through a native Windows media backend
- Adjust playback volume from the keyboard
- Save favorites in an OS-native per-user config directory
- Page through large station result sets

## Supported Platforms

- Linux
- Windows 10
- Windows 11

## Prerequisites

- [Rust](https://rustup.rs/) with Cargo

### Linux

- VLC with the `cvlc` command available on `PATH`
- Debian/Ubuntu example:

```bash
sudo apt install vlc
```

### Windows

- PowerShell or Windows Terminal recommended
- No VLC installation required

## Verify

```bash
cargo test
cargo check
cargo fmt --check
```

## Build

```bash
cargo build --release

#cargo build --release --target x86_64-pc-windows-msvc
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

```

The binary will be at `target/release/cradio` on Linux and `target/release/cradio.exe` on Windows.

## Usage

```bash
cargo run --release
```

## Favorites Storage

- Linux: `~/.config/cradio/favorites.json` on most XDG-compliant systems
- Windows: `%APPDATA%\cradio\favorites.json`

No migration is performed from the older Linux-only `~/.cradio/favorites.json` path.

## Key Bindings

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate station list |
| `Enter` | Play selected station |
| `/` | Open filter mode |
| `Space` | Add/remove selected station from favorites |
| `f` | Toggle favorites view in station pane |
| `Tab` | Switch to next filter field in filter mode |
| `Esc` | Exit filter mode |
| `s` | Stop playback |
| `n` | Next page |
| `p` | Previous page |
| `+` | Volume up |
| `-` | Volume down |
| `q` | Quit |

## Filter Fields

- **Name**: partial station name such as `Jazz FM`
- **Tags**: comma-separated tags such as `jazz,blues`
- **Country**: ISO 3166-1 country code such as `US` or `DE`
- **Language**: ISO 639 language code such as `en` or `de`
- **Bitrate**: minimum bitrate in kbps

Press `Enter` in filter mode to apply the search and return to the station list.

## Troubleshooting

### Linux

- If playback fails immediately, verify that `cvlc` is installed and available on `PATH`.
- VLC handles a wider range of live radio streams than the previous native Linux backend, so Linux playback now prefers VLC intentionally.

### Windows

- Run the app in Windows Terminal or PowerShell if the console host behaves oddly with raw mode.
- If playback fails, confirm Windows has an active output device and that the edition includes the required media components.
- Stream URL fallback still comes only from Radio Browser data (`url_resolved`, `url`, and Radio Browser playlist URLs).

## License

MIT - see [LICENSE](LICENSE)
