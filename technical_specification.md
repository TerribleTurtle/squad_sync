# **Technical Specification: SquadSync**

## **1. Executive Overview**

SquadSync is a desktop application designed for gaming squads to capture, sync, and review gameplay highlights from multiple perspectives (POVs). Unlike traditional streaming platforms, SquadSync utilizes a **"Silent Recorder" architecture**. It records high-quality video locally on each user's machine and only uploads video segments to the cloud when a "Clip" trigger is activated. This ensures zero network latency impact during gameplay and minimizes cloud storage costs.

The system follows a **Distributed Peer-to-Cloud** model coordinated by a lightweight signaling server. The goal is to ship a focused, maintainable product that can scale to 1M+ users.

---

## **2. System Architecture**

### **2.1 High-Level Architecture**

*   **Client:** Tauri v2 (Rust + React) acting as a "Black Box" flight recorder.
*   **Signaling:** PartyKit (Serverless WebSockets) for room management, time synchronization, and upload coordination.
*   **Storage:** Cloudflare R2 (S3-compatible) for ephemeral video storage.
*   **Video Engine:** FFmpeg (Sidecar Process) handling hardware-accelerated capture and ring-buffering.

### **2.2 Technology Stack**

| Component | Technology | Reasoning |
| :---- | :---- | :---- |
| **App Framework** | **Tauri v2** | Native performance, tiny bundle size, secure access to OS binaries. |
| **Frontend** | **React + Tailwind** | Rapid UI development, responsive overlay capabilities. |
| **Backend Logic** | **Rust** | Handles file I/O, process management, and FFmpeg coordination. |
| **Video Core** | **FFmpeg (Binary)** | Robust hardware encoding (NVENC/AMF/QSV) and segment muxing. |
| **Signaling** | **PartyKit** | Low-latency WebSockets, stateful serverless for lobby management. |
| **Storage** | **Cloudflare R2** | Zero egress fees; compatible with S3 SDKs. |

| **Storage** | **Cloudflare R2** | Zero egress fees; compatible with S3 SDKs. |

### **2.3 Tauri v2 Specifics**

*   **Plugins:** Core features are now plugins. We will use:
    *   `@tauri-apps/plugin-shell` (FFmpeg spawning)
    *   `@tauri-apps/plugin-fs` (File management)
    *   `@tauri-apps/plugin-http` (Uploads)
    *   `@tauri-apps/plugin-dialog` (File pickers)
*   **Permissions:** Managed via **Capabilities** in `src-tauri/capabilities/`.
    *   `core:shell:allow-execute` for FFmpeg.
    *   `core:fs:allow-read-write` for temp/buffer directories.

---

## **3. MVP Feature Scope**

### **3.1 In Scope (Must Have)**

| Feature | Description | Priority |
|---------|-------------|----------|
| **Rolling Buffer** | 60-second disk-backed ring buffer via FFmpeg | P0 |
| **Manual Clip Trigger** | Hotkey + UI button to save clip | P0 |
| **Squad Rooms** | Join/create rooms via PartyKit | P0 |
| **Synchronized Clipping** | One trigger saves all POVs | P0 |
| **Time Sync** | NTP-style multi-sample sync (±30ms) | P0 |
| **Cloud Upload** | Presigned URL upload to R2 | P0 |
| **Multi-POV Playback** | 4-player grid view, synced playback | P0 |
| **24hr Auto-Delete** | R2 lifecycle policy | P0 |
| **Windows Support** | Windows 10/11, 64-bit | P0 |
| **Hardware Encoding** | NVENC/AMF/QSV detection | P0 |

### **3.2 Post-MVP (V2)**

| Feature | Description | Priority |
|---------|-------------|----------|
| Auto-clip plugins | Game event detection | P1 |
| Custom storage | Self-hosted S3-compatible | P1 |
| Extended buffer | 2-5 minute options | P1 |
| Clip length selection | 30s/60s/2m/5m at clip time | P1 |
| Upload progress | Progress bar + retry logic | P1 |

### **3.3 Future (V3+)**

| Feature | Description | Priority |
|---------|-------------|----------|
| Custom signaling server | Docker image for self-hosters | P2 |
| macOS/Linux support | Cross-platform capture | P2 |
| LiveKit streaming | Optional live spectate | P2 |
| ML auto-detection | Highlight detection | P3 |
| Mobile companion | View clips on phone | P3 |

