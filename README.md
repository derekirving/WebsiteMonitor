# Website Monitor

Experimental tool for monitoring web site availability.

## Build and run

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

## Resource Usage

| Technology | Cross Platform | Memory usage |
| ------------- | :-------------: |  ------------- |
| Electron | Yes | **128MB** |
| AvaloniaUI | Yes | **52MB** |
| WinForms | No | **16MB** |

So WinForms is the clear winner for a Windows only solution.