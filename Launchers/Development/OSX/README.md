# Arcadia Development Launcher

Minimal macOS menu bar launcher for the development GUI.

Run it with:

```sh
swift run
```

Build a clickable menu bar app with:

```sh
./build-app.sh
```

The launcher prefers `~/.local/bin/arcadia-gui` when it exists. If the global command has not been installed, it falls back to the same build-and-run flow used by that wrapper:

```sh
cargo build --manifest-path Desktop/Cargo.toml --no-default-features --features gui
Builds/workspace/debug/arcadia
```
