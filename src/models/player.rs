//! Player model — represents a game identity/persona.

use spacetimedb::{table, Timestamp};

/// A player identity in the Damsels game.
///
/// A User can have multiple Player identities (personas).
/// Each Player has its own display name, XP, and game progress.
/// When entering a room, a User selects which Player identity to use.
#[derive(Clone)]
#[table(name = player, public)]
pub struct Player {
    /// Unique player identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// Reference to the User this player belongs to.
    /// A User can have multiple Players.
    #[index(btree)]
    pub user_id: u64,

    /// Display name for this player identity (unique across all players).
    #[unique]
    pub username: String,

    /// Total experience points accumulated by this player.
    pub xp: u64,

    /// When this player identity was created.
    pub created_at: Timestamp,
}
