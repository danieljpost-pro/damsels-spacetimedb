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

/// Distinguishes between Skills and Activities.
///
/// A Skill is a foundational ability that players can demonstrate.
/// An Activity is a full experience that may require Skills or other Activities.
///
/// Key difference: Skills are atomic capabilities; Activities are composed experiences.
/// Both can be prerequisites for other Activities.
///
/// When a random dice roll occurs for activity selection, the system will
/// filter based on player Preferences (configured separately) and available
/// unlocked activities.
#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivityKind {
    /// A foundational skill that can be demonstrated.
    /// Skills are typically simpler and can serve as building blocks.
    Skill,
    /// A full activity experience.
    /// May require Skills or other Activities as prerequisites.
    Activity,
}

