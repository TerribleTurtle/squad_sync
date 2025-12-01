---
description: Build the desktop application for distribution
---

1. Navigate to the desktop app directory

```bash
cd apps/desktop
```

2. Configure the signaling server (Optional)
   If you are testing on another PC, you need to point the app to your signaling server (e.g., your dev machine's IP).
   Create a `.env` file in `apps/desktop` or set the variable inline:

```bash
# Example for Windows PowerShell
$env:VITE_PARTYKIT_HOST="192.168.2.202:1999"; pnpm tauri build
```

3. Run the build command
   // turbo

```bash
pnpm tauri build
```

3. Locate the installer
   The installer will be located in:
   `apps/desktop/src-tauri/target/release/bundle/nsis/squad_sync_0.1.0_x64-setup.exe`
