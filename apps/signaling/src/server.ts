import type * as Party from 'partykit/server';
import { RoomHandler } from './handlers/room';
import { SyncHandler } from './handlers/sync';
import { ClientMessageSchema, ClientMessage, View, Logger } from '@squadsync/shared';
import { S3Client, PutObjectCommand, HeadObjectCommand } from '@aws-sdk/client-s3';
import { getSignedUrl } from '@aws-sdk/s3-request-presigner';

import { RateLimiter } from './lib/ratelimit';

interface ClipData {
  id: string;
  timestamp: number;
  views: View[];
}

export default class Server implements Party.Server {
  private roomHandler: RoomHandler;
  private syncHandler: SyncHandler;
  private rateLimiter: RateLimiter;
  private s3Client: S3Client | null = null;
  private logger: Logger;

  // Map connection ID to User ID for disconnect handling
  private connToUser: Map<string, string> = new Map();

  // Map User ID to timeout timer for grace period
  private pendingDisconnects: Map<string, ReturnType<typeof setTimeout>> = new Map();

  constructor(readonly room: Party.Room) {
    this.roomHandler = new RoomHandler(room);
    this.syncHandler = new SyncHandler();
    this.rateLimiter = new RateLimiter();
    this.logger = new Logger({ service: 'SignalingServer' });
  }

  // Helper to get env vars safely (supports dev .env and prod secrets)
  private getEnv(key: string): string | undefined {
    const value = (this.room.env[key] as string) || (process.env[key] as string);
    return value ? value.trim() : undefined;
  }

  private getS3Client(): S3Client | null {
    if (this.s3Client) return this.s3Client;

    const accessKeyId = this.getEnv('R2_ACCESS_KEY_ID');
    const secretAccessKey = this.getEnv('R2_SECRET_ACCESS_KEY');
    const accountId = this.getEnv('R2_ACCOUNT_ID');

    if (!accessKeyId || !secretAccessKey || !accountId) {
      this.logger.error('❌ Missing R2 Environment Variables');
      return null;
    }

    this.s3Client = new S3Client({
      region: 'auto',
      endpoint: `https://${accountId}.r2.cloudflarestorage.com`,
      credentials: {
        accessKeyId,
        secretAccessKey,
      },
      requestChecksumCalculation: 'WHEN_REQUIRED',
      responseChecksumValidation: 'WHEN_REQUIRED',
      forcePathStyle: true,
    });
    this.logger.info('✅ S3Client initialized');
    return this.s3Client;
  }

  async onStart() {
    // Clear all storage on startup to prevent zombie members from previous runs
    // This ensures a clean state if the server restarts
    const stored = await this.room.storage.list();
    for (const key of stored.keys()) {
      await this.room.storage.delete(key);
    }
    this.logger.info(`Cleared ${stored.size} items from storage on start`);
  }

  async onConnect(conn: Party.Connection, _ctx: Party.ConnectionContext) {
    this.logger.info(`Connected: ${conn.id} to room ${this.room.id}`);

    // Send existing clips to new connection
    const clips = await this.room.storage.list<unknown>();
    const clipList: unknown[] = [];
    for (const [key, value] of clips) {
      if (key.startsWith('clip:')) {
        clipList.push(value);
      }
    }

    if (clipList.length > 0) {
      conn.send(
        JSON.stringify({
          type: 'CLIP_LIST',
          clips: clipList,
        })
      );
    }
  }

