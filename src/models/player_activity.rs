//! Player-Activity relationship models — tracking progress and vouching.

use spacetimedb::{table, Timestamp};
use super::enums::ActivityStatus;

/// Tracks a Player's status for a specific Activity.
///
/// This represents the player's progress in their Activity Tree.
#[table(name = player_activity, public)]
pub struct PlayerActivity {
    /// Unique identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// The player.
    #[index(btree)]
    pub player_id: u64,

    /// The activity.
    #[index(btree)]
    pub activity_id: u64,

    /// Current status of this activity for this player.
    pub status: ActivityStatus,

    /// When the activity was completed (if completed).
    pub completed_at: Option<Timestamp>,

    /// Player ID who marked this as complete (self or another player).
    pub completed_by: Option<u64>,

    /// True if this was unlocked via vouch rather than natural progression.
    pub vouched: bool,
}

/// Audit record of a prerequisite vouch.
///
/// When Player A vouches that Player B has fulfilled prerequisites
/// for an Activity, allowing B to unlock it.
#[table(name = prerequisite_vouch, public)]
pub struct PrerequisiteVouch {
    /// Unique identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// Player who vouched (Player A).
    #[index(btree)]
    pub voucher_id: u64,

    /// Player who received the unlock (Player B).
    #[index(btree)]
    pub recipient_id: u64,

    /// Activity that was unlocked.
    #[index(btree)]
    pub activity_id: u64,

    /// When the vouch occurred.
    pub created_at: Timestamp,
}

