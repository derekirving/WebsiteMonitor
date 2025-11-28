# Website Monitor Tauri + Vanilla

## Setup

Install **Rust** from https://rustup.rs/
Install **Node.js** from https://nodejs.org/

### Windows

Download the **Build Tools for Visual Studio 2022**: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022

## Development

```bash
npm run tauri dev
```

## Production

```bash
npm run tauri build
```

Output will be in `src-tauri\target\release`

## Notes

Change the app icon

```bash
npm run tauri icon ./src/icon.png
```

Read the Rust documentation offline:

```bash
rustup doc
```