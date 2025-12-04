# Sync Migration Implementation Plan

## Goal Description

Implement a comprehensive synchronization migration to ensure frame-accurate playback across all clients. This plan addresses timebase confusion, filename parsing fragility, and frontend sync jitter, moving from implicit start times to explicit UTC timestamping.

## User Review Required

> [!WARNING]
> **No Backwards Compatibility**: We are strictly enforcing the presence of `videoStartTimeMs` (Epoch time). Legacy clients or clips without valid timestamps will be treated as invalid/error states. There is no fallback for "unsynced" legacy views.

> [!IMPORTANT]
> **Fail Hard Policy**: If the backend cannot determine a precise `start_time_utc_ms` from the filename or `ffprobe` (validated against Epoch), the upload process will be aborted. This guarantees that all uploaded content is syncable.

## Proposed Changes

### Phase 1: Backend Architecture (The Anchor)

**Focus**: `apps/desktop/src-tauri`
**Objective**: Accurate extraction of the absolute UTC start time for generated clips.

#### 1.1 Timestamp Extraction Logic

- **Action**: Create `parse_segment_filename_to_epoch_ms(filename: &str) -> Result<u64>` in `src/ffmpeg/utils.rs`.
- **Requirement**:
  - Target the _first segment_ used in the concatenation list.
  - Parse the last 14 digits of the filename strictly as `%Y%m%d%H%M%S`.
  - Convert to UTC Epoch Milliseconds.
  - **Fail Hard**: If parsing fails, return specific error (do not fallback to relative time).

#### 1.2 Offset Calculation & Validation

- **Action**: Refactor `probe_start_time` in `src/commands/replay.rs`.
- **Logic**:
  - Use `parse_segment_filename_to_epoch_ms` as the primary source.
  - Validate `ffprobe` output: Reject relative timestamps (small values) if they don't match the filename's Epoch ballpark.
  - **Formula**: `FirstSegmentEpochMs + ActualTrimOffsetMs`
- **Fail Hard**: If `probe_start_time` returns an error, `save_replay_impl` must abort.

#### 1.3 API Contract Update

- **Action**: Update the `SavedReplay` return struct in `src/commands/replay.rs`.
- **Definition**:
  ```rust
  pub struct SavedReplay {
      pub file_path: String,
      pub duration_ms: u64,
      pub start_time_utc_ms: Option<u64>, // Strictly enforced to be Some() on success
      pub version: u32,
  }
  ```

### Phase 2: Desktop Client Integration (The Transport)

**Focus**: `apps/desktop`
**Objective**: Securely transport the extracted metadata to the signaling server.

#### 2.1 Invoke Handler Update

- **Action**: Update `useRecorder.ts` (and `RoomManager` if applicable) to handle the new `SavedReplay` response.
- **Logic**: Ensure `start_time_utc_ms` is captured. If missing (which shouldn't happen due to Fail Hard), treat as error.

#### 2.2 Payload Modification

- **Action**: Modify the `UPLOAD_COMPLETE` socket message payload.
- **New Payload Structure**:
  ```typescript
  {
    type: 'UPLOAD_COMPLETE',
    clipId: string,
    userId: string,
    videoUrl: string,
    videoStartTimeMs: number, // Required (was number | null)
    durationMs: number
  }
  ```

### Phase 3: Signaling Server (The Storage)

**Focus**: `apps/signaling/src`
**Objective**: Persist synchronization metadata with validation.

#### 3.1 Type Definition Updates

- **Action**: Update `View` type in `@squadsync/shared` (and signaling) to include `videoStartTimeMs: number`.

#### 3.2 Input Validation

- **Action**: Add validation middleware for `UPLOAD_COMPLETE`.
- **Rule**: `videoStartTimeMs` must be a valid epoch timestamp (> 1,600,000,000,000). Reject invalid/impossible dates.

### Phase 4: Web Client Playback Engine (The Synchronization)

**Focus**: `apps/web/src/components/WebSquadGrid.tsx`
**Objective**: Frame-accurate playback using a "Global Timeline" model.

#### 4.1 Global Anchor Calculation

- **Action**: Implement `computeTimelineStartMs` in `packages/shared`.
- **Logic**: `timelineStartMs = Min(all_synced_start_times)`.

#### 4.2 Offset Computation

- **Action**: Implement `computeClipOffsetMs` in `packages/shared`.
- **Formula**: `offsetMs = view.videoStartTimeMs - timelineStartMs`.

#### 4.3 The Sync Loop (Implementation Core)

- **Action**: Implement active sync correction loop in `WebSquadGrid.tsx`.
- **Correction Logic**:
  - **Diff > 200ms**: Hard Seek (`currentTime = target`).
  - **Diff 50ms - 200ms**: Rate Tweak (±3% playback speed).
  - **Diff < 50ms**: No action (Target 1.0x speed).
- **Edge Clips**: Pause/hide players if global time is outside their specific `[0, duration]` range.

#### 4.4 Scrubbing UX

- **Action**: Implement "Pause-Scrub-Resume" logic.
- **Logic**: Dragging the scrubber calculates a GlobalTime, which translates to specific `currentTime` values for every video. Sync loop is paused during scrub.

### Phase 5: Verification & QA Strategy

**Objective**: Verify sync accuracy under real-world conditions.

#### 5.1 The "Clap" Test (End-to-End)

- **Setup**: Multiple users clap simultaneously on camera.
- **Verify**: In playback, the audio spike and visual contact of hands happen at the exact same moment across all viewports.

#### 5.2 Chaos Engineering (Artificial Delay)

- **Setup**: Inject a 500ms `thread::sleep` in the Rust backend _before_ file generation to simulate processing lag.
- **Verify**: The system correctly identifies the later `start_time_utc_ms` and the playback engine delays that specific video by 500ms relative to others.

#### 5.3 Fail Hard Verification

- **Setup**: Rename a temp file to have an invalid timestamp format.
- **Verify**: The upload process aborts and reports a clear error ("Unable to sync clip").