### **3.4 Explicitly Out of Scope**

| Feature | Reason |
|---------|--------|
| Full VOD recording | Cost killer (SquadOV lesson) |
| Permanent storage | Scope creep, cost |
| Game stats overlay | Requires per-game maintenance |
| Social features | Focus on core utility first |
| Code signing | Cost prohibitive for indie |

---

## **4. Project Structure**

### **4.1 Monorepo Layout**

```
squadsync/
├── apps/
│   ├── desktop/                 # Tauri app
│   │   ├── src/                 # React frontend
│   │   ├── src-tauri/           # Rust backend
│   │   │   ├── capabilities/    # Tauri v2 Permissions
│   │   │   ├── src/
│   │   │   └── tauri.conf.json
│   └── signaling/               # PartyKit server
│       ├── src/
│       │   └── server.ts
│       └── package.json
│
├── packages/
│   ├── shared/                  # Shared types & constants
│   │   ├── src/
│   │   │   ├── types.ts         # WebSocket payloads, config
│   │   │   ├── constants.ts     # Limits, defaults
│   │   │   └── index.ts
│   │   └── package.json
│   └── ffmpeg-commands/         # FFmpeg command builders
│       ├── src/
│       │   ├── capture.ts
│       │   ├── concat.ts
│       │   └── index.ts
│       └── package.json
│
├── plugins/                     # Game plugins (V2)
│   ├── core/                    # Plugin SDK
│   │   ├── src/
│   │   │   └── interface.ts
│   │   └── package.json
│   └── valorant/                # Example plugin
│       └── ...
│
├── docs/
│   ├── ARCHITECTURE.md
│   ├── BUILDING.md
│   ├── PLUGINS.md
│   ├── SELF-HOSTING.md
│   └── CONTRIBUTING.md
│
├── .github/
│   ├── workflows/
│   │   ├── ci.yml               # Lint, test, typecheck
│   │   ├── release.yml          # Build + GitHub release
│   │   └── dependabot.yml
│   └── ISSUE_TEMPLATE/
│
├── package.json                 # Workspace root
├── pnpm-workspace.yaml
├── turbo.json                   # Turborepo config
├── README.md
├── LICENSE
└── CHANGELOG.md
```

### **4.2 Why This Structure?**

| Decision | Reasoning |
|----------|-----------|
| **Monorepo** | Shared types, atomic changes, single CI |
| **pnpm workspaces** | Fast, disk-efficient, strict |
| **Turborepo** | Cached builds, parallel tasks |
| **Separate `packages/`** | Reusable logic, testable in isolation |
| **Plugins outside core** | Optional, community-contributed |

---

## **5. Module Architecture**

### **5.1 Desktop App (Tauri)**

```
apps/desktop/
├── src/                         # React Frontend
│   ├── components/
│   │   ├── ui/                  # Generic UI (buttons, modals)
│   │   ├── room/                # Room join/create
│   │   ├── recording/           # REC indicator, clip button
│   │   └── playback/            # Video grid, sync controls
│   ├── hooks/
│   │   ├── useRoom.ts           # PartyKit connection
│   │   ├── useRecorder.ts       # Tauri ↔ FFmpeg bridge
│   │   ├── useTimeSync.ts       # NTP sync logic
│   │   └── useClipUpload.ts     # Upload state management
│   ├── lib/
│   │   ├── tauri.ts             # Tauri invoke wrappers
│   │   ├── partykit.ts          # WebSocket client
│   │   └── storage.ts           # Config persistence
│   ├── stores/                  # Zustand stores
│   │   ├── roomStore.ts
│   │   ├── recordingStore.ts
│   │   └── settingsStore.ts
│   ├── App.tsx
│   └── main.tsx
│
├── src-tauri/                   # Rust Backend
│   ├── capabilities/            # Permission definitions
│   │   └── default.json
│   ├── src/
│   │   ├── main.rs              # Entry point
│   │   ├── commands/            # Tauri commands
│   │   │   ├── mod.rs
│   │   │   ├── recording.rs     # Start/stop buffer
│   │   │   ├── clip.rs          # Create clip from buffer
│   │   │   ├── upload.rs        # Upload to presigned URL
│   │   │   └── system.rs        # GPU detection, paths
│   │   ├── ffmpeg/              # FFmpeg process management
│   │   │   ├── mod.rs
│   │   │   ├── process.rs       # Spawn, monitor, kill
│   │   │   ├── commands.rs      # Command builders
│   │   │   ├── encoder.rs       # GPU encoder detection
│   │   └── buffer/              # Ring buffer management
│   │   │   ├── mod.rs
│   │   │   ├── segments.rs      # Segment file handling
│   │   │   └── cleanup.rs       # Old segment deletion
│   │   ├── config/              # App configuration
│   │   │   ├── mod.rs
│   │   │   └── storage.rs       # Custom storage config
│   │   └── error.rs             # Error types
│   ├── Cargo.toml
│   └── tauri.conf.json
```

