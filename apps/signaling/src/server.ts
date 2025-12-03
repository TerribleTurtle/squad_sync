import type * as Party from 'partykit/server';
import { RoomHandler } from './handlers/room';
import { SyncHandler } from './handlers/sync';
import { ClientMessageSchema, ClientMessage } from '@squadsync/shared';
import { S3Client, PutObjectCommand } from '@aws-sdk/client-s3';
import { getSignedUrl } from '@aws-sdk/s3-request-presigner';

import { RateLimiter } from './lib/ratelimit';

export default class Server implements Party.Server {
  private roomHandler: RoomHandler;
  private syncHandler: SyncHandler;
  private rateLimiter: RateLimiter;
  private s3Client: S3Client | null = null;

  // Map connection ID to User ID for disconnect handling
  private connToUser: Map<string, string> = new Map();

  // Map User ID to timeout timer for grace period
  private pendingDisconnects: Map<string, ReturnType<typeof setTimeout>> = new Map();

  constructor(readonly room: Party.Room) {
    this.roomHandler = new RoomHandler(room);
    this.syncHandler = new SyncHandler();
    this.rateLimiter = new RateLimiter();
  }

  // Helper to get env vars safely (supports dev .env and prod secrets)
  private getEnv(key: string): string | undefined {
    return (this.room.env[key] as string) || (process.env[key] as string);
  }

  private getS3Client(): S3Client | null {
    if (this.s3Client) return this.s3Client;

    const accessKeyId = this.getEnv('R2_ACCESS_KEY_ID');
    const secretAccessKey = this.getEnv('R2_SECRET_ACCESS_KEY');
    const accountId = this.getEnv('R2_ACCOUNT_ID');

    if (!accessKeyId || !secretAccessKey || !accountId) {
      console.error('❌ Missing R2 Environment Variables');
      return null;
    }

    this.s3Client = new S3Client({
      region: 'auto',
      endpoint: `https://${accountId}.r2.cloudflarestorage.com`,
      credentials: {
        accessKeyId,
        secretAccessKey,
      },
    });
    return this.s3Client;
  }

  async onStart() {
    // Clear all storage on startup to prevent zombie members from previous runs
    // This ensures a clean state if the server restarts
    const stored = await this.room.storage.list();
    for (const key of stored.keys()) {
      await this.room.storage.delete(key);
    }
    console.log(`Cleared ${stored.size} items from storage on start`);
  }

  async onConnect(conn: Party.Connection, ctx: Party.ConnectionContext) {
    console.log(`Connected: ${conn.id} to room ${this.room.id}`);
  }

  async onMessage(message: string, sender: Party.Connection) {
    try {
      const parsed = JSON.parse(message);
      const result = ClientMessageSchema.safeParse(parsed);

      if (!result.success) {
        console.error('Invalid message:', result.error);
        return;
      }

      const msg = result.data as ClientMessage;

      // Check Rate Limit
      if (!this.rateLimiter.checkLimit(sender.id, msg.type as any)) {
        console.warn(`Rate limit exceeded for ${sender.id} on ${msg.type}`);
        sender.send(
          JSON.stringify({
            type: 'ERROR',
            code: 'RATE_LIMITED',
            message: 'You are sending too many requests. Please slow down.',
          })
        );
        return;
      }

      switch (msg.type) {
        case 'JOIN_ROOM':
          // Cancel any pending disconnect for this user
          const pendingTimeout = this.pendingDisconnects.get(msg.userId);
          if (pendingTimeout) {
            console.log(`Cancelling pending disconnect for ${msg.userId}`);
            clearTimeout(pendingTimeout);
            this.pendingDisconnects.delete(msg.userId);
          }

          this.connToUser.set(sender.id, msg.userId);
          // Persist connection mapping for recovery after restart
          await this.room.storage.put(`conn:${sender.id}`, msg.userId);
          await this.roomHandler.handleJoin(sender, msg);
          break;
        case 'LEAVE_ROOM':
          const userId =
            this.connToUser.get(sender.id) ||
            (await this.room.storage.get<string>(`conn:${sender.id}`));
          if (userId) {
            await this.roomHandler.handleLeave(sender, userId);
            this.connToUser.delete(sender.id);
            await this.room.storage.delete(`conn:${sender.id}`);

            // Also clear any pending disconnects if they explicitly left
            const timeout = this.pendingDisconnects.get(userId);
            if (timeout) {
              clearTimeout(timeout);
              this.pendingDisconnects.delete(userId);
            }
          }
          break;
        case 'TIME_SYNC_REQUEST':
          this.syncHandler.handleTimeSync(sender, msg);
          break;
        case 'TRIGGER_CLIP':
          const clipId = crypto.randomUUID();
          let uploadUrl = '';

          const s3 = this.getS3Client();
          const bucketName = this.getEnv('R2_BUCKET_NAME');

          if (s3 && bucketName) {
            try {
              const key = `uploads/${this.room.id}/${clipId}.mp4`;
              const command = new PutObjectCommand({
                Bucket: bucketName,
                Key: key,
                ContentType: 'video/mp4',
              });

              // Generate Presigned URL (valid for 15 mins)
              uploadUrl = await getSignedUrl(s3, command, { expiresIn: 900 });
              console.log(`✅ Generated Presigned URL for ${key}`);
            } catch (err) {
              console.error('❌ Failed to generate presigned URL:', err);
              sender.send(
                JSON.stringify({
                  type: 'ERROR',
                  code: 'UPLOAD_ERROR',
                  message: 'Failed to generate upload URL',
                })
              );
            }
          } else {
            console.warn('⚠️ R2 not configured. Skipping upload URL generation.');
          }

          const startClipMsg = {
            type: 'START_CLIP',
            clipId,
            segmentCount: msg.segmentCount || 60, // Default to 60s if not specified
            referenceTime: Date.now(),
            uploadUrl,
          };
          this.room.broadcast(JSON.stringify(startClipMsg));
          console.log(`Clip ${clipId} triggered by ${sender.id}`);
          break;
        case 'UPLOAD_COMPLETE':
          // TODO: Implement upload completion
          console.log('Upload complete from', sender.id);
          break;
      }
    } catch (e) {
      console.error('Error processing message:', e);
    }
  }

  async onClose(conn: Party.Connection) {
    console.log(`Disconnected: ${conn.id}`);
    let userId = this.connToUser.get(conn.id);

    if (!userId) {
      // Try to recover from storage
      userId = await this.room.storage.get<string>(`conn:${conn.id}`);
    }

    if (userId) {
      // Don't leave immediately. Wait 10s to see if they reconnect.
      console.log(`Scheduling disconnect for ${userId} in 10s`);

      // Clear any existing timeout for this user just in case
      const existing = this.pendingDisconnects.get(userId);
      if (existing) clearTimeout(existing);

      const timeout = setTimeout(async () => {
        if (!userId) return; // Should not happen due to closure, but safe check

        console.log(`Executing delayed disconnect for ${userId}`);
        await this.roomHandler.handleLeave(conn, userId);
        this.connToUser.delete(conn.id);
        await this.room.storage.delete(`conn:${conn.id}`);
        this.pendingDisconnects.delete(userId);
      }, 10000); // 10 seconds grace period

      this.pendingDisconnects.set(userId, timeout);
    }
  }
}

Server satisfies Party.Worker;
