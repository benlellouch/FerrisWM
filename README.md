# rdwm

rdwm is a small, minimalist dynamic window manager written in Rust using XCB.

Features
- Minimal dynamic tiling layout (even horizontal tiling across visible windows)
- Multiple workspaces (configurable via `NUM_WORKSPACES`)
- Keyboard-driven actions: spawn apps, kill focused client, change focus, switch workspaces
- Lightweight: few dependencies (xcb, xkbcommon)

Building
1. Install Rust toolchain (stable) and necessary X development libraries for `xcb` and `xkbcommon`.
2. Build in debug mode:

```bash
cargo build
```

3. Build a release binary:

```bash
cargo build --release
```

Running (preview)
- The repository includes `preview.sh` which builds and then uses `xinit` with `Xephyr` to run `rdwm` in a nested X server for testing.
- Example:

```bash
./preview.sh
```

Notes
- Configure key bindings and behavior in `src/config.rs` and `src/key_mapping.rs`.
- Logging uses the `log` and `env_logger` crates; run with `RUST_LOG=debug` to see debug output.
- This project is experimental — use in a nested session (Xephyr) for testing before using as your main window manager.

License
- (Add your license here)
