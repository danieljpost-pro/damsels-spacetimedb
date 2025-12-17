//! Player History — permanent record of activity participation.

use spacetimedb::{table, Timestamp};
use super::enums::PlayerRole;

/// A permanent record of a Player's participation in an Activity.
///
/// This is an audit log — once created, entries are never deleted.
/// Separate from PlayerActivity which tracks unlock/prerequisite status.
#[table(name = player_history, public)]
pub struct PlayerHistory {
    /// Unique identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// The player who participated.
    #[index(btree)]
    pub player_id: u64,

    /// The activity they participated in.
    #[index(btree)]
    pub activity_id: u64,

    /// The room where this occurred.
    #[index(btree)]
    pub room_id: u64,

    /// The role the player had during this activity.
    pub role: PlayerRole,

    /// When the activity was completed.
    pub completed_at: Timestamp,
}

