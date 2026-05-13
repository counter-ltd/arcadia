# Build and Run

Cargo artifacts for **every crate** in this repo go under **`Builds/workspace/`**, driven by [`/.cargo/config.toml`](../.cargo/config.toml) (`build.target-dir`). You normally **do not pass `--target-dir`**. One-off override: `CARGO_TARGET_DIR=/tmp/foo cargo build …`.

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

Artifacts go under `Builds/workspace/` with everything else:

```sh
cargo test -p arcadia-core --manifest-path Shared/Cargo.toml
```

## iOS app

The iOS app links a Rust static library (`libarcadia_ios.a`) built from the `arcadia` package with `--features ios-gui`. The Xcode project's "Build Rust (cargo)" phase runs cargo automatically when you build the app, so the normal flow is just:

```sh
# Install Rust targets once
rustup target add aarch64-apple-ios aarch64-apple-ios-sim

# Open and build
open Mobile/iOS/ArcadiaApp.xcodeproj
```

Or from the command line:

```sh
xcodebuild \
  -project Mobile/iOS/ArcadiaApp.xcodeproj \
  -scheme ArcadiaApp \
  -configuration Release \
  -sdk iphonesimulator \
  -destination "generic/platform=iOS Simulator" \
  -derivedDataPath Builds/Mobile/iOS/DerivedData/Simulator \
  build
```

To build only the static lib (for inspection or manual linking):

```sh
bash Shared/Scripts/Builds/build-ios-app.sh
```

## Launcher menus

```sh
bash Shared/Scripts/Launchers/Launcher.sh
pwsh  Shared/Scripts/Launchers/Launcher.ps1
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
