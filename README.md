# SquadSync

**SquadSync** is a distributed "Silent Recorder" for gaming squads. It captures high-quality gameplay locally and synchronizes clip generation across your entire squad, ensuring you never miss a moment from any perspective.

## 🚀 Key Features

*   **Silent Recording**: Zero-latency local recording using a rolling buffer (60s default).
*   **Squad Synchronization**: One button press saves the last 60 seconds for *everyone* in the lobby.
*   **Multi-POV Playback**: Watch the action unfold from every angle in a synchronized grid view.
*   **Cloud Integration**: Clips are automatically uploaded to the cloud for instant sharing.
*   **Resource Efficient**: Hardware-accelerated encoding (NVENC/AMF/QSV) minimizes impact on game performance.
*   **Smart Capture**: Automatically targets your primary monitor at native resolution.
*   **Configurable**: Customize resolution, bitrate, framerate, and encoder via `config.toml`.

## 🛠️ Tech Stack

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

## 🚦 Getting Started

### Prerequisites

*   **Node.js**: v18+
*   **Rust**: Stable
*   **pnpm**: `npm install -g pnpm`
*   **FFmpeg**: Installed and available in PATH (for development).
    *   **Note**: The desktop app also requires an `ffmpeg` binary in `apps/desktop/src-tauri/bin/` (or `externalBin` configured) for the release build.

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

### Development Workflow

*   **Local CI**: Run `pnpm run ci` to execute build, lint, and typecheck locally.
*   **CI/CD**: GitHub Actions is configured for manual triggers only. Go to the "Actions" tab on GitHub to run the CI pipeline.

## 📖 Documentation

*   [Technical Specification](technical_specification.md): Detailed system architecture and requirements.
*   [Developer Guide](developer_guide.md): Code patterns, standards, and examples.
*   [Phases & Roadmap](phases.md): High-level project roadmap.

## 🤝 Contributing

1.  Fork the repository.
2.  Create a feature branch (`git checkout -b feature/amazing-feature`).
3.  Commit your changes (`git commit -m 'Add some amazing feature'`).
4.  Push to the branch (`git push origin feature/amazing-feature`).
5.  Open a Pull Request.

## 📄 License

[ISC](LICENSE)
