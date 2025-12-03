# FluxReplay Project Phases

## ✅ Completed Milestones

### **Phase 0: Initialization & Documentation**

**Goal:** Define project scope and initial scaffolding.

- [x] Create project directory
- [x] Create `technical_specification.md`
- [x] Create `developer_guide.md`
- [x] Create `phases.md`
- [x] Initialize basic monorepo config (`pnpm-workspace.yaml`, `turbo.json`, `package.json`)

### **Phase 1: Project Skeleton & Monorepo Setup**

**Goal:** Initialize the repository structure and shared tooling.

- [x] Initialize Git repository
- [x] Create directory structure (`apps/`, `packages/`)
- [x] Setup `pnpm` workspace & `Turborepo`
- [x] Configure root `package.json` (scripts, engines)
- [x] Configure `tsconfig.json`, `eslintrc`, `prettier`
- [x] Setup IDE (`.editorconfig`, `.vscode/`)
- [x] Setup Basic CI (Lint, Build, Typecheck) - _Manual Trigger Only_

### **Phase 2: Core Video Tech (Proof of Concept)**

**Goal:** Prove the "Silent Recorder" architecture works (FFmpeg + Rust + Disk).
**Priority:** CRITICAL

- [x] Initialize `apps/desktop` (Tauri v2)
- [x] Implement Rust FFmpeg sidecar management
- [x] Implement basic Ring Buffer (write to disk)
- [x] Verify hardware acceleration detection (NVENC/AMF/QSV)
- [x] **Deliverable:** A running Tauri app that records the screen to a temp folder.
- [x] Implement Configuration Management (config.toml) & Advanced Overrides
- [x] **High Performance Capture** (ddagrab + Zero-Copy)
- [x] **Manual Recording Mode** (MP4 Output)
- [x] **Circular Buffer Mode** (Replay Buffer Engine) - _Fragmented MP4_
- [x] **Buffer Cleanup Logic** (Startup Cleanup)
- [x] **Game/System Audio Capture** (via Config)
- [x] **Screen Selector** (Multi-Monitor Support)
- [x] **Dynamic Bitrate & Smart Resolution**
- [x] **Global Hotkey** (Alt+F10)

### **Phase 3: MVP Robustness & Polish**

**Goal:** Ensure the app is stable, safe, and performant enough for public release.
**Priority:** HIGH

- [ ] **Disk Space Watchdog** (Auto-disable if low disk space) - _Deferred_
- [ ] **Structured Error Handling** (AppError enum, User-friendly messages) - _Deferred_
- [x] **Process Priority Management** (Prevent game lag)
- [x] **Zombie Process Prevention** (Windows Job Objects)
- [x] **Verify Audio Sync** (Long duration test)
- [x] **Migrate to MKV/PCM** (Intermediate Recording Format)
- [x] **Decoupled Audio/Video Pipeline**
- [ ] **Codebase Audit Fixes**
  - [x] **Critical**: Fix time conversion panics in `replay.rs` (DST safety)
  - [x] **Critical**: Fix blocking `std::thread::sleep` in async runtime
  - [x] **Cleanup**: Replace unsafe `unwrap()` with proper error handling
  - [x] **Cleanup**: Fix hardcoded audio settings in `process.rs`
  - [x] **Cleanup**: Improve Windows path handling in `stitch_segments`
  - [x] **Cleanup**: Remove stray test artifacts from root
  - [x] **Cleanup**: Fix all `cargo clippy` warnings

### **Phase 4: Frame-Perfect Sync & Replay Architecture**

**Goal:** Achieve frame-perfect synchronization of clips across multiple users by using an authoritative time source (NTP) and absolute time-based file naming.

- [x] **NTP Synchronization**
  - [x] Implement `rsntp` client with periodic sync (15m interval)
  - [x] Implement local fallback (Offset=0) for offline mode
  - [x] Store authoritative `NtpOffset` in app state
- [x] **Time-Based Capture Architecture**
  - [x] Update FFmpeg to use `%Y%m%d%H%M%S` file naming
  - [x] Ensure `use_wallclock_as_timestamps` is active
  - [x] Switch to 2s segments for granular control
