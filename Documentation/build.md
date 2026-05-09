# Build and Run

## Desktop GUI

```sh
cd Desktop && cargo build --features gui
cd Desktop && cargo run --features gui
```

## Desktop CLI (headless)

Default features are `headless`:

```sh
cd Desktop && cargo run
```

## Desktop release

```sh
cd Desktop && cargo build --release --features gui
```

## Core tests

```sh
cd Shared && cargo test -p arcadia-core
```

## iOS framework + Swift bindings

Run after any change to `ffi.rs` or exported types:

```sh
bash Shared/Scripts/build-ios-framework.sh
```

Regenerates `Mobile/iOS/ArcadiaCore/Generated/` and rebuilds `ArcadiaCore.xcframework`. Then open `ArcadiaApp` in Xcode and build.

## Launcher menus

```sh
bash Shared/Scripts/Launcher.sh
pwsh  Shared/Scripts/Launcher.ps1
```

## Global wrappers (macOS)

```sh
bash Shared/Scripts/Installers/install-global-commands-macos.sh
```

Installs helpers to `~/.local/bin` — ensure it's on `PATH`.

## macOS dev launcher app

```sh
cd Launchers/Development/OSX && bash build-app.sh
```

See `Launchers/Development/OSX/README.md` for details.
