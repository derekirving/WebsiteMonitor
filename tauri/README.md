# Website Monitor Tauri + Vanilla

## Setup

Install **Rust** from https://rustup.rs/

Install **Node.js** from https://nodejs.org/

For authentication, create a new registration on Azure.

1. Choose *Public client/native (mobile & desktop)* and set the *Redirect URI* as `http://localhost`.

2. In the **Authentication** tab under **Settings** ensure that **Allow public client flows** is enabled.

Make a note of the Client ID as it's required in [main.ts](./src/main.ts#L10).

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