- [x] **Frame-Perfect Replay Logic**
  - [x] Implement `find_segments_by_time` (NTP -> Local Time conversion)
  - [x] Implement `probe_start_time` (ffprobe) for sub-second precision
  - [x] Implement precise trimming (`TriggerTime - Duration - ActualStart`)
- [x] **Verification**
  - [x] Verify NTP offset accuracy
  - [x] Verify clip start precision (Stopwatch test)

---

## 🚧 Active Development

### **Phase 5: Signaling & Connectivity**

**Goal:** Enable multi-user connectivity via PartyKit and connect the frontend.

- [x] **Infrastructure**
  - [x] Initialize `apps/signaling` (PartyKit)
  - [x] Define shared types in `packages/shared`
  - [x] Setup Test Infrastructure (Vitest & Cargo)
  - [x] **Refactor**: Extract `useRecorder` and stores from `App.tsx` (Pre-req for Signaling)
- [ ] **Implementation**
  - [x] Implement Room Logic (Join/Leave) in PartyKit
  - [x] Implement Client WebSocket Hook (`useRoom`)
  - [x] Implement Signaling Error Handling
- [ ] **Verification**
  - [x] Verify Room Connection (Multi-Client)
  - [x] Verify Error States (Disconnect/Reconnect)

### **Phase 5.5: Rebranding to FluxReplay**

**Goal:** Establish final product identity and assets.

- [x] **Identity**
  - [x] Rename project to **FluxReplay**
  - [x] Create "Time Stack" Logo & Design System
  - [x] Generate App Icons (.ico, .icns, .png)
- [x] **Codebase Updates**
  - [x] Update `tauri.conf.json` (Identifier/Title)
  - [x] Update `package.json` & Documentation
  - [x] Verify Build Health

---

## 📅 Upcoming Milestones

### **Phase 6: Playback & Cloud**

**Goal:** Complete the loop with local review and cloud sharing.

- [x] **Local Playback**
  - [x] Implement `LocalPlaybackView` (Watch/Manage local clips)
  - [x] Implement basic clip management (Delete/Rename)
  - [x] Implement video thumbnails
- [ ] **Cloud & Web Playback**
  - [x] Implement `concat_segments` FFmpeg command
  - [x] Implement R2 Upload (Presigned URLs)
  - [ ] Implement `WebSquadGrid` (Canvas/Video)
  - [ ] Implement Leader/Follower Seek Sync
  - [ ] Verify frame-perfect playback synchronization

### **Phase 7: MVP Production Infrastructure**

**Goal:** Setup production environment for PartyKit and R2.

- [ ] **Vercel CLI & Setup**
  - [ ] Install Vercel CLI
  - [ ] Configure Vercel Project
- [ ] **PartyKit Production Setup**
  - [ ] Create PartyKit Project
  - [ ] Configure `partykit.json` for Production
  - [ ] **Environment Variables**
    - [ ] Add R2 Credentials to PartyKit Secrets (`partykit env add`)
    - [ ] Configure CORS for Production

## 🔮 Future Roadmap (Post-MVP)

**Goal:** Optimizations and architectural improvements for scale.

- [ ] **Core Logic Extraction**: Move engine to `squad_sync_engine` crate.
- [ ] **Custom Tauri Plugins**: Wrap recorder as a plugin.
- [ ] **Zero-Copy IPC**: Shared memory for preview.
- [ ] **Hardware Benchmarking**: Auto-detect capability.
- [ ] **Sidecar Watchdog**: External process monitor.
- [ ] **Integration Tests**: CI/CD pipeline for recording.
- [ ] **Auto-Clip Plugin Support**: Game event detection hooks.
- [ ] **Event Timeline**: Visual markers for key events on the playback scrubber.
- [ ] **Squad Link**: Deep linking for easy sharing of rooms and clips.
- [ ] **Match Summary**: Session history and basic stats.
- [ ] **Security Hardening**:
  - [ ] Implement strict Content Security Policy (CSP).
  - [ ] Implement strict Content Security Policy (CSP).
  - [ ] Restrict `ffmpeg` execution arguments or use a sidecar wrapper.
- [ ] **Built-in Video Player**: Custom player with scrubbing, volume control, and trimming.
