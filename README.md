# cradio
Interactive terminal app for listening to internet radio on Linux and Windows 10/11.

## Features

- Browse and search radio stations from [radio-browser.info](https://www.radio-browser.info/)
- Filter by station name, tags, country code (ISO 3166-1), language (ISO 639), and bitrate
- Play radio streams with a native Rust audio backend
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

- ALSA development headers for native audio output
- Debian/Ubuntu example:

```bash
sudo apt install pkg-config libasound2-dev
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

- If the build fails while compiling audio dependencies, install `pkg-config` and `libasound2-dev`.
- If no sound device is available, `cradio` will show an audio output error in the UI.

### Windows

- Run the app in Windows Terminal or PowerShell if the console host behaves oddly with raw mode.
- If playback fails, confirm the selected station is reachable and that Windows has an active output device.
- Some rare stream formats may fail to decode; those stations will surface an in-app playback error.

## License

MIT - see [LICENSE](LICENSE)