  async onMessage(message: string, sender: Party.Connection) {
    try {
      const parsed = JSON.parse(message);
      const result = ClientMessageSchema.safeParse(parsed);

      if (!result.success) {
        this.logger.error('Invalid message:', result.error);
        return;
      }

      const msg = result.data as ClientMessage;

      // Check Rate Limit
      if (!this.rateLimiter.checkLimit(sender.id, msg.type)) {
        this.logger.warn(`Rate limit exceeded for ${sender.id} on ${msg.type}`);
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
        case 'JOIN_ROOM': {
          // Cancel any pending disconnect for this user
          const pendingTimeout = this.pendingDisconnects.get(msg.userId);
          if (pendingTimeout) {
            this.logger.info(`Cancelling pending disconnect for ${msg.userId}`);
            clearTimeout(pendingTimeout);
            this.pendingDisconnects.delete(msg.userId);
          }

          this.connToUser.set(sender.id, msg.userId);
          // Persist connection mapping for recovery after restart
          await this.room.storage.put(`conn:${sender.id}`, msg.userId);
          await this.roomHandler.handleJoin(sender, msg);
          break;
        }
        case 'LEAVE_ROOM': {
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
        }
        case 'TIME_SYNC_REQUEST':
          this.syncHandler.handleTimeSync(sender, msg);
          break;
        case 'TRIGGER_CLIP': {
          const clipId = crypto.randomUUID();

          // Just broadcast the start event. Clients will request their own upload URLs.
          const startClipMsg = {
            type: 'START_CLIP',
            clipId,
            segmentCount: msg.segmentCount || 60,
            referenceTime: Date.now(),
          };

          // Initialize clip metadata in storage (will be updated as uploads complete)
          await this.room.storage.put(`clip:${clipId}`, {
            id: clipId,
            timestamp: Date.now(),
            views: [], // Track multiple views here
          });

          this.room.broadcast(JSON.stringify(startClipMsg));
          this.logger.info(`Clip ${clipId} triggered by ${sender.id}`);
          break;
        }

        case 'REQUEST_UPLOAD_URL': {
          if (!msg.clipId) return;

          const s3 = this.getS3Client();
          const bucketName = this.getEnv('R2_BUCKET_NAME');
          const requestUserId = this.connToUser.get(sender.id);

          if (s3 && bucketName && requestUserId) {
            // Unique key per user: uploads/{roomId}/{clipId}/{userId}.mp4
            const key = `uploads/${this.room.id}/${msg.clipId}/${requestUserId}.mp4`;

            try {
              const command = new PutObjectCommand({
                Bucket: bucketName,
                Key: key,
                ContentType: 'video/mp4',
              });

              const uploadUrl = await getSignedUrl(s3, command, { expiresIn: 900 });
              this.logger.info(`✅ Generated Presigned URL for ${key}`);

              sender.send(
                JSON.stringify({
                  type: 'UPLOAD_URL_GRANTED',
                  clipId: msg.clipId,
                  uploadUrl,
                  filename: `${requestUserId}.mp4`,
                })
              );
            } catch (err) {
              this.logger.error('❌ Failed to generate presigned URL:', err);
              sender.send(
                JSON.stringify({
                  type: 'ERROR',
                  code: 'UPLOAD_ERROR',
                  message: 'Failed to generate upload URL',
                })
              );
            }
          }
          break;
        }

        case 'UPLOAD_COMPLETE': {
          this.logger.info('Upload complete from', sender.id, 'for clip', msg.clipId);

          // Validate timestamp (Fail Hard Policy)
          if (!msg.videoStartTimeMs || msg.videoStartTimeMs < 1700000000000) {
            this.logger.error(`❌ Invalid videoStartTimeMs: ${msg.videoStartTimeMs}`);
            sender.send(
              JSON.stringify({
                type: 'ERROR',
                code: 'INVALID_TIMESTAMP',
                message: 'Missing or invalid videoStartTimeMs',
              })
            );
            return;
          }

          const s3Verify = this.getS3Client();
          const bucketVerify = this.getEnv('R2_BUCKET_NAME');
          const verifyUserId = this.connToUser.get(sender.id);

          if (s3Verify && bucketVerify && verifyUserId) {
            const keyVerify = `uploads/${this.room.id}/${msg.clipId}/${verifyUserId}.mp4`;
            const playbackUrl = `https://clips.fluxreplay.com/${keyVerify}`;

            try {
              await s3Verify.send(
                new HeadObjectCommand({
                  Bucket: bucketVerify,
                  Key: keyVerify,
                })
              );
              this.logger.info(`✅ File verified in R2: ${keyVerify}`);

              // Update clip metadata
              const clipKey = `clip:${msg.clipId}`;
              const clipData = (await this.room.storage.get<ClipData>(clipKey)) || {
                id: msg.clipId,
                timestamp: Date.now(),
                views: [],
              };

              // Add or update this user's view
              const view: View = {
                author: verifyUserId, // Using userId as author for now, could be display name
                url: playbackUrl,
                timestamp: Date.now(),
                videoStartTimeMs: msg.videoStartTimeMs,
                durationMs: msg.durationMs,
              };

              // Remove existing view from same author if any
              clipData.views = (clipData.views || []).filter((v) => v.author !== verifyUserId);
              clipData.views.push(view);

              await this.room.storage.put(clipKey, clipData);

              // Notify all clients of the updated clip
              this.room.broadcast(
                JSON.stringify({
                  type: 'CLIP_UPDATED',
                  clipId: msg.clipId,
                  view,
                })
              );

              sender.send(
                JSON.stringify({
                  type: 'UPLOAD_VERIFIED',
                  code: 'UPLOAD_VERIFIED',
                  message: `File verified: ${keyVerify}`,
                })
              );
            } catch (err) {
              this.logger.error(`❌ File NOT found in R2: ${keyVerify}`, err);
              sender.send(
                JSON.stringify({
                  type: 'ERROR',
                  code: 'UPLOAD_VERIFICATION_FAILED',
                  message: `File NOT found in R2: ${keyVerify}`,
                })
              );
            }
          }
          break;
        }
      }
    } catch (e) {
      this.logger.error('Error processing message:', e);
    }
  }

  async onClose(conn: Party.Connection) {
    this.logger.info(`Disconnected: ${conn.id}`);
    let userId = this.connToUser.get(conn.id);

    if (!userId) {
      // Try to recover from storage
      userId = await this.room.storage.get<string>(`conn:${conn.id}`);
    }

    if (userId) {
      // Don't leave immediately. Wait 10s to see if they reconnect.
      this.logger.info(`Scheduling disconnect for ${userId} in 10s`);

      // Clear any existing timeout for this user just in case
      const existing = this.pendingDisconnects.get(userId);
      if (existing) clearTimeout(existing);

      const timeout = setTimeout(async () => {
        if (!userId) return; // Should not happen due to closure, but safe check

        this.logger.info(`Executing delayed disconnect for ${userId}`);
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
