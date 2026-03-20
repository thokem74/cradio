# cradio
Interactive terminal app for listening to internet radio stations on Linux and Windows.

## Features

- 🎵 Browse and search radio stations from [radio-browser.info](https://www.radio-browser.info/)
- 🌍 Filter by station name, tags, country code (ISO 3166-1), and language (ISO 639)
- 📻 Play streams with VLC
- 🎚️ Volume control
- 📄 Pagination through thousands of stations
- ★ Persist station favorites per user

## Supported Platforms

- Linux
- Windows

## Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- VLC media player

### Linux

- Install VLC from your package manager, for example:

```bash
sudo apt install vlc
```

`cradio` will try `cvlc` first and then `vlc`.

### Windows

- Install VLC from [videolan.org](https://www.videolan.org/)
- Make sure `vlc.exe` is on your `PATH`, or install it in the default `C:\Program Files\VideoLAN\VLC` location

## Build

### Linux

```bash
cargo build --release
```

The binary will be at `target/release/cradio`.

### Windows

In PowerShell:

```powershell
cargo build --release
```

The binary will be at `target\release\cradio.exe`.

## Usage

### Linux

```bash
./target/release/cradio
```

### Windows

```powershell
.\target\release\cradio.exe
```

## Key Bindings

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate station list |
| `Enter` | Play selected station |
| `/` | Open filter mode |
| `Space` | Add/remove selected station from favorites |
| `f` | Toggle favorites view in station pane |
| `Tab` | Switch to next filter field (in filter mode) |
| `Esc` | Exit filter mode |
| `s` | Stop playback |
| `n` | Next page |
| `p` | Previous page |
| `+` | Volume up |
| `-` | Volume down |
| `q` | Quit |

## Filter Fields

- **Name** — partial station name (e.g. `Jazz FM`)
- **Tags** — comma-separated tags (e.g. `jazz,blues`)
- **Country (ISO)** — ISO 3166-1 country code (e.g. `US`, `DE`)
- **Language (ISO)** — ISO 639 language code (e.g. `en`, `de`)

Press `Enter` in filter mode to apply the search and return to the station list.

## Favorites Storage

Favorites are persisted as a JSON array of objects:

```json
[{"stationuuid":"...","name":"...","url":"..."}]
```

Default locations:

- Linux: `$XDG_CONFIG_HOME/cradio/favorites.json` when `XDG_CONFIG_HOME` is set, otherwise `$HOME/.cradio/favorites.json`
- Windows: `%APPDATA%\cradio\favorites.json`

## License

MIT — see [LICENSE](LICENSE)
