# FluxReplay

**FluxReplay** is a distributed "Silent Recorder" for gaming squads. "Every Angle. One Timeline." It captures high-quality gameplay locally and synchronizes clip generation across your entire squad, ensuring you never miss a moment from any perspective.

- **Desktop App**: [Tauri v2](https://v2.tauri.app/) (Rust + React)
- **Signaling**: [PartyKit](https://partykit.io/) (Serverless WebSockets)
- **Video Engine**: FFmpeg (Sidecar Process)
- **Storage**: Cloudflare R2

## 📂 Project Structure

This project is a monorepo managed with `pnpm` and `turborepo`.

- `apps/desktop`: The main desktop application (Tauri).
- `apps/web`: The web application (Next.js) for homepage and playback.
- `apps/signaling`: The WebSocket signaling server (PartyKit).
- `packages/shared`: Shared TypeScript types, schemas, and constants.
- `packages/ffmpeg-commands`: FFmpeg command builders and utilities.

## 🛠️ Tech Stack

- **Frontend (Web)**: Next.js, React, TailwindCSS
- **Frontend (Desktop)**: React, TailwindCSS, Zustand
- **Backend (Desktop)**: Rust (Tauri v2)
- **Signaling**: PartyKit (Cloudflare Workers)
- **Video Processing**: FFmpeg (Sidecar)
- **Build System**: TurboRepo, pnpm

## 🚦 Getting Started

### Prerequisites

- **Node.js**: v18+
- **Rust**: Stable
- **pnpm**: `npm install -g pnpm`
- **FFmpeg**: Installed and available in PATH (for development).
- **FFmpeg**: Installed and available in PATH (for development).
  - **Note**: The desktop app requires a specific FFmpeg binary. Run `pnpm setup:ffmpeg` in `apps/desktop` to download it.

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

# 2. Build All
pnpm build

# 3. Build Specific App
pnpm --filter desktop tauri build
pnpm --filter web build
```

### Deployment

- **Web App**: Deployed on Vercel (`apps/web`).

4.  Push to the branch (`git push origin feature/amazing-feature`).
5.  Open a Pull Request.

## 📄 License

[ISC](LICENSE)

```

```
