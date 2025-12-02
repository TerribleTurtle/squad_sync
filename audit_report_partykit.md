# PartyKit & Networking Audit Report

## Findings

### 1. Zombie Members (Critical)

**Issue**: The `connToUser` map, which maps connection IDs to User IDs, is stored in-memory in `apps/signaling/src/server.ts`.

- If the PartyKit server process restarts (e.g., due to deployment or cold start), this in-memory map is lost.
- When a client subsequently disconnects, `onClose` is called.
- The server attempts to look up the `userId` using `this.connToUser.get(conn.id)`, which returns `undefined`.
- Consequently, `handleLeave` is skipped, and the user remains in the persistent `room.storage` indefinitely as a "zombie" member.

**Impact**:

- Users may appear online in the squad list even when they are offline.
- Re-joining might create duplicate entries or inconsistent state if not handled perfectly.

**Recommendation**:

1.  **Persist Connection Mapping**: Store the `connectionId -> userId` mapping in `room.storage` (e.g., key `conn:${conn.id}`). This ensures the mapping survives server restarts.
2.  **Clear State on Start**: Implement `onStart` to clear all `member:*` and `conn:*` keys from storage. This ensures a clean slate when the room initializes.

### 2. Race Conditions in `handleJoin` (Minor)

**Issue**: `handleJoin` performs a `room.storage.put` and then broadcasts. If a user joins from multiple tabs or devices simultaneously, there's a slight race condition, but `PartyKit` handles requests sequentially per room, so this is likely mitigated by the single-threaded nature of the room actor.

### 3. Rate Limiting (Good)

**Observation**: The `RateLimiter` implementation is in-memory and cleans up every 60 seconds. This is appropriate for the current scale.

### 4. Client-Side Logic (Good)

**Observation**: `useRoom` correctly handles connection states and message dispatching. The use of `ref` for `onClipStart` prevents stale closures.

## Implementation Plan

1.  **Modify `apps/signaling/src/server.ts`**:
    - Add `onStart` method to clear all storage.
    - Update `onMessage` (JOIN_ROOM) to store `conn:${sender.id}` -> `userId` in storage.
    - Update `onClose` to retrieve `userId` from storage if not found in memory.
2.  **Modify `apps/signaling/src/handlers/room.ts`**:
    - Update `handleLeave` to clean up the `conn:${conn.id}` key.