### **5.2 Signaling Server (PartyKit)**

```
apps/signaling/
├── src/
│   ├── server.ts                # Main PartyKit server
│   ├── handlers/
│   │   ├── room.ts              # Join, leave, list members
│   │   ├── sync.ts              # Time sync requests
│   │   ├── clip.ts              # Clip trigger, URL generation
│   │   └── upload.ts            # Upload complete tracking
│   ├── lib/
│   │   ├── r2.ts                # R2 client, presigned URLs
│   │   ├── ratelimit.ts         # Per-user rate limiting
│   │   └── validation.ts        # Message validation
│   └── types.ts                 # Server-specific types
├── package.json
└── partykit.json
```

### **5.3 Shared Package**

```
packages/shared/
├── src/
│   ├── types/
│   │   ├── websocket.ts         # All WS message types
│   │   ├── clip.ts              # Clip metadata
│   │   ├── room.ts              # Room state
│   │   └── config.ts            # App config schema
│   ├── constants/
│   │   ├── limits.ts            # Rate limits, max sizes
│   │   ├── defaults.ts          # Default config values
│   │   └── encoding.ts          # FFmpeg presets
│   ├── utils/
│   │   ├── time.ts              # Time sync helpers
│   │   └── validation.ts        # Zod schemas
│   └── index.ts                 # Public exports
└── package.json
```

---

## **6. Video Logic (Rust & FFmpeg)**

### **6.1 Rolling Buffer Strategy**

The Rust backend spawns FFmpeg as a sidecar process.

**FFmpeg Command for Rolling Buffer:**
```bash
ffmpeg \
  -f gdigrab -framerate 60 -i desktop  # Capture (Windows)
  -c:v h264_nvenc -b:v 6M -preset p4   # Hardware Encode (Example: NVIDIA)
  -f segment                           # Muxer
  -segment_time 1                      # 1-second chunks
  -segment_wrap 70                     # Keep 70 chunks (Ring Buffer)
  -reset_timestamps 1
  %app_cache%/buffer/out_%03d.ts       # Output pattern
```

### **6.2 Clip Logic**

1.  Receive `CLIP_REQUEST` with Timestamp T.
2.  Pause buffer writing (optional, or copy files to temp).
3.  Identify the `.ts` segments corresponding to the last 60 seconds.
4.  Generate `concat_list.txt`.
5.  Run FFmpeg Concat (Copy Stream) -> `clip.mp4`.
6.  Upload `clip.mp4` to R2.

---

## **7. Data Protocols**

### **7.1 WebSocket Protocol**

**Message Types (TypeScript Interface):**

