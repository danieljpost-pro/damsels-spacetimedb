//! Room Activity models — activity instances occurring in rooms.

use spacetimedb::{table, Timestamp};
use super::enums::RoomActivityStatus;

/// An instance of an Activity occurring in a Room.
///
/// Created when players roll for an activity.
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

    /// When the activity started.
    pub started_at: Timestamp,

    /// When the activity was completed (if completed).
    pub completed_at: Option<Timestamp>,
}

/// A Player participating in a RoomActivity.
///
/// Subset of room members selected to participate in this activity instance.
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

    /// Whether this player has completed the activity.
    pub completed: bool,

    /// Player ID who marked this participant as complete.
    pub completed_by: Option<u64>,
}

