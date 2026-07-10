# Es-Pelem 🚀

[![Build App](https://github.com/endrisusanto/Es-Pelem/actions/workflows/release.yml/badge.svg)](https://github.com/endrisusanto/Es-Pelem/actions/workflows/release.yml)
[![Tauri](https://img.shields.io/badge/Tauri-v2-blue.svg?logo=tauri)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-v1.75+-orange.svg?logo=rust)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

**Es-Pelem** is a modern, lightweight firmware release tool designed to intercept, sync, and inspect Samsung firmware release logs from the SSCM portal. Built with a high-end, glassmorphic dark-theme UI on **Tauri (Rust + Vanilla HTML/CSS/JS)** and supported by a custom **SSCM Browser Extension** for seamless data extraction.

---

## 🏗️ Architecture Layout

The tool operates on a dual-component pipeline:

```mermaid
graph LR
    A[Samsung SSCM Portal] -->|getReleaseListAjax.do| B(Chrome Interceptor Extension)
    B -->|HTTP POST| C[Tauri Sync Server :14120]
    C -->|IPC Emitter| D[Tauri UI Dashboard]
```

1. **Chrome Interceptor Extension (`/extension`)**: 
   A lightweight Manifest V3 extension that hooks into Chrome's AJAX layer to intercept data packets from `getReleaseListAjax.do` and transparently push them to the local desktop app.
2. **Tauri App Dashboard (`/src-tauri` & `/src`)**:
   A native desktop app containing a zero-dependency Rust `TcpListener` that accepts incoming data from the extension, caches it in memory, and triggers interactive UI updates.

---

## 🌟 Key Features

* **Real-time Live Sync**: Intercepted firmware entries appear in your Tauri dashboard instantly as you browse the SSCM portal, with visual status feedback.
* **Granular Search Filters**: Instantly query entries on the fly by:
  - **AP Version** (`codeVersion`)
  - **CP Version** (`bbVersion`)
  - **CSC Version** (`cscVersion`)
  - **Dev Model Name** (`modelNm`)
  - **Release Date/Time Range**
* **Offline Mock Caching**: Pre-loaded with sample release logs so you can browse, filter, and test the app interface immediately offline.
* **Firmware Details Viewer**: Click on any grid row to open an overlay panel with extensive metadata (One UI version, Knox version, target countries list, changelist number, FOTA dates, status logs, and database keys).
* **Manual Data Importer**: A JSON paste panel that allows manual clipboard imports of raw responses if needed.

---

## 🛠️ Development & Local Setup

### Prerequisites

Ensure you have the standard Tauri and Rust tools installed:
* [Rust & Cargo](https://www.rust-lang.org/tools/install)
* [Node.js & npm](https://nodejs.org)
* Platform build tools (e.g., C compiler for Rust compilation).

### Running the Desktop App

1. Clone the repository:
   ```bash
   git clone https://github.com/endrisusanto/Es-Pelem.git
   cd Es-Pelem
   ```
2. Install npm CLI dependencies:
   ```bash
   npm install
   ```
3. Start the Tauri app in developer mode:
   ```bash
   npm run tauri dev
   ```

### Running the Chrome Interceptor Extension

1. Open Google Chrome (or any Chromium-based browser) and navigate to `chrome://extensions/`.
2. Enable **Developer mode** (top-right toggle switch).
3. Click **Load unpacked** (top-left button) and select the `/extension` directory inside this repository.
4. Browse the Samsung SSCM portal. A status badge in the bottom-right corner of the page will show your connection status to the Tauri app (Green for Connected, Red for Disconnected).

---

## 📦 Automated Release Pipeline

We use a tag-based automated compiler to build and publish Windows installer files (`.msi`).

### Local Release Trigger

To prepare a version bump, compile checks, and trigger a release:
1. Run the local release script from the root folder:
   ```bash
   # Bumps patch version (e.g., 0.1.0 -> 0.1.1) and triggers release
   ./scripts/release.sh patch
   
   # Bumps minor version (0.1.0 -> 0.2.0)
   ./scripts/release.sh minor
   
   # Or pass a specific semver tag
   ./scripts/release.sh 1.2.3
   ```
2. The script will automatically:
   - Update `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` with the new version.
   - Run compilation checks (`cargo check`).
   - Create a local git commit and tag (e.g. `v0.1.1`).
   - Push the code and tags to the remote repository.

### GitHub Actions Compiler

Upon receiving a version tag (`v*`), the GitHub Actions release workflow (`.github/workflows/release.yml`) automatically triggers:
1. Boots a **Windows Runner** (`windows-latest`).
2. Installs Rust and Node compilation hooks.
3. Packages the Tauri application into a production-ready **Windows MSI installer** using the built-in Wix toolkit.
4. Publishes a GitHub Release page containing the compiled MSI file.

---

## 📄 License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
