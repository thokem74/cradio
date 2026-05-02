# Contributing

Thanks for helping improve `cradio`.

## Development Setup

Install Rust from [rustup.rs](https://rustup.rs/).

On Linux, install the playback and build prerequisites:

```bash
sudo apt install vlc pkg-config libssl-dev
```

Then clone the repository and verify the project:

```bash
cargo test
cargo check
cargo fmt --check
```

## Running Locally

```bash
cargo run --release
```

On Linux, playback requires `cvlc` to be available on `PATH`. On Windows 10/11, playback uses the native Windows media backend.

## Making Changes

- Keep changes focused and easy to review.
- Follow the existing code style and module layout.
- Run formatting and tests before opening a pull request.
- Update `README.md` when user-facing behavior, setup, or commands change.
- Update `CHANGELOG.md` for notable changes.

## Versioning

This project follows [Semantic Versioning](https://semver.org/).

- Bug fixes increment the patch version, for example `0.1.0` to `0.1.1`.
- New features or breaking changes before `1.0.0` increment the minor version, for example `0.1.0` to `0.2.0`.
- After `1.0.0`, breaking changes increment the major version.

The package version in `Cargo.toml` is the source of truth.

## Pull Requests

Before opening a pull request, run:

```bash
cargo test
cargo check
cargo fmt --check
```

Include a short summary of what changed and note any platform-specific testing you performed.
