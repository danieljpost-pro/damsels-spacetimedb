//! Room models — ephemeral game sessions.

use spacetimedb::{table, Timestamp};
use super::enums::{InvitationStatus, PlayerRole};

/// An ephemeral game room where Players gather.
///
/// Rooms exist until all players exit.
#[table(name = room, public)]
pub struct Room {
    /// Unique room identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// Shareable room code (e.g., "ABCD-1234").
    #[unique]
    pub code: String,

    /// Player who created the room.
    #[index(btree)]
    pub owner_id: u64,

    /// When the room was created.
    pub created_at: Timestamp,
}

/// A Player's membership in a Room.
#[table(name = room_member, public)]
pub struct RoomMember {
    /// Unique identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// The room.
    #[index(btree)]
    pub room_id: u64,

    /// The player.
    #[index(btree)]
    pub player_id: u64,

    /// Role chosen by the player for this room session.
    pub role: PlayerRole,

    /// When the player joined.
    pub joined_at: Timestamp,
}

/// An invitation to join an activity within a Room.
///
/// Invitations are between players already in the same Room.
#[table(name = invitation, public)]
pub struct Invitation {
    /// Unique identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// The room where this invitation was sent.
    #[index(btree)]
    pub room_id: u64,

    /// Player who sent the invitation.
    #[index(btree)]
    pub inviter_id: u64,

    /// Player who received the invitation.
    #[index(btree)]
    pub invitee_id: u64,

    /// Current status of the invitation.
    pub status: InvitationStatus,

    /// When the invitation was sent.
    pub created_at: Timestamp,
}

