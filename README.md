# Resize Rabbit

A Windows desktop app to save and apply window size and position profiles for your applications — with drag-and-drop reordering.

Based on [Resize Raccoon](https://github.com/mistenkt/resize-raccoon) by mistenkt.

## Features

- Save window size and position profiles per application
- One-click apply to snap a window to its saved layout
- Drag-and-drop to reorder profiles on the home screen
- Process watcher to auto-apply profiles when an application launches
- Import profiles from an existing Resize Raccoon installation
- Launch on startup, minimize to tray

## Installation

Download the latest `.msi` from [Releases](https://github.com/rosscarlson/resize-rabbit/releases).

## Triggering profiles from scripts / Stream Deck

While the app is running you can apply a profile from a `.bat` file or command line:

```
echo apply-profile {profileName} > \\.\pipe\resize-rabbit
```

If your profile name contains spaces, wrap it in quotes:

```
echo apply-profile "my profile" > \\.\pipe\resize-rabbit
```

## Development

```bash
git clone https://github.com/rosscarlson/resize-rabbit.git
cd resize-rabbit
yarn install
yarn tauri dev
```

To build a release installer:

```bash
yarn tauri build
```

Requires Node.js, Yarn, Rust (stable-msvc), and Visual Studio Build Tools with the C++ workload.
