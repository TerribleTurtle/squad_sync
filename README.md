# SquadSync

**SquadSync** is a distributed "Silent Recorder" for gaming squads. It captures high-quality gameplay locally and synchronizes clip generation across your entire squad, ensuring you never miss a moment from any perspective.
*   **Desktop App**: [Tauri v2](https://v2.tauri.app/) (Rust + React)
*   **Signaling**: [PartyKit](https://partykit.io/) (Serverless WebSockets)
*   **Video Engine**: FFmpeg (Sidecar Process)
*   **Storage**: Cloudflare R2

## 📂 Project Structure

This project is a monorepo managed with `pnpm` and `turborepo`.

*   `apps/desktop`: The main desktop application (Tauri).
*   `apps/signaling`: The WebSocket signaling server (PartyKit).
*   `packages/shared`: Shared TypeScript types, schemas, and constants.
*   `packages/ffmpeg-commands`: FFmpeg command builders and utilities.

## 🛠️ Tech Stack

*   **Frontend**: React, TailwindCSS, Zustand
*   **Backend (Desktop)**: Rust (Tauri v2)
*   **Signaling**: PartyKit (Cloudflare Workers)
*   **Video Processing**: FFmpeg (Sidecar)
*   **Build System**: TurboRepo, pnpm

## 🚦 Getting Started

### Prerequisites

*   **Node.js**: v18+
*   **Rust**: Stable
*   **pnpm**: `npm install -g pnpm`
*   **FFmpeg**: Installed and available in PATH (for development).
*   **FFmpeg**: Installed and available in PATH (for development).
    *   **Note**: The desktop app requires a specific FFmpeg binary. Run `pnpm setup:ffmpeg` in `apps/desktop` to download it.

### Installation

1.  Clone the repository:
    ```bash
    git clone https://github.com/TerribleTurtle/squad_sync.git
    cd squad_sync
    ```

2.  Install dependencies:
    ```bash
    pnpm install
    ```

    ```bash
    pnpm dev
    ```

### Building for Production

To build the desktop application:

```bash
# 1. Setup FFmpeg binary
cd apps/desktop
pnpm setup:ffmpeg
cd ../..

# 2. Build
pnpm build
# OR specifically for desktop
pnpm --filter desktop tauri build
```

### Development Workflow

*   **Local CI**: Run `pnpm run ci` to execute build, lint, and typecheck locally.
*   **CI/CD**: GitHub Actions is configured for manual triggers only. Go to the "Actions" tab on GitHub to run the CI pipeline.

## 📖 Documentation

*   [Technical Specification](technical_specification.md): Detailed system architecture and requirements.
*   [Developer Guide](developer_guide.md): Code patterns, standards, and examples.
*   [Phases & Roadmap](phases.md): High-level project roadmap.

### Developer Notes

*   **FFmpeg Sidecar**: The app uses a sidecar FFmpeg binary located in `apps/desktop/src-tauri/bin`. This is excluded from git to save space. Use `pnpm setup:ffmpeg` to fetch it.
*   **Logs**: Runtime logs are written to `devices.log` in the app root during development.
*   **Architecture**: The app records locally ("Silent Recorder") and syncs metadata via PartyKit. Video clips are uploaded to Cloudflare R2 only when requested.
*   **Smart Recording**:
    *   **Dynamic Bitrate**: Automatically calculates optimal bitrate based on resolution and framerate (0.1 bits/pixel).
    *   **Smart Scaler**: Bypasses scaling filters when recording at native resolution for zero-overhead capture.
    *   **Multi-Monitor**: Select any connected display for recording.
    *   **Crash Resilience**: Uses Fragmented MP4 (`movflags=+frag_keyframe+empty_moov`) for temporary buffer segments to ensure data integrity even if the app crashes.

## 🤝 Contributing

1.  Fork the repository.
2.  Create a feature branch (`git checkout -b feature/amazing-feature`).
3.  Commit your changes (`git commit -m 'Add some amazing feature'`).
4.  Push to the branch (`git push origin feature/amazing-feature`).
5.  Open a Pull Request.

## 📄 License

[ISC](LICENSE)
