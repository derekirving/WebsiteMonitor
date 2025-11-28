# Website Monitor

Experimental tool for monitoring web site availability.

UI requirements were a desktop app that could be minimised to the tray and support native OS notifications.

## Build and run

For Electron and AvaloniaUI, Install **Node.js** from https://nodejs.org/

For Tauri, Install **Node.js** from https://nodejs.org/ and on Windows, download the **Build Tools for Visual Studio 2022**: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022.

### Electron

```bash
cd electron
npm ci
npm start
```

### AvaloniaUi

```bash
cd avalonia
dotnet restore
dotnet run
```

### WinForms

```bash
cd winforms
dotnet restore
dotnet run
```

### Tauri

```bash
cd tauri
npm ci
npm run tauri dev
```

## Resource Usage

| Technology | Cross Platform | Memory usage |
| ------------- | :-------------: |  ------------- |
| Electron | Yes | **128MB** |
| AvaloniaUI | Yes | **52MB** |
| WinForms | No | **16MB** |
| Tauri | Yes | **4MB** |

So Tauri is the clear winner. You just need to learn [rust](https://rust-lang.org/)!