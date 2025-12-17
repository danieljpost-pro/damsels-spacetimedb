//! Player model — represents a user in the game.

use spacetimedb::{table, Identity, Timestamp};

/// A player in the Damsels game.
///
/// Linked to OAuth identity via SpacetimeDB Identity.
#[table(name = player, public)]
pub struct Player {
    /// Unique player identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// SpacetimeDB identity (linked to OAuth provider).
    #[unique]
    pub identity: Identity,

    /// Display name chosen by the player.
    #[unique]
    pub username: String,

    /// Total experience points accumulated.
    pub xp: u64,

    /// When the player account was created.
    pub created_at: Timestamp,

    /// Last time the player was active.
    pub last_seen: Timestamp,
}

