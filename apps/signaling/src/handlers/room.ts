import type * as Party from 'partykit/server';
import {
  RoomState,
  RoomMember,
  JoinRoomMessage,
  LeaveRoomMessage,
  MemberJoinedMessage,
  MemberLeftMessage,
  RoomStateMessage,
} from '@squadsync/shared';

export class RoomHandler {
  constructor(private room: Party.Room) {}

  async handleJoin(conn: Party.Connection, message: JoinRoomMessage): Promise<void> {
    const member: RoomMember = {
      userId: message.userId,
      displayName: message.displayName,
      isRecording: false,
      lastSeen: Date.now(),
    };

    // Store member in room storage
    await this.room.storage.put(`member:${member.userId}`, member);

    // Broadcast new member to others
    const joinMsg: MemberJoinedMessage = {
      type: 'MEMBER_JOINED',
      member,
    };
    this.room.broadcast(JSON.stringify(joinMsg), [conn.id]);

    // Send current room state to new member
    const members = await this.getMembers();
    const stateMsg: RoomStateMessage = {
      type: 'ROOM_STATE',
      state: {
        id: this.room.id,
        members,
        serverTime: Date.now(),
      },
    };
    conn.send(JSON.stringify(stateMsg));
  }

  async handleLeave(conn: Party.Connection, userId?: string): Promise<void> {
    // If userId not provided, try to find it (implementation detail: might need a connection->user mapping)
    // For now, assume userId is passed or we iterate.
    // Optimization: Store connectionId -> userId mapping in memory or storage.

    if (!userId) return;

    await this.room.storage.delete(`member:${userId}`);

    const leaveMsg: MemberLeftMessage = {
      type: 'MEMBER_LEFT',
      userId,
    };
    this.room.broadcast(JSON.stringify(leaveMsg));
  }

  async getMembers(): Promise<RoomMember[]> {
    const members = await this.room.storage.list<RoomMember>({
      prefix: 'member:',
    });
    return Array.from(members.values());
  }
}
