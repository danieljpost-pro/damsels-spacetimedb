//! Player-Activity relationship models — tracking progress and vouching.

use spacetimedb::{table, Timestamp};
use super::enums::{ActivityKind, ActivityStatus};

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

/// An activity that is currently available/unlocked for a player.
///
/// This table is automatically maintained by the server and pushed to clients.
/// When a player's XP increases or they complete prerequisites, new activities
/// may be unlocked and added here.
///
/// Clients subscribe to this table to receive real-time updates about newly
/// available activities.
#[table(name = player_unlocked_activity, public)]
pub struct PlayerUnlockedActivity {
    /// Unique identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// The player who has this activity unlocked.
    #[index(btree)]
    pub player_id: u64,

    /// The unlocked activity.
    #[index(btree)]
    pub activity_id: u64,

    /// Activity name (denormalized for client convenience).
    pub activity_name: String,

    /// Activity description.
    pub activity_description: String,

    /// Category ID.
    pub category_id: u64,

    /// Category name (denormalized).
    pub category_name: String,

    /// Skill or Activity.
    pub kind: ActivityKind,

    /// XP required (already met since unlocked).
    pub xp_required: u64,

    /// XP reward for completing.
    pub xp_reward: u64,

    /// When this activity was unlocked for the player.
    pub unlocked_at: Timestamp,

    /// Whether this was newly unlocked (set to true on insert, can be marked false after client acknowledges).
    pub is_new: bool,
}

