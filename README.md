# Rs Game Suite

> Two Separate Utilities: Each tool can be downloaded and used independently. Check the [Releases](../../releases) page for separate downloads.

This monorepo contains two standalone applications for WoW

## Git-Based Addon Manager

A modern GUI addon manager that uses Git repositories to install, update, and manage your WoW addons across multiple directories.

![Socials Addon Manager](images/socials-addon-manager.png)

**Key Features:**

- Install addons directly from GitHub/GitLab URLs
- Bulk install/update all addons
- Manage multiple addon directories
- Import/export addon lists
- Cross-platform (Windows, Linux, macOS)

## Game Launcher & Patcher

A graphical launcher featuring a built-in patching system. Previously compatible with [Project Epoch](https://www.project-epoch.net/play/).

![Socials Launcher](images/socials-launcher.png)

**Key Features:**

- Manifest-based patch system
- Visual transaction overviews
- Game launching with Wine support if available
- CLI patcher also available

[//]: # 'Table of Contents'

## Table of Contents

- [Git-Based Addon Manager](#-git-based-addon-manager)
    - [Features](#addon-manager-features)
    - [How to Use](#addon-manager-usage)
- [Game Launcher & Patcher](#-game-launcher--patcher)
    - [Features](#patcher-features)
    - [CLI Patcher](#cli)
    - [GUI Launcher](#gui)
- [Tech Stack](#tech-stack)
- [Local Development](#local-development)
- [Acknowledgements](#Acknowledgements)

---

## Addon Manager Features

| Feature                                                                        | Supported         |
| ------------------------------------------------------------------------------ | ----------------- |
| Windows, Linux, MacOS                                                          | Yes               |
| Install, Remove, Update addons                                                 | Yes               |
| git sources GitHub, GitLab, etc                                                | Yes               |
| Install all                                                                    | Yes               |
| Update all                                                                     | Yes               |
| Multiple addon directories                                                     | Yes               |
| Import / Export addon list                                                     | Yes               |
| Switch Git branches                                                            | Yes               |
| Manage subaddons                                                               | Yes               |
| Install via Git (HTTPS)                                                        | Yes               |
| Install via Git (SSH)                                                          | Not planned       |
| Install from GitHub Releases or Packages                                       | Not yet supported |
| Manage non-Git addons                                                          | Not yet supported |
| Auto updater with [Tauri Updater plugin](https://v2.tauri.app/plugin/updater/) | Not yet supported |

## Addon Manager Usage

Usage:

1. Download the latest release for your platform
1. Install and launch the application
1. Select your WoW `Interface/AddOns` folder
1. Add addons using their Git repository URLs

Example Git URLs:

- `https://github.com/Sattva-108/AdiBags.git`
- `https://gitlab.com/username/addon-name.git`

![Clone](images/addon-manager/clone.png)
![Menu](images/addon-manager/main-menu.png)

https://github.com/user-attachments/assets/2491b729-0b62-41d4-bf91-dabb3065cbea

**Import Format Example:**

```
C:\Games\wow335\Interface\AddOns AdiBags *https://github.com/Sattva-108/AdiBags.git main
```

---

## Patcher Features

| Feature                                 | CLI (Rust)     | GUI (Tauri + Vue.js)   |
| --------------------------------------- | -------------- | ---------------------- |
| Manifest-based patch updates            | Yes            | Yes                    |
| Integrity verification                  | No             | Yes                    |
| Remove deprecated patches               | Yes            | Yes                    |
| Transaction overview                    | Console output | Detailed visual output |
| Directory selection                     | No             | GUI-based              |
| Launch executable (wine on non-Windows) | No             | Yes                    |
| Customizable front-end                  | N/A            | Yes                    |
| Client download                         | Not planned    | Not yet supported      |
| Manage addons                           | Not planned    | Not planned            |

### CLI

A lightweight Rust-based terminal CLI for basic patching. It downloads patches from a `manifest.json`

How to use:

- Download the CLI binary for your platform from the release
- Place the binary in your WoW folder where `Wow.exe` is located
- Run the CLI binary to apply patches

### GUI

A Tauri + Vue.js wrapper around the same Rust libraries:

- Allows directory selection
- Provides art, transaction overviews, and a launch button
- Uses the same patching logic as the CLI

![Launcher Dark](images/tauri_game_launcher_dark.png)

https://github.com/user-attachments/assets/7c642947-a57c-46b0-aab9-eeb456b6e115

How to use:

- Download the GUI binary for your platform from the release
- Install the GUI application before launching it. This does not need to be your game folder
- Upon first launch, select your WoW game folder
- After patching is complete, the launcher can start the game executable.
  note: On non-Windows platforms, Wine will use the `.wine` directory located in the game folder by default. This behavior can be overridden by setting the `WINEPREFIX` environment variable before launching the GUI.

## Tech Stack

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Vue.js](https://img.shields.io/badge/Vue.js-35495E?style=for-the-badge&logo=vue.js&logoColor=4FC08D)](https://vuejs.org)
[![Tauri](https://img.shields.io/badge/Tauri-24C8D8?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app)
[![DaisyUI](https://img.shields.io/badge/DaisyUI-5A0EF8?style=for-the-badge&logo=daisyui&logoColor=white)](https://daisyui.com)

### Core Technologies

- **CLI:** Rust-based command line interface
- **GUI:** Tauri + Vue.js application with DaisyUI components

## Local Development

### Prerequisites

- Install Rust from the https://www.rust-lang.org/
- Install Bun package manager from https://bun.sh/docs/installation

### Formatting & Pre-commit

To set up formatting and the pre-commit hook:

1. Install Prettier and the Vue Prettier config:

    ```sh
    bun add -D prettier @vue/eslint-config-prettier
    ```

2. Set up the pre-commit hook by creating `.git/hooks/pre-commit` with:

    ```bash
    #!/bin/sh

    cargo fmt --all
    bun run prettier --write .
    ```

    Make sure the hook is executable:

    ```sh
    chmod +x .git/hooks/pre-commit
    ```

3. Run manually
    ```sh
    bunx eslint --ext .ts,.vue addon-manager-gui/src/
    ```

### Local CDN

The local CDN serves patch files and the manifest for both the CLI and GUI. Run these commands from the project root:

```sh
cd manifest-cdn
# Generate manifest.json (run once or when files change)
cargo run --bin manifest-cdn -- --create
# Start the local CDN server
cargo run --bin manifest-cdn
```

## CLI

From project root

1. Run the CLI

    ```sh
    # Demo mode (default)
    cargo run --bin downloader-cli

    # Production mode (sets production banner, description, and manifest URL)
    cargo run --features production --bin downloader-cli
    ```

1. Build

    ```sh
    # Demo build (default)
    cargo build --bin downloader-cli --release --locked
    cargo build --bin downloader-cli --target x86_64-pc-windows-gnu --release --locked

    # Production build (sets production banner, description, and manifest URL)
    cargo build --features production --bin downloader-cli --release --locked
    cargo build --features production --bin downloader-cli --target x86_64-pc-windows-gnu --release --locked
    ```

### GUI

From `launcher-gui/`

1. Install dependencies:

    ```sh
     bun install
    ```

1. Start the development server

    ```sh
    # Demo mode (default)
    bun run tauri dev

    # Production mode (sets production banner, description, and manifest URL in the GUI)
    bun run tauri dev --features production
    ```

1. Build the project

    ```sh
    # Demo build (default)
    bun run tauri build

    # Production build (sets production banner, description, and manifest URL in the GUI)
    bun run tauri build --features production
    ```

#### Fake client directory setup

The downloader CLI does not do any client validation.

```
mkdir -p client client/Data
touch client/Battle.net.dll
touch client/Data/lichking.MPQ
touch client/Data/patch-3.MPQ
```

## Acknowledgements

The Addon Manager UI and features were inspired by [GitAddonsManager](https://gitlab.com/woblight/GitAddonsManager) and other existing Wow Addon Managers. For an overview of existing addon managers, I recommend this comparison video by Arcane Intellect: [WoW Addon Managers Compared](https://www.youtube.com/watch?v=_V0RZG4YRVY)

## License
