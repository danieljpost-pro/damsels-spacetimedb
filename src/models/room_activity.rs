//! Room Activity models — activity instances occurring in rooms.

use spacetimedb::{table, Timestamp};
use super::enums::{PlayerRole, RoomActivityStatus};

/// An instance of an Activity occurring in a Room.
///
/// Created when players select an activity to view or perform.
/// Lifecycle: Viewing → InProgress → Completed (or Cancelled at any point)
#[derive(Clone)]
#[table(name = room_activity, public)]
pub struct RoomActivity {
    /// Unique identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// The room where this activity is occurring.
    #[index(btree)]
    pub room_id: u64,

    /// The activity being performed.
    #[index(btree)]
    pub activity_id: u64,

    /// Current status.
    pub status: RoomActivityStatus,

    /// Player who selected/started this activity.
    pub started_by: u64,

    /// When the activity was selected for viewing.
    pub created_at: Timestamp,

    /// When the activity moved to InProgress (if started).
    pub started_at: Option<Timestamp>,

    /// When the activity was completed (if completed).
    pub completed_at: Option<Timestamp>,
}

/// A Player participating in a RoomActivity.
///
/// Created when activity moves to InProgress. Records all room members
/// at that moment as participants with their current roles.
#[derive(Clone)]
#[table(name = activity_participant, public)]
pub struct ActivityParticipant {
    /// Unique identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// The room activity instance.
    #[index(btree)]
    pub room_activity_id: u64,

    /// The participating player.
    #[index(btree)]
    pub player_id: u64,

    /// The player's role during this activity.
    pub role: PlayerRole,

    /// XP earned from this activity (set on completion).
    pub xp_earned: u64,

    /// Whether this player has been credited for the activity.
    pub completed: bool,
}

