# Smart screen 🦀

An application for collecting and sending metrics to a 3.5-inch smart screen.

> **Status:** Work in progress.

# Prerequisites

## Windows Metrics (CPU Temperature)

To collect CPU temperature metrics on Windows (Intel/AMD), you need to:

1. Install [LibreHardwareMonitor](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor).
2. Run it and enable the **HTTP Server** in the settings (`Options` -> `Remote Web Server` -> `Run`).
3. Set it to **Run on Windows Startup** for continuous monitoring.

## Cross-compilation

To build a Windows application from Linux, you need to install the appropriate cross-compiler package:

```bash

sudo apt-get install gcc-mingw-w64-x86-64
rustup target add x86_64-pc-windows-gnu
```

# Build and Run

## Windows

```bash

cargo build --target x86_64-pc-windows-gnu --release
```

```bash
target/x86_64-pc-windows-gnu/release/smart_display.exe
```

## Linux

```bash
cargo build --release
```

```bash
target/release/smart_screen
```

# Run tests

```bash
cargo test
```

## License

[MIT](LICENSE) © 2026 Maxim Shandruk
