# **SquadSync Developer Guide**

This guide accompanies the `technical_specification.md` and provides concrete code examples and patterns to be used during development.

---

## **1. TypeScript Patterns (Frontend & Signaling)**

### **1.1 Validation with Zod**

We use Zod for runtime validation of all external data (User input, WebSocket messages, Config files).

```typescript
import { z } from "zod";

// Define the schema
const ClipRequestSchema = z.object({
  clipId: z.string().uuid(),
  timestamp: z.number().positive(),
  segmentCount: z.number().min(10).max(300),
});

type ClipRequest = z.infer<typeof ClipRequestSchema>;

async function handleClip(request: unknown) {
  // Parse throws if invalid
  const validated = ClipRequestSchema.parse(request);
  await processClip(validated);
}
```

### **1.2 Result Pattern for Async Operations**

Avoid throwing errors for expected failure modes (e.g., network errors). Use a Result type.

```typescript
type Result<T, E = Error> = 
  | { ok: true; value: T }
  | { ok: false; error: E };

async function uploadClip(url: string, data: Blob): Promise<Result<string>> {
  try {
    const response = await fetch(url, { method: "PUT", body: data });
    if (!response.ok) {
      return { ok: false, error: new Error(`HTTP ${response.status}`) };
    }
    return { ok: true, value: url };
  } catch (e) {
    return { ok: false, error: e as Error };
  }
}
```

---
 
 ## **1.3 Tauri v2 Plugins (Frontend)**
 
 In Tauri v2, core APIs are moved to plugins.
 
 ```typescript
 // OLD (v1)
 // import { invoke } from "@tauri-apps/api/tauri";
 // import { appDataDir } from "@tauri-apps/api/path";
 
 // NEW (v2)
 import { invoke } from "@tauri-apps/api/core";
 import { appDataDir } from "@tauri-apps/api/path"; // Now requires @tauri-apps/plugin-path
 import { Command } from "@tauri-apps/plugin-shell";
 
 async function startFfmpeg() {
   // Requires `shell` plugin and permissions
   const command = Command.sidecar("ffmpeg", ["-i", ...]);
   const child = await command.spawn();
 }
 ```
 
 ---

## **2. Rust Patterns (Backend)**

### **2.1 Error Handling**

Use `thiserror` for library/module errors.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClipError {
    #[error("FFmpeg process failed: {0}")]
    FfmpegFailed(String),
    
    #[error("Buffer segment not found: {0}")]
    SegmentNotFound(String),
    
    #[error("Upload failed: {0}")]
    UploadFailed(#[from] reqwest::Error),
}

pub type ClipResult<T> = Result<T, ClipError>;
```

### **2.2 Builder Pattern for FFmpeg Commands**

Use the Builder pattern to construct complex FFmpeg commands safely.

```rust
pub struct FfmpegCommand {
    input: String,
    encoder: Encoder,
    bitrate: u32,
    output: String,
}

impl FfmpegCommand {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            encoder: Encoder::default(),
            bitrate: 6_000_000,
            output: String::new(),
        }
    }
    
    pub fn encoder(mut self, encoder: Encoder) -> Self {
        self.encoder = encoder;
        self
    }
    
    pub fn bitrate(mut self, bitrate: u32) -> Self {
        self.bitrate = bitrate;
        self
    }
    
    pub fn build(self) -> Vec<String> {
        // Logic to construct the argument list
        vec![
            "-i".to_string(), self.input,
            "-b:v".to_string(), format!("{}k", self.bitrate / 1000),
            // ...
        ]
    }
}
```

### **2.3 Tauri Commands**

Tauri commands should return `Result` to properly propagate errors to the frontend.

```rust
#[tauri::command]
async fn create_clip(
    state: State<'_, AppState>,
    segment_count: u32,
) -> ClipResult<ClipMetadata> {
    let segments = state.buffer.get_last_n_segments(segment_count)?;
    let output = state.buffer.concat_segments(&segments)?;
    
    Ok(ClipMetadata {
        path: output,
        duration: segment_count as f64,
        created_at: chrono::Utc::now(),
    })
}
```

```
 
 ### **2.4 Capabilities & Permissions (Tauri v2)**
 
 Permissions are defined in `src-tauri/capabilities/default.json`.
 
 ```json
 {
   "identifier": "default",
   "description": "Default permissions for the app",
   "local": true,
   "windows": ["main"],
   "permissions": [
     "core:default",
     "core:shell:allow-execute",
     "core:fs:allow-read-text-file",
     "core:fs:allow-write-file"
   ]
 }
 ```
 
 ---

## **3. React Component Patterns**

### **3.1 Component Structure**

*   **Imports**: External first, then internal.
*   **Interface**: Props definition.
*   **Component**: Hooks -> Early Returns -> Render.

### **3.2 Sync Logic Example (VideoGrid)**

This example demonstrates the "Master/Follower" sync logic required for the multi-POV player.

```tsx
interface VideoGridProps {
  clips: Clip[];
  activeIndex: number;
  onActiveChange: (index: number) => void;
}

export function VideoGrid({ clips, activeIndex, onActiveChange }: VideoGridProps) {
  const videoRefs = useRef<(HTMLVideoElement | null)[]>([]);
  
  // Sync loop: Runs at 60fps to keep passive players in sync with active player
  useEffect(() => {
    const interval = setInterval(() => {
      const masterVideo = videoRefs.current[activeIndex];
      if (!masterVideo) return;
      
      const masterTime = masterVideo.currentTime;
      
      videoRefs.current.forEach((video, i) => {
        if (video && i !== activeIndex) {
          const drift = Math.abs(video.currentTime - masterTime);
          // Only correct if drift is noticeable (>100ms) to avoid audio stutter
          if (drift > 0.1) {
            video.currentTime = masterTime;
          }
        }
      });
    }, 16);
    
    return () => clearInterval(interval);
  }, [activeIndex]);
  
  return (
    <div className="grid grid-cols-2 gap-2">
      {clips.map((clip, index) => (
        <VideoPlayer
          key={clip.id}
          ref={(el) => (videoRefs.current[index] = el)}
          src={clip.url}
          isActive={index === activeIndex}
          onClick={() => onActiveChange(index)}
        />
      ))}
    </div>
  );
}

---

## **4. CI/CD Workflow**

### **4.1 Local CI**

To save GitHub Actions minutes and iterate quickly, run the full CI suite locally before pushing. This command runs `build`, `lint`, and `typecheck` in parallel.

```bash
pnpm run ci
```

### **4.2 GitHub Actions**

We use GitHub Actions for a final verification in the cloud. To conserve minutes, **automatic triggers (push/PR) are disabled**.

**To run CI on GitHub:**
1.  Go to the **Actions** tab in the repository.
2.  Select the **CI** workflow.
3.  Click **Run workflow**.
4.  Select the branch (usually `main`) and click **Run workflow**.
```
