# amptop

> Interactive terminal UI for monitoring laptop battery statistics

**amptop** is a lightweight, cross-platform TUI (Terminal User Interface) application that provides real-time battery monitoring with historical data tracking. Built with Rust for performance and reliability.

## Features

- **Real-time Monitoring** - Live battery statistics including charge level, power consumption, and thermal data
- **Historical Tracking** - Background daemon collects battery statistics over time for trend analysis
- **Cross-platform** - Supports Linux, macOS, FreeBSD, and DragonFlyBSD (Windows support planned)
- **Visual Graphs** - Interactive charts showing battery drain patterns and usage history
- **Lightweight** - Minimal resource usage, runs efficiently in any terminal
- **Multiple Units** - Display metrics in human-readable or SI units

## Installation

### From Source

```bash
git clone https://github.com/harshvsri/amptop.git
cd amptop
cargo build --release
```

The binary will be available at `target/release/amptop`.

### Using Cargo

```bash
cargo install --git https://github.com/harshvsri/amptop
```

## Usage

### Interactive TUI Mode

Launch the interactive terminal interface:

```bash
amptop
```

**Keyboard Controls:**
- `q` or `Esc` - Quit application
- `Ctrl+C` - Force exit

**Options:**
- `-d, --delay <SECONDS>` - Set update interval (default: 1 second)
- `-u, --units <human|si>` - Choose measurement units (default: human)

```bash
amptop --delay 2 --units si
```

### Background Daemon

Start collecting battery statistics in the background:

```bash
amptop daemon start --interval 60
```

The daemon logs battery data to `~/.local/share/amptop/battery.db` at the specified interval (in seconds). Recommended interval: 60-300 seconds.

**Daemon Commands:**
- `amptop daemon start --interval <SECONDS>` - Start background monitoring
- `amptop daemon stop` - Stop the daemon
- `amptop daemon status` - Check daemon status

## Project Status

**⚠️ Early Development** - This project is in active development. Features and APIs may change. Contributions and feedback are welcome!

## Contributing

Contributions are welcome! Here's how you can help:

1. **Report Bugs** - Open an issue describing the problem
2. **Suggest Features** - Share ideas for improvements
3. **Submit PRs** - Fork the repo and submit pull requests
4. **Documentation** - Help improve docs and examples

Please ensure your code follows the existing style and includes appropriate tests.

## Architecture

- **TUI Framework** - [ratatui](https://github.com/ratatui-org/ratatui) for terminal rendering
- **Battery APIs** - [battery](https://github.com/svartalf/rust-battery) for cross-platform battery access
- **Storage** - SQLite for historical data persistence
- **CLI Parsing** - [clap](https://github.com/clap-rs/clap) for command-line argument handling

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
