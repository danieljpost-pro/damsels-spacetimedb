//! Enum definitions for the Damsels game.

use spacetimedb::SpacetimeType;

/// Role a Player chooses within a Room.
#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerRole {
    /// Top role.
    Top,
    /// Bottom role.
    Bottom,
    /// Observer — watches but does not participate.
    Observer,
    /// Photographer — documents the activity.
    Photographer,
    /// Activity Admin — can create/edit activities.
    /// 
    /// # Development Only
    /// 
    /// This variant is only available when compiled with `--features dev`.
    /// It is excluded from production builds entirely.
    #[cfg(feature = "dev")]
    ActivityAdmin,
}

/// Status of an Activity for a specific Player.
#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivityStatus {
    /// Prerequisites not met and not vouched.
    Locked,
    /// Ready to play — prerequisites met or vouched.
    Available,
    /// Activity has been completed.
    Completed,
}

/// Status of an Activity instance occurring in a Room.
#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomActivityStatus {
    /// Activity is currently in progress.
    InProgress,
    /// Activity has been completed.
    Completed,
    /// Activity was cancelled.
    Cancelled,
}

/// Status of an Invitation.
#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvitationStatus {
    /// Awaiting response.
    Pending,
    /// Invitation was accepted.
    Accepted,
    /// Invitation was declined.
    Declined,
}

