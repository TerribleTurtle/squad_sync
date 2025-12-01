export interface RoomMember {
  userId: string;
  displayName: string;
  isRecording: boolean;
  lastSeen: number;
}

export interface RoomState {
  id: string;
  members: RoomMember[];
  serverTime: number;
}
