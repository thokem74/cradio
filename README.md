# cradio
Interactive command line tool to listen to internet radio stations for Linux.

## Features

- 🎵 Browse and search radio stations from [radio-browser.info](https://www.radio-browser.info/)
- 🌍 Filter by station name, tags, country code (ISO 3166-1), and language (ISO 639)
- 📻 Play streams using `cvlc` (VLC command-line player)
- 🎚️ Volume control
- 📄 Pagination through thousands of stations

## Prerequisites

- Linux
- [Rust](https://rustup.rs/) (1.70+)
- `cvlc` (VLC media player CLI) — `sudo apt install vlc` or equivalent

## Build

```bash
cargo build --release
```

The binary will be at `target/release/cradio`.

## Usage

```bash
./target/release/cradio
```

### Key Bindings

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

### Filter Fields

- **Name** — partial station name (e.g. `Jazz FM`)
- **Tags** — comma-separated tags (e.g. `jazz,blues`)
- **Country (ISO)** — ISO 3166-1 country code (e.g. `US`, `DE`)
- **Language (ISO)** — ISO 639 language code (e.g. `en`, `de`)

Press `Enter` in filter mode to apply the search and return to the station list.

Favorites are persisted in `$HOME/.cradio/favorites.json` as a JSON array of objects: `[{"stationuuid":"...","name":"...","url":"..."}]`.

## License

MIT — see [LICENSE](LICENSE)
