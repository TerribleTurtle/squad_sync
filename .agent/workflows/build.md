---
description: Build the desktop application for distribution
---

1. Navigate to the desktop app directory
```bash
cd apps/desktop
```

2. Run the build command
// turbo
```bash
pnpm tauri build
```

3. Locate the installer
The installer will be located in:
`apps/desktop/src-tauri/target/release/bundle/nsis/squad_sync_0.1.0_x64-setup.exe`
