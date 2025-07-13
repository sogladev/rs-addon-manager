# Rs Game Launcher
This monorepo contains a cross-platform (Windows + Linux + MacOS) game launcher in 2 forms: minimal CLI downloader (Rust) and a GUI launcher (Tauri + Vue.js). It is designed to manage patch updates using a manifest. The launcher displays a transaction overview, detailed progress, and only overwrites files listed in the manifest without removing extra files.

The project is designed for easy extension with minimal dependencies, utilizing shared Rust libraries for backend operations while keeping the front-end lightweight.

![Socials](images/socials.png)

## Tech Stack
[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Vue.js](https://img.shields.io/badge/Vue.js-35495E?style=for-the-badge&logo=vue.js&logoColor=4FC08D)](https://vuejs.org)
[![Tauri](https://img.shields.io/badge/Tauri-24C8D8?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app)
[![DaisyUI](https://img.shields.io/badge/DaisyUI-5A0EF8?style=for-the-badge&logo=daisyui&logoColor=white)](https://daisyui.com)

### Core Technologies
- **CLI:** Rust-based command line interface
- **GUI:** Tauri + Vue.js application with DaisyUI components

## Feature Comparison

| Feature                                 | CLI (Rust)        | GUI (Tauri + Vue.js)   |
| --------------------------------------- | ------------------| ---------------------- |
| Manifest-based patch updates            | Yes               | Yes                    |
| Integrity verification                  | No                | Yes                    |
| Transaction overview                    | Console output    | Detailed visual output |
| Directory selection                     | No                | GUI-based              |
| Launch executable (wine on non-Windows) | No                | Yes                    |
| Customizable front-end                  | N/A               | Yes                    |
| Client download                         | Not planned       | Not yet supported      |
| Manage addons                           | Not planned       | Not planned            |
| Remove deprecated patches               | Not yet supported | Not yet supported      |

## CLI Details
A lightweight Rust-based terminal CLI for basic patching. It downloads patches from a `manifest.json`

![CLI](images/rs_patcher.gif)

## GUI Details
A Tauri + Vue.js wrapper around the same Rust libraries:
- Allows directory selection
- Provides art, transaction overviews, and a launch button
- Uses the same patching logic as the CLI

![Launcher Dark](images/tauri_game_launcher_dark.png)

https://github.com/user-attachments/assets/7c642947-a57c-46b0-aab9-eeb456b6e115

## Local Development

### Prerequisites
- Install Rust from the https://www.rust-lang.org/
- Local test CDN server to serve `manifest.json` and files
  - Install Go from https://go.dev/doc/install
  - or compiled binary https://github.com/sogladev/go-manifest-patcher/releases
- (Only for GUI) Install Bun package manager from https://bun.sh/docs/installation

### CLI

1. Start local CDN
    ```sh
    go run main.go -create-manifest
    go run main.go
    ```

From project root

2. Run the CLI
    ```sh
    cargo run --bin downloader-cli -- --manifest="http://localhost:8080/manifest.json"
    ```

3. Build
    ```sh
    cargo build --bin downloader-cli --release --locked
    cargo build --bin downloader-cli --target x86_64-pc-windows-gnu --release --locked
    ```

### GUI
1. Start local CDN
    ```sh
    go run main.go -create-manifest
    go run main.go
    ```

From `launcher-gui/`

2. Install dependencies:
    ```sh
    bun install
    ```

3. Start the development server
    ```sh
    bun run tauri dev
    ```

4. Build the project
    ```sh
    bun run tauri build
    ```

#### Fake client directory
```
mkdir -p client client/Data
touch client/Battle.net.dll
touch client/Data/lichking.MPQ
touch client/Data/patch-3.MPQ
```

## License
