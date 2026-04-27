# cradio
Interactive terminal app for listening to internet radio on Linux and Windows 10/11.

## Features

- Browse and search radio stations from [radio-browser.info](https://www.radio-browser.info/)
- Filter by station name, tags, country code (ISO 3166-1), language (ISO 639), and bitrate
- Play streams on Linux using `cvlc` (VLC command-line player)
- Play streams on Windows 10/11 using the native Windows media backend
- Adjust playback volume from the keyboard
- Save favorites in an OS-native per-user config directory
- Page through large station result sets

## Supported Platforms

- Linux
- Windows 10
- Windows 11

## Prerequisites

- [Rust](https://rustup.rs/) (1.70+)

### Linux

- `cvlc` (VLC media player CLI)

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

Build on the current platform:

```bash
cargo build --release
```

Cross-compile a Windows GNU build from Linux:

```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

Compile a Windows MSVC build from Windows:

```bash
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

You also need the Microsoft C++ build tools installed. 
Easiest route:
1. Install Visual Studio Build Tools
2. Select Desktop development with C++
3. Make sure the Windows SDK and MSVC toolchain are included
4. Open PowerShell or Windows Terminal in the project folder
5. Run: cargo build --release --target x86_64-pc-windows-msvc

The binary will be at `target/release/cradio` on Linux, `target/release/cradio.exe` on native Windows builds, and `target/x86_64-pc-windows-gnu/release/cradio.exe` for the Linux cross-compile command above.

## Usage

```bash
cargo run --release
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
- **Bitrate** — minimum bitrate in kbps

Press `Enter` in filter mode to apply the search and return to the station list.

## Favorites Storage

Favorites are persisted as a JSON array of objects: `[{"stationuuid":"...","name":"...","url":"..."}]`.

- Linux: usually `~/.config/cradio/favorites.json`
- Windows: `%APPDATA%\cradio\favorites.json`

No migration is performed from the older Linux-only `~/.cradio/favorites.json` path.

## Troubleshooting

### Linux

- If playback fails immediately, verify that `cvlc` is installed and available on `PATH`.

### Windows

- Run the app in Windows Terminal or PowerShell if the console host behaves oddly with raw mode.
- If playback fails, confirm Windows has an active output device and that the edition includes the required media components.

## License

MIT - see [LICENSE](LICENSE)
