# **Implementation Plan: Explicit Timestamp Synchronization**

**Goal:** Achieve frame-accurate playback synchronization for "Flux Replay" by transitioning from implicit start times to explicit UTC timestamping.

## **Phase 1: Backend Architecture (The Anchor)**

Focus: apps/desktop/src-tauri  
Objective: Accurate extraction of the absolute UTC start time for generated clips.

### **1.1 Timestamp Extraction Logic**

* **Action:** Implement filename parsing logic in replay.rs.  
* **Requirement:**  
  * Target the *first segment* used in the concatenation list.  
  * Parse the last 14 digits of the filename strictly as %Y%m%d%H%M%S.  
  * Convert to UTC Epoch Milliseconds.  
  * *Failure Mode:* If parsing fails, return None (do not crash).

### **1.2 Offset Calculation**

* **Action:** Calculate the final start\_time\_utc\_ms.  
* **Formula:** FirstSegmentEpochMs \+ ActualTrimOffsetMs  
  * *Note:* ActualTrimOffsetMs must be derived from media timestamps (ffprobe/PTS), not just the FFmpeg argument.

### **1.3 API Contract Update**

* **Action:** Update the SavedReplay return struct.  
* **Definition:**  
  pub struct SavedReplay {  
      pub file\_path: String,  
      pub duration\_ms: u64,  
      pub start\_time\_utc\_ms: Option\<u64\>, // New field  
      pub version: u32,  
  }

## **Phase 2: Desktop Client Integration (The Transport)**

Focus: apps/desktop/src/components/room  
Objective: Securely transport the extracted metadata to the signaling server.

### **2.1 Invoke Handler Update**

* **Action:** Update RoomManager.tsx to handle the new SavedReplay response format from the Rust backend.  
* **Logic:** Ensure start\_time\_utc\_ms is captured. If None, default to null to signal an "unsynced" state.

### **2.2 Payload Modification**

* **Action:** Modify the UPLOAD\_COMPLETE socket message payload.  
* **New Payload Structure:**  
  {  
    type: 'UPLOAD\_COMPLETE',  
    clipId: string,  
    userId: string,  
    videoUrl: string,  
    videoStartTimeMs: number | null, // The new field  
    durationMs: number  
  }

## **Phase 3: Signaling Server (The Storage)**

Focus: apps/signaling/src  
Objective: Persist synchronization metadata with validation.

### **3.1 Type Definition Updates**

* **Action:** Update View type to include videoStartTimeMs: number | null.

### **3.2 Input Validation**

* **Action:** Add validation middleware for UPLOAD\_COMPLETE.  
* **Rule:** If videoStartTimeMs is provided, it must be a valid epoch timestamp (e.g., \> 1,600,000,000,000). Treat invalid/impossible dates as null.

### **3.3 Persistence**

* **Action:** Ensure the new field is saved to the Room's persistent state object so it survives page reloads/rejoins.

## **Phase 4: Web Client Playback Engine (The Synchronization)**

Focus: apps/web/src/components/WebSquadGrid.tsx  
Objective: Frame-accurate playback using a "Global Timeline" model.

### **4.1 Global Anchor Calculation**

* **Action:** Determine the timelineStartMs (Global Zero).  
* **Logic:**  
  * Filter for views where videoStartTimeMs \!== null.  
  * timelineStartMs \= Min(synced\_start\_times).  
  * *Fallback:* If no synced views exist, default to 0 (legacy behavior).

### **4.2 Offset Computation**

* **Action:** Calculate local offsets for each clip.  
* **Formula:** offsetMs \= (view.videoStartTimeMs \- timelineStartMs) (treat nulls as 0).

### **4.3 The Sync Loop (Implementation Core)**

* **Action:** Implement the active sync correction loop (using requestAnimationFrame or timeupdate).  
* **Correction Logic:**  
  * **Diff \> 200ms:** Hard Seek (currentTime \= target).  
  * **Diff 50ms \- 200ms:** Rate Tweak (±3% playback speed).  
  * **Diff \< 50ms:** No action (Target 1.0x speed).  
* **Clamping:** Ensure players pause if the global time is outside their specific start/end bounds.

### **4.4 Scrubbing UX**

* **Action:** Implement "Pause-Scrub-Resume" logic.  
* **Logic:** Dragging the scrubber calculates a GlobalTime, which translates to specific currentTime values for every video based on their individual offsets.

## **Phase 5: Verification & QA Strategy**

**Objective:** Verify sync accuracy under real-world conditions.

### **5.1 The "Clap" Test (End-to-End)**

* **Setup:** Multiple users clap simultaneously on camera.  
* **Verify:** In playback, the audio spike and visual contact of hands happen at the exact same moment across all viewports.

### **5.2 Chaos Engineering (Artificial Delay)**

* **Setup:** Inject a 500ms thread::sleep in the Rust backend *before* file generation to simulate processing lag.  
* **Verify:** The system correctly identifies the later start\_time\_utc\_ms and the playback engine delays that specific video by 500ms relative to others.

### **5.3 Legacy Compatibility**

* **Verify:** Test with one client on the new version and one on the old version. The system should degrade gracefully (old client marked as "Unsynced" but still playable).