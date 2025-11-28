# SquadSync Project Phases

## **Phase 1: Project Skeleton & Monorepo Setup**
**Goal:** Initialize the repository structure and shared tooling.
- [ ] Initialize Git repository
- [ ] Setup `pnpm` workspace & `Turborepo`
- [ ] Create directory structure (`apps/`, `packages/`)
- [ ] Configure root `package.json`, `tsconfig.json`, `eslintrc`

## **Phase 2: Core Video Tech (Proof of Concept)**
**Goal:** Prove the "Silent Recorder" architecture works (FFmpeg + Rust + Disk).
**Priority:** CRITICAL
- [ ] Initialize `apps/desktop` (Tauri v2)
- [ ] Implement Rust FFmpeg sidecar management
- [ ] Implement basic Ring Buffer (write to disk)
- [ ] Verify hardware acceleration detection (NVENC/AMF/QSV)
- [ ] **Deliverable:** A running Tauri app that records the screen to a temp folder.

## **Phase 3: Shared Infrastructure**
**Goal:** Set up the data types and contracts for the full app.
- [ ] `packages/shared`: Types, Zod Schemas, Constants
- [ ] `packages/ffmpeg-commands`: Command builders

## **Phase 4: Signaling & Synchronization**
**Goal:** Enable multi-user connectivity.
- [ ] Initialize `apps/signaling` (PartyKit)
- [ ] Implement Room Logic (Join/Leave)
- [ ] Implement NTP Time Sync

## **Phase 5: Desktop Frontend & Integration**
**Goal:** Connect the UI to the backend and signaling server.
- [ ] React UI (Overlay, Grid)
- [ ] Connect `useRecorder` hook to Rust backend
- [ ] Connect `useRoom` hook to PartyKit

## **Phase 6: Cloud Upload & Playback**
**Goal:** Complete the loop.
- [ ] Implement Clip Generation (FFmpeg concat)
- [ ] Implement R2 Upload (Presigned URLs)
- [ ] Implement Playback Grid with Sync Logic
