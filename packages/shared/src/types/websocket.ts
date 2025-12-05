import { RoomMember, RoomState } from './room';
import { ClipMetadata } from './clip';

// ============ Client → Server ============

export interface JoinRoomMessage {
  type: 'JOIN_ROOM';
  roomId: string;
  userId: string;
  displayName: string;
}

export interface LeaveRoomMessage {
  type: 'LEAVE_ROOM';
}

export interface TimeSyncRequestMessage {
  type: 'TIME_SYNC_REQUEST';
  clientTime: number;
}

export interface TriggerClipMessage {
  type: 'TRIGGER_CLIP';
  segmentCount: number; // Default 60
}

export interface RequestUploadUrlMessage {
  type: 'REQUEST_UPLOAD_URL';
  clipId: string;
}

export interface UploadCompleteMessage {
  type: 'UPLOAD_COMPLETE';
  clipId: string;
  key?: string;
  videoStartTimeMs: number;
}

export type ClientMessage =
  | JoinRoomMessage
  | LeaveRoomMessage
  | TimeSyncRequestMessage
  | TriggerClipMessage
  | RequestUploadUrlMessage
  | UploadCompleteMessage;

// ============ Server → Client ============

export interface RoomStateMessage {
  type: 'ROOM_STATE';
  state: RoomState;
}

export interface MemberJoinedMessage {
  type: 'MEMBER_JOINED';
  member: RoomMember;
}

export interface MemberLeftMessage {
  type: 'MEMBER_LEFT';
  userId: string;
}

export interface TimeSyncResponseMessage {
  type: 'TIME_SYNC_RESPONSE';
  clientTime: number; // Echo back
  serverReceive: number; // When server got request
  serverSend: number; // When server sent response
}

export interface StartClipMessage {
  type: 'START_CLIP';
  clipId: string;
  segmentCount: number;
  referenceTime: number;
  // uploadUrl removed, client must request it
}

export interface UploadUrlGrantedMessage {
  type: 'UPLOAD_URL_GRANTED';
  clipId: string;
  uploadUrl: string;
  filename: string;
}

export interface View {
  author: string;
  url: string;
  timestamp: number;
  videoStartTimeMs: number;
}

export interface ClipUpdatedMessage {
  type: 'CLIP_UPDATED';
  clipId: string;
  view: View;
}

export interface ClipReadyMessage {
  type: 'CLIP_READY';
  clip: ClipMetadata;
}

export interface AllClipsReadyMessage {
  type: 'ALL_CLIPS_READY';
  clipId: string;
  clips: ClipMetadata[];
}

export interface ErrorMessage {
  type: 'ERROR';
  code: string;
  message: string;
}

export type ServerMessage =
  | RoomStateMessage
  | MemberJoinedMessage
  | MemberLeftMessage
  | TimeSyncResponseMessage
  | StartClipMessage
  | UploadUrlGrantedMessage
  | ClipUpdatedMessage
  | ClipReadyMessage
  | AllClipsReadyMessage
  | UploadVerifiedMessage
  | ErrorMessage;

export interface UploadVerifiedMessage {
  type: 'UPLOAD_VERIFIED';
  code: string;
  message: string;
}