```typescript
// packages/shared/src/types/websocket.ts

// ============ Client → Server ============

interface JoinRoomMessage {
  type: "JOIN_ROOM";
  roomId: string;
  userId: string;
  displayName: string;
}

interface LeaveRoomMessage {
  type: "LEAVE_ROOM";
}

interface TimeSyncRequestMessage {
  type: "TIME_SYNC_REQUEST";
  clientTime: number;
}

interface TriggerClipMessage {
  type: "TRIGGER_CLIP";
  segmentCount: number;  // Default 60
}

interface UploadCompleteMessage {
  type: "UPLOAD_COMPLETE";
  clipId: string;
  key: string;
}

// ============ Server → Client ============

interface RoomStateMessage {
  type: "ROOM_STATE";
  members: RoomMember[];
  serverTime: number;
}

interface MemberJoinedMessage {
  type: "MEMBER_JOINED";
  member: RoomMember;
}

interface MemberLeftMessage {
  type: "MEMBER_LEFT";
  userId: string;
}

interface TimeSyncResponseMessage {
  type: "TIME_SYNC_RESPONSE";
  clientTime: number;      // Echo back
  serverReceive: number;   // When server got request
  serverSend: number;      // When server sent response
}

interface StartClipMessage {
  type: "START_CLIP";
  clipId: string;
  segmentCount: number;
  referenceTime: number;
  uploadUrl: string;       // Presigned PUT URL
}

interface ClipReadyMessage {
  type: "CLIP_READY";
  clipId: string;
  userId: string;
  url: string;             // Public URL for playback
}

interface AllClipsReadyMessage {
  type: "ALL_CLIPS_READY";
  clipId: string;
  clips: {
    userId: string;
    displayName: string;
    url: string;
  }[];
}

interface ErrorMessage {
  type: "ERROR";
  code: string;
  message: string;
}
```

### **7.2 R2 Bucket Structure**

```
squad-clips/
├── {ClipID}/
│   ├── {UserID_A}.mp4
│   ├── {UserID_B}.mp4
│   ├── {UserID_C}.mp4
│   └── {UserID_D}.mp4
```

### **7.3 Configuration Schema**

```toml
# %APPDATA%/squadsync/config.toml

[user]
display_name = "PlayerOne"
user_id = "uuid-generated-on-first-run"

[recording]
buffer_seconds = 60
resolution = "1080p"
framerate = 60
bitrate_mbps = 6

[recording.encoder]
preference = "auto" # auto, nvenc, amf, qsv, software

[hotkeys]
trigger_clip = "Ctrl+Shift+C"
toggle_recording = "Ctrl+Shift+R"

[storage]
provider = "default" # default, custom

[signaling]
provider = "default" # default, custom
```

---

## **8. Coding Standards**

### **8.1 TypeScript (Frontend + Signaling)**

*   **Explicit Types:** Avoid `any`. Define interfaces for all data structures.
*   **Validation:** Use `Zod` for runtime validation of config and network payloads.
*   **Async/Await:** Prefer over `.then()`.
*   **Result Pattern:** Use `{ ok: true, value: T } | { ok: false, error: E }` for operations that can fail (like uploads).

### **8.2 Rust (Backend)**

*   **Error Handling:** Use `thiserror` for library errors and `anyhow` for application errors.
*   **Type Aliases:** Use `ClipResult<T>` for clarity.
*   **Builder Pattern:** Use for complex configurations (e.g., `FfmpegCommand`).
*   **Tauri Commands:** Return `Result<T, CommandError>` to handle errors gracefully in the frontend.

> [!TIP]
> See `developer_guide.md` for concrete code examples of these patterns, including the Builder pattern for FFmpeg and the Result pattern for async operations.

### **8.3 React Components**

*   **Structure:** One component per file.
*   **Props:** Interface defined above the component.
*   **Hooks:** Custom hooks for logic (e.g., `useTimeSync`, `useRecorder`).
*   **State:** Use Zustand for global stores (Room, Recording, Settings).

---

## **9. Error Handling & Testing**

### **9.1 Error Categories**

*   **Connection:** `CONNECTION_FAILED`, `ROOM_NOT_FOUND`.
*   **Recording:** `FFMPEG_NOT_FOUND`, `ENCODER_NOT_AVAILABLE`, `DISK_FULL`.
*   **Clip/Upload:** `UPLOAD_FAILED`, `RATE_LIMITED`.

### **9.2 Testing Strategy**

| Type | Tool | What to Test |
|------|------|--------------|
| **Unit (TS)** | Vitest | Hooks, utils, validation |
| **Unit (Rust)** | cargo test | FFmpeg commands, buffer logic |
| **Component** | React Testing Library | UI interactions |
| **Integration** | Vitest | WebSocket message flows |
| **E2E** | Playwright (future) | Full clip flow |

---

## **10. Deployment**

1.  **Client:** MSI/EXE installer built via `tauri build`.
2.  **Server:** PartyKit deployed to Cloudflare Workers (Edge).
3.  **Storage:** Cloudflare R2 bucket.
    *   **CORS:** Allow PUT from `tauri://localhost`.
    *   **Lifecycle:** Delete objects older than 1 day.
