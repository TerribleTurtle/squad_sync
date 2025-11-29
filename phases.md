# SquadSync Project Phases

## **Phase 0: Initialization & Documentation**
**Goal:** Define project scope and initial scaffolding.
- [x] Create project directory
- [x] Create `technical_specification.md`
- [x] Create `developer_guide.md`
- [x] Create `phases.md`
- [x] Initialize basic monorepo config (`pnpm-workspace.yaml`, `turbo.json`, `package.json`)


## **Phase 1: Project Skeleton & Monorepo Setup**
**Goal:** Initialize the repository structure and shared tooling.
- [x] Initialize Git repository
- [x] Create directory structure (`apps/`, `packages/`)
- [x] Setup `pnpm` workspace & `Turborepo`
- [x] Configure root `package.json` (scripts, engines)
- [x] Configure `tsconfig.json`, `eslintrc`, `prettier`
- [x] Setup IDE (`.editorconfig`, `.vscode/`)
- [x] Setup Basic CI (Lint, Build, Typecheck) - *Manual Trigger Only*


## **Phase 2: Core Video Tech (Proof of Concept)**
**Goal:** Prove the "Silent Recorder" architecture works (FFmpeg + Rust + Disk).
**Priority:** CRITICAL
- [x] Initialize `apps/desktop` (Tauri v2)
- [x] Implement Rust FFmpeg sidecar management
- [x] Implement basic Ring Buffer (write to disk)
- [x] Verify hardware acceleration detection (NVENC/AMF/QSV)
- [x] **Deliverable:** A running Tauri app that records the screen to a temp folder.
- [x] Implement Configuration Management (config.toml)
- [x] **High Performance Capture** (ddagrab + Zero-Copy)
- [x] **Manual Recording Mode** (MP4 Output)
- [ ] Implement Buffer Cleanup Logic
- [ ] Implement Basic Error Handling (Recording/FFmpeg)

## **Phase 3: Shared Infrastructure**
**Goal:** Set up the data types and contracts for the full app.
- [ ] `packages/shared`: Types, Zod Schemas, Constants
- [ ] `packages/ffmpeg-commands`: Command builders
- [ ] Setup Test Infrastructure (Vitest & Cargo)

## **Phase 4: Signaling & Synchronization**
**Goal:** Enable multi-user connectivity.
- [ ] Initialize `apps/signaling` (PartyKit)
- [ ] Implement Room Logic (Join/Leave)
- [ ] Implement NTP Time Sync
- [ ] Implement Signaling Error Handling

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
